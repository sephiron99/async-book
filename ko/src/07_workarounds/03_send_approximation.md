# `Send` Approximation

(TODO: 재번역) 몇몇 `async fn` 상태기계는 스레드 간 이동에 안전하지만 나머지는
그렇지 않습니다. `async fn` `Future`이 `Send`인지 여부는 `Send`가 아닌 타입이
`.await` 포인트에 걸쳐 홀드되는지 여부에 달려 있습니다. 컴파일러 값이 `.await`
포인트에 걸쳐 홀드될지도 모를 때, 무엇을(TODO: ) 추정하려고 최선을 다합니다.
하지만, 이런 분석은 오늘날 많은 경우에 너무 보수적입니다.

예를 들어, 간단한 비(非) `Send` 타입이 `Rc`를 가지고 있을지도 모른다고
가정해봅시다.

```rust
use std::rc::Rc;

#[derive(Default)]
struct NotSend(Rc<()>);
```

Variables of type `NotSend` can briefly appear as temporaries in `async fn`s
even when the resulting `Future` type returned by the `async fn` must be `Send`:

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

However, if we change `foo` to store `NotSend` in a variable, this example no
longer compiles:

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

This error is correct. If we store `x` into a variable, it won't be dropped
until after the `.await`, at which point the `async fn` may be running on
a different thread. Since `Rc` is not `Send`, allowing it to travel across
threads would be unsound. One simple solution to this would be to `drop`
the `Rc` before the `.await`, but unfortunately that does not work today.

In order to successfully work around this issue, you may have to introduce
a block scope encapsulating any non-`Send` variables. This makes it easier
for the compiler to tell that these variables do not live across an
`.await` point.

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
