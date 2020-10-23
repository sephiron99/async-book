// ANCHOR: imports
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};
// ANCHOR_END: imports

// ANCHOR: timer_decl
pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

/// future와 대기중인 스레드 사이에 공유된 상태
struct SharedState {
    /// 타이머가 경과되었는 지 여부
    completed: bool,

    /// `TimerFuture`가 실행될 태스크 용 waker.
    /// 스레드는 `completed = true`라고 설정한 후에 `TimerFuture`의 태스크에게
    /// '일어나서 `completed = true`인지 확인하고 진행하라'고 전하는 데 이
    /// waker를 사용할 수 있다.
    waker: Option<Waker>,
}
// ANCHOR_END: timer_decl

// ANCHOR: future_for_timer
impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 타이머가 이미 완성되었는지 알기 위해 공유된 상태를 확인.
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            // waker를 설정해서 타이머가 완성되었을 때 스레드가 현재의 태스크를
            // 깨울 수 있게 한다. 이렇게 함으로써 future가 다시 poll되어
            // `completed = true`가 맞는지 확인 할 수 있다.
            //
            // waker를 매번 반복적으로 클론하지 않고 한 번만 클론하고 싶을 수도
            // 있다. 하지만, `TimerFuture`는 executor의 여러 태스크들로 이동할 수
            // 있기 때문에, 한 번만 클론하면 잘못된 태스크를 가리키는 정체된
            // waker가 만들어져 `TimerFuture`가 제대로 못 깨워질 것이다.
            //
            // 주의: `Waker::will_wake` 함수를 이용하여 `TimerFuture`가
            // 제대로 못 깨워지는 문제를 체크할 수 있으나, 예제를 간단하게 
            // 하기 위해 생략하였다.
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
// ANCHOR_END: future_for_timer

// ANCHOR: timer_new
impl TimerFuture {
    /// 주어진 시간이 경과하면 완성되는 새로운 `TimerFuture`를 만든다.
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        // 새로운 스레드 생성
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();
            // 타이머가 완성되어서 future가 poll된 마지막 태스크를 (존재한다면)
            // 깨우는 시그널
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake()
            }
        });

        TimerFuture { shared_state }
    }
}
// ANCHOR_END: timer_new

#[test]
fn block_on_timer() {
    futures::executor::block_on(async {
        TimerFuture::new(Duration::from_secs(1)).await
    })
}
