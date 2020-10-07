# `async`/`.await`

첫 장에서 우리는 `async`/`.await`을 짧게 다뤘습니다. 이제 `async` 코드가
어떻게 동작하고, 어떻게 전형적인 러스트 프로그램과 다른지 더 자세히 들여다봅시다.

`async`/`.await`은 하나의 작업을 마칠 때 까지 기다리는 동안 다른 코드가 실행되는 것을 
막거나 허용하는 대신 현재 스레드의 제어를 양보해주는 러스트 문법의 특별한 도구입니다.

`async`를 다루는 주 방법 두 가지, `async fn`과 `async` 블록이 있습니다. 각각 `Future` 
트레잇을 구현한 값을 반환합니다.

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:async_fn_and_block_examples}}
```

첫 장에서 봤듯이, `async` 안쪽과 그 밖의 future 구현체는 게으릅니다. 즉 실행될 때까지 아무 
일도 안 합니다. `Future`를 실행하기 위해서는 `.await`을 써야 합니다. `Future` 상에서 
`.await`이 나오면 그 코드는 마칠 때까지 실행하려 할 것입니다. `Future`는 자신이 중지되면 
현재 스레드의 제어를 넘겨줍니다. 더 많은 흐름이 나올 시, `Future`는 `.await`을 풀고 실행자에게 
선택돼 실행을 재개할 것입니다.

## `async`한 수명

참조나 기타 `'static`하지 않은 인자를 다루는 `async fn`은 전형적인 함수와 달리 인자의 수명으로 
정해지는 `Future`를 반환합니다.

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:lifetimes_expanded}}
```

즉 `'static`하지 않은 인자가 여전히 유효하는 한 `async fn`에서 나온 future는 `.await`해야 
합니다. 보통 `foo(&x).await`와 같이 함수를 호출하고 바로 future를 `.await`할 때는 문제가 
없습니다. 허나 future를 또다른 작업이나 스레드로 저장하거나 보내면 문제가 생길 수 있습니다.

참조를 인자로 가진 `async fn`을 `'static`한 future로 바꾸는 방법 한 가지는 `async fn` 호출과 
인자값을 하나의 `async` 블록으로 묶는 것입니다.

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:static_future_with_borrow}}
```

우리는 인자를 `async` 블록으로 옮김으로써 `good` 함수에서 나오는 `Future`의 것과 맞춰 인자의 
수명을 늘릴 수 있습니다.

## `async move`

`async` 블록과 클로저는 보통 클로저처럼 `move` 키워드를 허용합니다. `async move` 블록은
자신이 참조하는 변수의 소유권을 현재 구역보다 더 오래 살리면서 다른 코드가 공유할 수 없도록 합니다.

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:async_move_examples}}
```

## 멀티스레드 실행자 상에서 `.await`하기

`Future`는 멀티스레드 상에서 `Future` 실행자를 사용할 때, `async` 안쪽에 쓰인 어떤 
변수라도 스레드 사이를 넘나들 수 있어야 하기 위해 어떤 `.await`이라도 잠재적으로 새로운 
스레드로 갈아탈 수 있듯이 스레드 사이를 건널 수도 있습니다.

그 말인 즉슨 `Sync` 트레잇을 구현하지 않은 타입에 대한 참조를 비롯해 `Send` 트레잇을 
구현하지 않은 `Rc`, `&RefCell` 등 어떠한 타입도 안전하지 않습니다.

(알림: `.await`으로 호출하는 동안 구역 안에 있으면 이 타입들을 사용할 수 있습니다.)

요컨데 `.await`을 넘어서 future가 인식할 수 없는 전형적인 락을 다루는 것은 좋은 생각이 
아닙니다. 하나가 락을 차지하고, `.await`하면서 실행자에게 양보하면 또다른 작업이 락을 차지하려 
해 데드락이 발생하기 때문입니다. 이런 문제를 피하기 위해 `std::sync` 대신 `futures::lock`에 
있는 `Mutex`를 사용하시기 바랍니다.

[the first chapter]: ../01_getting_started/04_async_await_primer.md