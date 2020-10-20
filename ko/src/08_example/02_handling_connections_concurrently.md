# 연결을 동시에 처리하기

지금까지 우리 코드의 문제는 `listener.incoming()`이 블록하는 반복자라는 점입니다.
executor는 `listener`가 수신 연결을 기다리는 동안 다른 future를 실행할 수 없고,
우리는 이전 연결을 다 처리할 때까지 새로운 연결을 처리할 수 없습니다.

이를 고치기 위해서 블로킹 반복자인 `listener.incoming()`을 논블로킹 스트림으로
전환시킬 것입니다. 스트림은 반복자와 비슷하지만, 비동기적으로 소비될 수
있습니다. 더 많은 정보를 원하시면, [스트림 관련
장](../05_streams/01_chapter.md)을 보세요.

블록하는 `std::net:TcpListener`를 블록하지 않는 `async_std::net::TcpListener`로
바꿔봅시다. 그리고 `async_std::net::TcpStream`을 받을 수 있게
`handle_connection`을 수정합시다. 
```rust,ignore
{{#include ../../examples/08_04_concurrent_tcp_server/src/main.rs:handle_connection}}
```


The asynchronous version of `TcpListener` implements the `Stream` trait for
`listener.incoming()`, a change which provides two benefits. The first is that
`listener.incoming()` no longer blocks the executor. The executor can now yield
to other pending futures while there are no incoming TCP connections to be
processed.

The second benefit is that elements from the Stream can optionally be processed concurrently,
using a Stream's `for_each_concurrent` method.
Here, we'll take advantage of this method to handle each incoming request concurrently.
We'll need to import the `Stream` trait from the `futures` crate, so our Cargo.toml now looks like this:
```diff
+[dependencies]
+futures = "0.3"

 [dependencies.async-std]
 version = "1.6"
 features = ["attributes"]
```

Now, we can handle each connection concurrently by passing `handle_connection` in through a closure function.
The closure function takes ownership of each `TcpStream`, and is run as soon as a new `TcpStream` becomes available.
As long as `handle_connection` does not block, a slow request will no longer prevent other requests from completing.
```rust,ignore
{{#include ../../examples/08_04_concurrent_tcp_server/src/main.rs:main_func}}
```
