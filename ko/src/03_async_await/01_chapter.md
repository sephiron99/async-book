# `async`/`.await`

In [the first chapter], we took a brief look at `async`/`.await`.
This chapter will discuss `async`/`.await` in
greater detail, explaining how it works and how `async` code differs from
traditional Rust programs.

첫 장에서 우리는 `async`/`.await`을 짤막하게 다뤘습니다. 이제부터 `async` 코드가 
어떻게 동작하고 어떻게 전통적인 러스트 프로그램과 다른지 더 자세한 설명을 통해 들여다봅시다.

`async`/`.await` are special pieces of Rust syntax that make it possible to
yield control of the current thread rather than blocking, allowing other
code to make progress while waiting on an operation to complete.

`async`/`.await`은 

`async fn` 이나 `async` 블록같이 `async`를 사용하는 두 가지 주요 방법이 있습니다. 각각 
`Future` 트레잇을 구현한 값을 반환합니다.

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:async_fn_and_block_examples}}
```

As we saw in the first chapter, `async` bodies and other futures are lazy:
they do nothing until they are run. The most common way to run a `Future`
is to `.await` it. When `.await` is called on a `Future`, it will attempt
to run it to completion. If the `Future` is blocked, it will yield control
of the current thread. When more progress can be made, the `Future` will be picked
up by the executor and will resume running, allowing the `.await` to resolve.

첫 장에서 봤듯이, `async` 안과 다른 future 구현체는 게으릅니다. 즉, 실행될 때까지 
아무 것도 안 합니다. `Future`를 실행하기 위해서는 `.await`을 써야 합니다. `Future` 상에서 
`.await`이 나오면 그 코드는 끝날 때까지 실행하려 합니다. `Future`가 막히면, 현재 스레드에서 
제어를 멈춥니다. 더 많은 작업이 만들어질 시, `Future`는 

## `async`적인 생명주기

보통 함수와 달리, 참조나 `static`하지 않은 인자를 다루는 `async fn`은 인자의 
수명이 나누는 `Future`를 반환합니다.

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:lifetimes_expanded}}
```

This means that the future returned from an `async fn` must be `.await`ed
while its non-`'static` arguments are still valid. In the common
case of `.await`ing the future immediately after calling the function
(as in `foo(&x).await`) this is not an issue. However, if storing the future
or sending it over to another task or thread, this may be an issue.

즉 `async fn`이 반환한 future는 `static`하지 않은 인자가 여전히 유효할 시 `.await`해야 
합니다. 

One common workaround for turning an `async fn` with references-as-arguments
into a `'static` future is to bundle the arguments with the call to the
`async fn` inside an `async` block:

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:static_future_with_borrow}}
```

By moving the argument into the `async` block, we extend its lifetime to match
that of the `Future` returned from the call to `good`.

인자를 `async` 블록으로 옮김으로써, 우리는 인자의 수명을

## `async move`

`async` blocks and closures allow the `move` keyword, much like normal
closures. An `async move` block will take ownership of the variables it
references, allowing it to outlive the current scope, but giving up the ability
to share those variables with other code:

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:async_move_examples}}
```

## 다중스레드 실행자 위에 `.await`함

Note that, when using a multithreaded `Future` executor, a `Future` may move
between threads, so any variables used in `async` bodies must be able to travel
between threads, as any `.await` can potentially result in a switch to a new
thread.

This means that it is not safe to use `Rc`, `&RefCell` or any other types
that don't implement the `Send` trait, including references to types that don't
implement the `Sync` trait.

(Caveat: it is possible to use these types so long as they aren't in scope
during a call to `.await`.)

Similarly, it isn't a good idea to hold a traditional non-futures-aware lock
across an `.await`, as it can cause the threadpool to lock up: one task could
take out a lock, `.await` and yield to the executor, allowing another task to
attempt to take the lock and cause a deadlock. To avoid this, use the `Mutex`
in `futures::lock` rather than the one from `std::sync`.

[the first chapter]: ../01_getting_started/04_async_await_primer.md
