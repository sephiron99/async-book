# `Send` 추정

몇몇 `async fn` 상태기계는 스레드 간 이동에 안전하지만 나머지는 그렇지 않습니다.
`async fn` `Future`가 `Send`인지 여부는 `Send`가 아닌 타입이 `.await`의 위치
앞뒤로 걸쳐서 유지되는지 여부에 달려 있습니다. 값들이 `.await`의 위치 앞뒤로
걸쳐서 유지될 가능성이 있을 때, 컴파일러는 `async fn` `Future`가 `Send`인지
여부를 추정하려고 최선을 다합니다. 하지만, 이런 분석은 오늘날 많은 경우에 너무
보수적입니다.

예를 들어, 간단한 비(非) `Send` 타입이 `Rc`를 가지고 있을지도 모른다고
가정해봅시다.

```rust
use std::rc::Rc;

#[derive(Default)]
struct NotSend(Rc<()>);
```

`async fn`이 반환한 마지막(TODO: Resulting, 결과적인)`Future` 타입이
`Send`이어야만 하는 경우에도, `NotSend` 타입의 변수들이 임시값이라면 `async fn`
안에서도 간편하게 사용할 수 있습니다.

```rust,edition2018
# use std::rc::Rc;
# #[derive(Default)]
# struct NotSend(Rc<()>);
async fn bar() {}
async fn foo() {
    NotSend::default();
    bar().await;
}

fn require_send(_: impl Send) {}

fn main() {
    require_send(foo());
}
```

하지만, `foo`를 수정하여 한 변수 안에 `NotSend`를 저장하는 코드를 추가한다면, 이
예제는 컴파일되지 않습니다.

```rust,edition2018
# use std::rc::Rc;
# #[derive(Default)]
# struct NotSend(Rc<()>);
# async fn bar() {}
async fn foo() {
    let x = NotSend::default();
    bar().await;
}
# fn require_send(_: impl Send) {}
# fn main() {
#    require_send(foo());
# }
```

```
error[E0277]: `std::rc::Rc<()>` cannot be sent between threads safely
  --> src/main.rs:15:5
   |
15 |     require_send(foo());
   |     ^^^^^^^^^^^^ `std::rc::Rc<()>` cannot be sent between threads safely
   |
   = help: within `impl std::future::Future`, the trait `std::marker::Send` is not implemented for `std::rc::Rc<()>`
   = note: required because it appears within the type `NotSend`
   = note: required because it appears within the type `{NotSend, impl std::future::Future, ()}`
   = note: required because it appears within the type `[static generator@src/main.rs:7:16: 10:2 {NotSend, impl std::future::Future, ()}]`
   = note: required because it appears within the type `std::future::GenFuture<[static generator@src/main.rs:7:16: 10:2 {NotSend, impl std::future::Future, ()}]>`
   = note: required because it appears within the type `impl std::future::Future`
   = note: required because it appears within the type `impl std::future::Future`
note: required by `require_send`
  --> src/main.rs:12:1
   |
12 | fn require_send(_: impl Send) {}
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: aborting due to previous error

For more information about this error, try `rustc --explain E0277`.
```

이 에러는 문제를 정확히 나타내 줍니다. `x`를 변수에 저장한다면, `x`는 `async
fn`이 다른 스레드에서 동작하고 있을 시점인 `.await`를 지난 다음에야 드랍될
것입니다. `Rc`는 `Send`가 아니기 때문에, `Rc`가 스레드 사이를 이동하게 만드는
것은 위험합니다. 이에 대한 간단한 해법은 `.await`이전에 `Rc`를 `drop`하는
것입니다만, 불행하게도 지금은 해당되지 않습니다.

이 이슈를 해결하기 위해서는, 모든 비 `Send` 변수를 캡슐화하는 블록 범위를
도입해야 할 것입니다. 이렇게 하면, 이 변수들이 `.await` 포인트에 걸쳐 존재하지
않는다는 사실을 컴파일러가 알게 하기 쉽습니다.

```rust,edition2018
# use std::rc::Rc;
# #[derive(Default)]
# struct NotSend(Rc<()>);
# async fn bar() {}
async fn foo() {
    {
        let x = NotSend::default();
    }
    bar().await;
}
# fn require_send(_: impl Send) {}
# fn main() {
#    require_send(foo());
# }
```
