# TCP 서버 테스트하기
`handle_connection` 함수를 테스트해 봅시다.

먼저, 테스트에 사용될 `TcpStream`이 필요합니다. 단대단이나 통합 테스트에서는
코드 테스트를 위해 실제 TCP 연결이 필요할 수도 있습니다. 실제 TCP 연결을
사용하여 테스트하는 방법 중 하나는 `localhost`의 0번 포트에서 리스닝하는
것입니다. 0번 포트는 유효한 유닉스 포트가 아니지만 테스트 목적으로는 작동합니다.
운영체제가 열린 TCP 포트를 하나 골라 줄 것입니다.

하지만, 아래 예제에서는 연결 핸들러에 대한 유닛 테스트를 작성하여, 각각의 입력에
맞는 올바른 응답이 반환되었는지 확인할 것입니다. 유닛 테스트를 격리되고
결정론적이게 만들기 위해,  `TcpStream`을 의사코드로 대체할 것입니다.

먼저, 테스트하기 쉽게 `handle_connection`의 시그니처(TODO: 사인?)를 바꿀
것입니다. `handle_connection`는 실제로는 `async_std::net::TcpStream`이 아니라
`async_std::io::Read`, `async_std::io::Write`, 그리고 `marker::Unpin`를 구현하는
모든 구조체가 필요합니다. 이 내용을 반영하여 타입 시그니처를 바꾸면 모조품을
테스트 넘겨 줄 수 있게 되니다.

First, we'll change the signature of `handle_connection` to make it easier to test.
`handle_connection` doesn't actually require an `async_std::net::TcpStream`;
it requires any struct that implements `async_std::io::Read`, `async_std::io::Write`, and `marker::Unpin`.
Changing the type signature to reflect this allows us to pass a mock for testing.
```rust,ignore
use std::marker::Unpin;
use async_std::io::{Read, Write};

async fn handle_connection(mut stream: impl Read + Write + Unpin) {
```

Next, let's build a mock `TcpStream` that implements these traits.
First, let's implement the `Read` trait, with one method, `poll_read`.
Our mock `TcpStream` will contain some data that is copied into the read buffer,
and we'll return `Poll::Ready` to signify that the read is complete.
```rust,ignore
{{#include ../../examples/08_05_final_tcp_server/src/main.rs:mock_read}}
```

Our implementation of `Write` is very similar,
although we'll need to write three methods: `poll_write`, `poll_flush`, and `poll_close`.
`poll_write` will copy any input data into the mock `TcpStream`, and return `Poll::Ready` when complete.
No work needs to be done to flush or close the mock `TcpStream`, so `poll_flush` and `poll_close`
can just return `Poll::Ready`.
```rust,ignore
{{#include ../../examples/08_05_final_tcp_server/src/main.rs:mock_write}}
```

Lastly, our mock will need to implement `Unpin`, signifying that its location in memory can safely be moved.
For more information on pinning and the `Unpin` trait, see the [section on pinning](../04_pinning/01_chapter.md).
```rust,ignore
{{#include ../../examples/08_05_final_tcp_server/src/main.rs:unpin}}
```

Now we're ready to test the `handle_connection` function.
After setting up the `MockTcpStream` containing some initial data,
we can run `handle_connection` using the attribute `#[async_std::test]`, similarly to how we used `#[async_std::main]`.
To ensure that `handle_connection` works as intended, we'll check that the correct data
was written to the `MockTcpStream` based on its initial contents.
```rust,ignore
{{#include ../../examples/08_05_final_tcp_server/src/main.rs:test}}
```
