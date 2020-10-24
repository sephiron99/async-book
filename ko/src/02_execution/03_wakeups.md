# `Waker`로 태스크 깨우기

future들이 첫 번째 `poll`에서는 완성되지 못하는 것이 일반적입니다. 완성되지
못했을 경우, 더 진행이 가능할 준비가 되면 future가 poll될 수 있게 확실히
조치해둘 필요가 있습니다. `Waker` 타입으로 이 조치를 취할 수 있습니다.

future가 poll될 때마다 한 "태스크"의 일부분으로서 poll됩니다. 태스크란 한
executor에게 제공된 최상위 future들입니다.

`Waker`는 `wake()` 메소드를 제공하는데, 이 메소드는 연관된 태스크가 깨워져야
한다고 executor에게 알리는데 사용됩니다. `wake()`가 호출되었을 때, executor는
`Waker`와 연관된 태스크가 진행될 준비가 되었으며, 태스크의 future가 다시
poll되어야 한다는 것을 알 수 있습니다.

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

먼저 future 타입 자체를 정의합시다. 우리의 future에게는 타이머가 경과되었는지,
그래서 future가 완성되어야 하는지 여부를 스레드와 통신할 방법이 필요합니다.
그래서 공유된 `Arc<Mutex<..>>` 값을 사용해서 스레드와 future 사이에 통신할
것입니다.

```rust,ignore
{{#include ../../examples/02_03_timer/src/lib.rs:timer_decl}}
```

자, 진짜로 `Future` 구현을 작성해 봅시다.

```rust,ignore
{{#include ../../examples/02_03_timer/src/lib.rs:future_for_timer}}
```

꽤 간단하죠? 스레드가 `shared_state.completed = true`로 설정하였다면 다 된
것입니다. 아니라면, 우리는 스레드가 태스크를 다시 깨울 수 있도록, `Waker`를 현재의
태스크용으로 클론하여 `shared_state.waker`에 전달합니다.

중요한 점은, `Waker`를 future가 poll될 때마다 갱신해야 한다는 점입니다.
왜냐하면, 그 future가 다른 `Waker`와 같이 다른 태스크로 이동했을 수 있기
때문입니다. 이런 상황은 future가 poll되고 나서 태스크 사이에서 여기저기 전달될 때
발생합니다. 

마지막으로, 실제로 타이머를 만들고 스레드를 시작할 API가 필요합니다.

```rust,ignore
{{#include ../../examples/02_03_timer/src/lib.rs:timer_new}}
```

짠! 이게 간단한 타이머 future를 만드는데 필요한 전부입니다. 이제 future가 실행될
executor만 있으면 되는데요...
