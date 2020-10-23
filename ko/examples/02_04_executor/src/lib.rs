#![cfg(test)]

// ANCHOR: imports
use {
    futures::{
        future::{BoxFuture, FutureExt},
        task::{waker_ref, ArcWake},
    },
    std::{
        future::Future,
        sync::mpsc::{sync_channel, Receiver, SyncSender},
        sync::{Arc, Mutex},
        task::{Context, Poll},
        time::Duration,
    },
    // 이전 장에서 작성한 타이머
    timer_future::TimerFuture,
};
// ANCHOR_END: imports

// ANCHOR: executor_decl
/// 채널에서 태스크를 받아서 실행하는 태스크 executor
struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

/// 새 future를 태스크 채널에 생성해 넣는 `Spawner`
#[derive(Clone)]
struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

/// `Executor`에게 poll될 수 있게 스스로를 재스케줄링하는 future
struct Task {
    /// 완성되기 위해서 큐에 넣어져야 하는, 진행중인 future
    ///
    /// 정확히 하자면, `Mutex`가 꼭 필요한 것은 아니다. 우리는 한 시점에
    /// (future들을 실행하는) 오직 하나의 스레드만 가지고 있기 때문이다. 하지만,
    /// 러스트는 우리의 `future`가 한 개의 스레드 안에서만 변경된다는 사실을 알
    /// 수 없기 때문에, 우리는 스레드 안전성을 위해 `Mutex`를 사용해야만 한다.
    /// 현업에서는 `Mutex` 대신 `UnsafeCell`을 사용할 수도 있다.
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    /// 태스크가 자기자신을 태스크 큐의 마지막에 넣는데 사용되는 핸들
    task_sender: SyncSender<Arc<Task>>,
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
    // 채널(큐)이 일시점에 가질 수 있는 태스크의 최대 갯수.
    // 그냥 `sync_channel`을 만드는데 필요한 것이고, 실제 executor에 적용될 일은
    // 없을 것이다.
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}
// ANCHOR_END: executor_decl

// ANCHOR: spawn_fn
impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("큐에 태스크가 너무 많습니다.");
    }
}
// ANCHOR_END: spawn_fn

// ANCHOR: arcwake_for_task
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // `wake`를 이 태스크를 다시 태스크 채널에 보내는 방식으로 구현한다. 그래서
        // executor가 이 태스크를 다시 poll할 것이다.
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("큐에 태스크가 너무 많습니다.");
    }
}
// ANCHOR_END: arcwake_for_task

// ANCHOR: executor_run
impl Executor {
    fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            // future를 취하고 나서, 아직 future가 완성되지 않았으면(아직 Some이면),
            // future를 완성하기 위해 poll한다.
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // task 자기자신으로부터 `LocalWaker`를 생성한다.
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);
                // `BoxFuture<T>`는 `Pin<Box<dyn Future<Output = T> + Send +
                // 'static>>`의 type alias이다.
                // `Pin::as_mut` 메소드를 호출하여 `BoxFuture<T>`로부터
                // `Pin<&mut dyn Future + Send + 'static>`을 얻을 수 있다.
                if let Poll::Pending = future.as_mut().poll(context) {
                    // future의 처리가 끝나지 않았으므로, 그것의 task에 도로
                    // 넣어서 미래에 다시 실행될 수 있게 한다.
                    *future_slot = Some(future);
                }
            }
        }
    }
}
// ANCHOR_END: executor_run

// ANCHOR: main
fn main() {
    let (executor, spawner) = new_executor_and_spawner();

    // 타이머 전후로 문자열을 출력할 태스크를 생성한다.
    spawner.spawn(async {
        println!("howdy!");
        // 우리의 타이머 future가 2초 후에 완성될 때까지 기다린다.
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("done!");
    });

    // 여러분의 executor가 spawner가 끝났음을 확인하고 더 이상 실행할 task를
    // 받지 않도록 spawner를 drop한다.
    drop(spawner);

    // excutor를 task 큐가 빌 때까지 실행한다. "howdy!" 출력, 일시중지,
    // "done!"출력 순으로 동작할 것이다.
    executor.run();
}
// ANCHOR_END: main

#[test]
fn run_main() {
    main()
}
