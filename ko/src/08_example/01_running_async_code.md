# 비동기 코드 실행하기

HTTP 서버는 동시에 여러 클라이언트에 동시에 서비스할 수 있어야 합니다. 즉, HTTP
서버는 현재의 요청을 처리하기 전에 기존의 요청이 끝나길 기다려서는 안된다는
말입니다. 러스트북의 예제에서는 모든 연결에 스레드를 하나씩 할당하는 스레드 풀을
만들어서 [이 문제를
해결합니다.](https://rinthel.github.io/rust-lang-book-ko/ch20-02-multithreaded.html#%EC%84%9C%EB%B2%84%EB%A5%BC-%EC%8B%B1%EA%B8%80-%EC%8A%A4%EB%A0%88%EB%93%9C%EC%97%90%EC%84%9C-%EB%A9%80%ED%8B%B0-%EC%8A%A4%EB%A0%88%EB%93%9C%EB%A1%9C-%EB%B0%94%EA%BE%B8%EA%B8%B0)

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

비동기 코드 그 자체 안에서 `await`하거나 `poll`할 수는 없습니다. future를
완성될때까지 스케쥴링하고 실행할 비동기 런타임이 필요합니다.
비동기 런타임, executor 그리고 reactor에 대한 자세한 정보를 원한다면 런타임
선택에 관한 장을 살펴보세요.

[//]: <> (TODO: Link to section on runtimes once complete.)

## Async 런타임 추가
여기서는 `async-std` 크레잇의 executor를 사용할 것입니다.
`async-std`의 `#[async_std::main]` 속성을 붙이면 비동기 main 함수를 작성할 수 있습니다.
`#[async_std::main]`을 사용하기 위해서 `async-std`의 `attributes` 기능을 `Cargo.toml`에서
활성화 하세요.
```toml
[dependencies.async-std]
version = "1.6"
features = ["attributes"]
```

첫 번째 단계로, main 함수를 비동기로 전환하고, 비동기 `handle_connection`이
반환한 future를 `await`할 것입니다. 그리고 나서, 서버가 어떻게 작동하는 지
테스트할 것입니다. 이렇게 작성한 코드는 아래와 같습니다.
```rust {{#include ../../examples/08_02_async_tcp_server/src/main.rs:main_func}}``` 
이제 서버가 연결을 동시에 처리할 수 있는 지 테스트해 봅시다. 단순히 `handle_connection`을
비동기로 만들었다고 해서 바로 서버가 여러개의 연결을 동시에 처리할 수 있게
되지는 않습니다. 곧 그 이유를 곧 알게 될 것입니다.

이를 설명하기 위해, 느린 요청 하나로 모의실험해 봅시다.
클라이언트가 `127.0.0.1:7878/sleep`으로 요청을 보냈을 때, 우리 서버는 5초간 잠들 것입니다.

```rust,ignore
{{#include ../../examples/08_03_slow_request/src/main.rs:handle_connection}}
```
이는 러스트북의 [현재 서버에서 느린 요청을
시뮬레이팅하기](https://rinthel.github.io/rust-lang-book-ko/ch20-02-multithreaded.html#%ED%98%84%EC%9E%AC-%EC%84%9C%EB%B2%84%EC%97%90%EC%84%9C-%EB%8A%90%EB%A6%B0-%EC%9A%94%EC%B2%AD%EC%9D%84-%EC%8B%9C%EB%AE%AC%EB%A0%88%EC%9D%B4%ED%8C%85%ED%95%98%EA%B8%B0)와
매우 유사합니다.

우리는 블로킹 함수인 `std::thread::sleep`대신에 논블로킹 함수인
`async_std::task::sleep`를 사용하고 있습니다. 코드 한 줄이라도 `async fn` 안에서
실행되고, `await`된다면 그 코드는 여전히 스레드를 블록할 수도 있음을 명심하세요.
우리 서버가 연결을 동시에 처리할 수 있는 지 테스트하려면, `handle_connection`이
논블로킹임을 확인해야 합니다.

서버를 실행하면, `127.0.0.1:7878/sleep`에 대한 한 개의 요청이 수신되는 다른
요청들을 5초간 블록하는 것을 확인할 수 있습니다! 그 이유는 우리가
`handle_connection`을 `await`하는 동안에 진행될만한 다른 동시성 태스크가 없기
때문입니다. 다음 장에서는 연결을 동시에 처리할 수 있는 비동기 코드를 작성하는
방법에 대해 알아 봅시다.
