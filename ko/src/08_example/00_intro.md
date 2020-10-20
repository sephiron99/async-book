# 마지막 프로젝트: 비동기 러스트로 동시성 웹 서버 만들기
이 장에선, 비동기 러스트로 러스트북의 [싱글스레드 기반 웹 서버
만들기](https://rinthel.github.io/rust-lang-book-ko/ch20-01-single-threaded.html)
를 수정하여 여러개의 요청을 동시에 수행할 수 있게 할 것입니다.

## 개요
이 강좌가 끝나면 아래 코드가 만들어 질 것입니다.

`src/main.rs`:
```rust
{{#include ../../examples/08_01_sync_tcp_server/src/main.rs}}
```

`hello.html`:
```html
{{#include ../../examples/08_01_sync_tcp_server/hello.html}}
```

`404.html`:
```html
{{#include ../../examples/08_01_sync_tcp_server/404.html}}
```

`cargo run`으로 서버를 실행시켜서 브라우저에서 `127.0.0.1:7878`에 접속했다면,
페리스의 친근한 인사말을 볼 수 있을 겁니다!
