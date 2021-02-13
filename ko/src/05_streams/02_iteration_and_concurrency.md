# 반복과 동시성

동기적 `Iterator`와 같이 `Stream`에서 값을 반복하고 처리하는 여러 가지 방법이 있습니다.
`map`, `filter`, `fold`와 같은 콤비네이터 방식과 `try_map`, `try_filter`, `try_fold`와 
같이 오류가 생기면 바로 종료하는 방식의 메소드가 있습니다.

슬프게도 `for` 반복문은 `Stream`과 같이 사용할 수 없지만, 명령형 코드를 위해 
`while let`과 `next`/`try_next` 함수를 쓸 수 있습니다.

```rust,edition2018,ignore
{{#include ../../examples/05_02_iteration_and_concurrency/src/lib.rs:nexts}}
```

하지만 우리가 단지 한 시점에 요소 한 개만 처리하고 있다면 잠재적으로 동시성의
기회를 내다버린 것과 다를 바 없습니다. 이 점이 바로 처음부터 비동기 코드를 써야 하는
이유입니다. 한 스트림 안에서 여러 아이템을 동시에 다루기 위해
`for_each_concurrent`와 `try_for_each_concurrent` 메소드를 사용하시기 바랍니다.

```rust,edition2018,ignore
{{#include ../../examples/05_02_iteration_and_concurrency/src/lib.rs:try_for_each_concurrent}}
```
