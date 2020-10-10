# `Stream` 트레잇

`Stream` 트레잇은 `Future`와 비슷하지만 작업을 끝내기 전 여러 값을 양보할 수 있다는 점에서
표준 라이브러리의 `Iterator` 트레잇과 비슷합니다.

```rust,ignore
{{#include ../../examples/05_01_streams/src/lib.rs:stream_trait}}
```

`Stream`의 한 가지 예시로 `futures` 크레이트에 있고 채널 타입에 쓰이는 `Receiver`가 있습니다. 
`Sender`가 값을 보낼 때마다 `Some(val)`을 내놓고, `Sender`의 실행을 거부해 끝내지 않은 모든 메시지를 
받을 때 `None`을 반환합니다.

```rust,edition2018,ignore
{{#include ../../examples/05_01_streams/src/lib.rs:channels}}
```
