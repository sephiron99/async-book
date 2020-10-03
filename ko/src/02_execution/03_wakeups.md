# `Waker`로 Task 깨우기

future들이 첫 번째 `poll`에서는 완성되지 못하는 것이 일반적입니다. 완성되지
못했을 경우, 더 진행이 가능할 준비가 되었을 때, future가 poll될 수 있게 확실히
조치해둘 필요가 있습니다. `Waker` 타입으로 이 조치를 취할 수 있습니다.

future가 poll될 때마다 한 "task"의 일부분으로서 poll됩니다. task들이란 한 executor에게
제공된 최상위 future들입니다.

`Waker`는 executor에게 연관된 task가 깨워져야 한다고 알리는데 사용되는 `wake()`
메소드를 제공합니다. `wake()`가 호출되었을 때, executor는 `Waker`와 연관된
task가 진행될 준비가 되었으며, task의 future가 다시 poll되어야 한다는 것을 알 수 있습니다.

`Waker`는 `clone()`도 구현하기 때문에, 필요한 곳에 복사되고 저장될 수 있습니다.

`Waker`를 사용하여 간단한 타이머를 구현해 봅시다.

## 응용: 타이머 만들기

이 예제의 목적에 따라, 우리는 타이머가 만들어졌을 때 그냥 새 스레드를 하나
생성할 것이고, 필요한 만큼 sleep할 것입니다. 그리고 time window가 지나면,
타이머 future에 시그널을 보낼 것입니다.

시작하려면 다음처럼 import해야할 것들이 있습니다.

```rust
{{#include ../../examples/02_03_timer/src/lib.rs:imports}}
```

Let's start by defining the future type itself. Our future needs a way for the
thread to communicate that the timer has elapsed and the future should complete.
We'll use a shared `Arc<Mutex<..>>` value to communicate between the thread and
the future.

```rust,ignore
{{#include ../../examples/02_03_timer/src/lib.rs:timer_decl}}
```

Now, let's actually write the `Future` implementation!

```rust,ignore
{{#include ../../examples/02_03_timer/src/lib.rs:future_for_timer}}
```

Pretty simple, right? If the thread has set `shared_state.completed = true`,
we're done! Otherwise, we clone the `Waker` for the current task and pass it to
`shared_state.waker` so that the thread can wake the task back up.

Importantly, we have to update the `Waker` every time the future is polled
because the future may have moved to a different task with a different
`Waker`. This will happen when futures are passed around between tasks after
being polled.

Finally, we need the API to actually construct the timer and start the thread:

```rust,ignore
{{#include ../../examples/02_03_timer/src/lib.rs:timer_new}}
```

Woot! That's all we need to build a simple timer future. Now, if only we had
an executor to run the future on...
