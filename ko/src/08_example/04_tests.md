# TCP 서버 테스트하기
`handle_connection` 함수를 테스트해 봅시다.

먼저, 테스트에 사용될 `TcpStream`이 필요합니다. 단대단이나 통합 테스트에서는
코드 테스트를 위해 실제 TCP 연결이 필요할 수도 있습니다. 실제 TCP 연결을
사용하여 테스트하는 방법 중 하나는 `localhost`의 0번 포트에서 리스닝하는
것입니다. 0번 포트는 유효한 유닉스 포트가 아니지만 테스트 목적으로는 작동합니다.
운영체제가 열린 TCP 포트를 하나 골라 줄 것입니다.

하지만, 아래 예제에서는 연결 핸들러에 대한 유닛 테스트를 작성하여, 각각의 입력에
맞는 올바른 응답이 반환되었는지 확인할 것입니다. 유닛 테스트를 격리되고
결정론적이게 만들기 위해,  `TcpStream`을 모조품으로 대체할 것입니다.

먼저, 테스트하기 쉽게 `handle_connection`의 시그니처를 바꿀
것입니다. 사실 `handle_connection`는 `async_std::net::TcpStream`이 꼭 필요한
것은 아닙니다. `async_std::io::Read`, `async_std::io::Write`, 그리고 `marker::Unpin`을 구현하는
어떤 구조체도 가능합니다. 이 내용을 반영하여 타입 시그니처를 바꾸면 모조품을
테스트 용으로 넘겨 줄 수 있게 됩니다.

```rust,ignore
use std::marker::Unpin;
use async_std::io::{Read, Write};

async fn handle_connection(mut stream: impl Read + Write + Unpin) {
```

이 트레잇 세 개를 구현하는 `TcpStream` 모조품을 만들어 봅시다.
먼저, `poll_read` 메소드 한 개만 있는 `Read` 트레잇을 구현합시다.
`TcpStream` 모조품은 읽기 버퍼로 복사되는 어떤 데이터를 가지고 있을 것이고,
복사가 끝나면 `poll_read`는 읽기가 끝났음을 알리는 `Poll::Ready`를 반환할 것입니다.

```rust,ignore
{{#include ../../examples/08_05_final_tcp_server/src/main.rs:mock_read}}
```

`poll_write`, `poll_flush`, 그리고 `poll_close`라는 세 개의 메소드를 작성해야
할지라도 `Write` 구현은 매우 간단합니다.
`poll_write`는 모든 입력 데이터를 `TcpStream` 모조품으로 복사하고, 완성되면
`Poll::Ready`를 반환할 것입니다. `TcpStream` 모조품을 플러싱하거나 닫기 위한
별도 작업이 필요 없기 때문에 `poll_flush`와 `poll_close`는 그냥 `Poll::Ready`를
반환하면 됩니다.

```rust,ignore
{{#include ../../examples/08_05_final_tcp_server/src/main.rs:mock_write}}
```

마지막으로, `TcpStream` 모조품은 메모리 상 위치가 안전하게 움직일 수 있다고
알리는 `Unpin`을 구현해야 합니다.
`Unpin`에 대한 자세한 정보는 [고정하기](../04_pinning/01_chapter.md)를 참고하세요.
```rust,ignore
{{#include ../../examples/08_05_final_tcp_server/src/main.rs:unpin}}
```

이제 `handle_connection` 함수를 테스트할 준비가 되었습니다.
 `MockTcpStream`이 임의의 초기 데이터를 가지도록 설정한 다음,
`#[async_std::main]`의 사용과 유사하게 `#[async_std::test]` 속성을 이용하여
`handle_connection`을 실행할 수 있습니다.

`handle_connection`이 잘 작동함을 확인하기 위해 데이터의 처음 부분을 비교하여 데이터가
`MockTcpStream`에 제대로 쓰여졌는 지 확인할 것입니다.
```rust,ignore
{{#include ../../examples/08_05_final_tcp_server/src/main.rs:test}}
```
