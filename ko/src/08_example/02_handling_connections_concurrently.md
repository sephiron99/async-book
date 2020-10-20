# 연결을 동시에 처리하기

지금까지 우리 코드의 문제는 `listener.incoming()`이 블록하는 반복자라는 점입니다.
executor는 `listener`가 수신 연결을 기다리는 동안 다른 future를 실행할 수 없고,
우리는 이전 연결을 다 처리할 때까지 새로운 연결을 처리할 수 없습니다.

이를 고치기 위해서 블록하는 반복자인 `listener.incoming()`을 블록하지 않는 Stream으로
전환시킬 것입니다. Stream은 반복자와 비슷하지만, 비동기적으로 소비될 수
있습니다. 더 많은 정보를 원하시면, [Stream 관련
장](../05_streams/01_chapter.md)을 보세요.

블록하는 `std::net:TcpListener`를 블록하지 않는 `async_std::net::TcpListener`로
바꿔봅시다. 그리고 `async_std::net::TcpStream`을 받을 수 있게
`handle_connection`을 수정합시다. 
```rust,ignore
{{#include ../../examples/08_04_concurrent_tcp_server/src/main.rs:handle_connection}}
```

`TcpListener`의 비동기 버전은 `listener.incoming()`에 대한 `Stream`을
구현합니다. 이는 두 가지 이득을 가져다 주는데요, 첫 째는 `listener.incoming()`이
더 이상 executr를 블록하지 않는다는 점입니다. 이렇게 되면 이제 executor는
처리해야할 수신된 TCP 연결이 없으면 계류중인 future에게 스레드를 양보할 수
있습니다.

두번 째 이득은 Stream에서 가져온 요소들을 Stream의 `for_each_concurrent` 메소드를
사용하여 선택적으로 동시에 처리할 수 있다는 점입니다.
아래에서는 각 수신된 요구를 동시에 처리하기 위해 이 메소드를 활용할 것입니다.
`futures` 크레잇의 `Stream` 트레잇을 import할 필요가 있습니다. 그러면
Cargo.toml은 이제 아래와 같이 될 것입니다.

```diff
+[dependencies]
+futures = "0.3"

 [dependencies.async-std]
 version = "1.6"
 features = ["attributes"]
```

이제 `handle_connection`을 클로저 함수 안으로 넣어서 각 연결을 동시에 처리할 수
있습니다. 클로저 함수는 각 `TcpStream`의 소유권을 획득하고, 새 `TcpStream`이
준비되자마자 실행됩니다.
`handle_connection`이 블록하지 않는 한, 더 이상 느린 요청은 다른 요청이 완성되지 못하게
방해하지 않을 것입니다.
```rust,ignore
{{#include ../../examples/08_04_concurrent_tcp_server/src/main.rs:main_func}}
```
