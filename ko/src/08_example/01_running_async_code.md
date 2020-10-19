# 비동기 코드 실행하기

HTTP 서버는 동시에 여러 클라이언트에 동시에 서비스할 수 있어야 합니다. 즉, HTTP
서버는 현재의 리퀘스트를 처리하기 전에 기존의 리퀘스트가 끝나길 기다려서는 안된다는 말입니다.
러스트북의 예제에서는 모든 연결에 스레드를 하나씩 할당하는 스레드 풀을 만들어서 
[이 문제를 해결합니다.](https://doc.rust-lang.org/book/ch20-02-multithreaded.html#turning-our-single-threaded-server-into-a-multithreaded-server)

여기서는, 스레드를 추가하여 처리성능을 향상시키기 보다, 비동기 코드를
사용하여 같은 효과를 내 봅시다.

`handle_connection`의 선언을 `async fn`으로 수정하여 future를 반환하게 합시다.
```rust,ignore
{{#include ../../examples/08_02_async_tcp_server/src/main.rs:handle_connection_async}}
```

`async`를 `handle_connection` 선언에 추가하면 반환값이 유닛 타입 `()`에서
`Future<Output=()>`을 구현하는 타입으로 변경됩니다.

이 코드를 컴파일하면 작동되지 않을 것이라는 컴파일러 에러가 발생합니다.
```console
$ cargo check
    Checking async-rust v0.1.0 (file:///projects/async-rust)
warning: unused implementer of `std::future::Future` that must be used
  --> src/main.rs:12:9
   |
12 |         handle_connection(stream);
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_must_use)]` on by default
   = note: futures do nothing unless you `.await` or poll them
```

`handle_connection`은 그 반환값을 `await`하거나 `poll`하지 않았기 때문에, 전혀
실행되지 않을 것입니다. 서버를 실행하고 브라우저에서 `127.0.0.1:7878` 열면
연결이 거부됨을 알 수 있습니다. 서버가 요청을 처리하지 않는 것입니다.

비동기 코드 자체안에서 `await`하거나 `poll`할 수는 없습니다. future를
완성될때까지 스케쥴링하고 실행할 비동기 런타임이 필요합니다.
비동기 런타임, executor 그리고 reactor에 대한 자세한 정보를 원한다면 런타임
선택에 관한 장을 살펴보세요.

[//]: <> (TODO: Link to section on runtimes once complete.)

## Adding an Async Runtime
Here, we'll use an executor from the `async-std` crate.
The `#[async_std::main]` attribute from `async-std` allows us to write an asynchronous main function.
To use it, enable the `attributes` feature of `async-std` in `Cargo.toml`:
```toml
[dependencies.async-std]
version = "1.6"
features = ["attributes"]
```

As a first step, we'll switch to an asynchronous main function,
and `await` the future returned by the async version of `handle_connection`.
Then, we'll test how the server responds.
Here's what that would look like:
```rust
{{#include ../../examples/08_02_async_tcp_server/src/main.rs:main_func}}
```
Now, let's test to see if our server can handle connections concurrently.
Simply making `handle_connection` asynchronous doesn't mean that the server
can handle multiple connections at the same time, and we'll soon see why.

To illustrate this, let's simulate a slow request.
When a client makes a request to `127.0.0.1:7878/sleep`,
our server will sleep for 5 seconds:

```rust,ignore
{{#include ../../examples/08_03_slow_request/src/main.rs:handle_connection}}
```
This is very similar to the 
[simulation of a slow request](https://doc.rust-lang.org/book/ch20-02-multithreaded.html#simulating-a-slow-request-in-the-current-server-implementation)
from the Book, but with one important difference:
we're using the non-blocking function `async_std::task::sleep` instead of the blocking function `std::thread::sleep`.
It's important to remember that even if a piece of code is run within an `async fn` and `await`ed, it may still block.
To test whether our server handles connections concurrently, we'll need to ensure that `handle_connection` is non-blocking.

If you run the server, you'll see that a request to `127.0.0.1:7878/sleep`
will block any other incoming requests for 5 seconds!
This is because there are no other concurrent tasks that can make progress
while we are `await`ing the result of `handle_connection`.
In the next section, we'll see how to use async code to handle connections concurrently.
