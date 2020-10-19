# `select!`

`futures::select` 매크로를 사용하면 여러 future를 동시에 실행하면서, 어떤
future라도 완성되면 사용자가 바로 반응할 수 있습니다.

```rust,edition2018
{{#include ../../examples/06_03_select/src/lib.rs:example}}
```

위의 함수는 `t1`과 `t2` 둘 다 동시에 실행할 것입니다. 둘 중에 하나가 끝나면,
대응하는 핸들러가 `println!`을 호출하고, 위 함수는 나머지 task를 완성하지 않고
바로 종료됩니다.

`select`의 기본 문법은 `<pattern> = <expression> => <code>,`이고, `select`에
넣을 future 개수만큼 반복하면 됩니다.

## `default => ...` 와 `complete => ...`

또한 `select`는 `default`와 `complete` 분기를 지원합니다.

`default` 분기는 `select`에 넣어진 future들 중 아무것도 완성되지 않았으면
실행됩니다. 따라서 `default` 분기가 있는 `select`는 항상 즉시 반환합니다. 다른
어떤 future도 준비되지 않았으면 `defualt`가 실행되기 때문입니다.

`complete` 분기는 `select`에 넣어진 모든 future가 모두 완성되어 더 이상 진행할
일이 없는 경우를 다루기 위해 사용됩니다. `complete` 분기는 `select!`를 반복문
안에 넣을 때 유용합니다.

```rust,edition2018
{{#include ../../examples/06_03_select/src/lib.rs:default_and_complete}}
```

## `Unpin`과 `FusedFuture`로 상호작용하기

위 첫 번째 예제에서, 여러분은 두 `async fn`가 반환한 future에 대해 `pin_mut`으로
고정하고, `.fuse()`를 호출해야 한다는 점을 인지했을 겁니다. `select`안에서
사용된 future들은 `Unpin`, `FusedFuture` 트레잇 둘 다 구현해야 하기 때문에, 이
호출들이 필요합니다.

`select`가 사용하는 future는 값으로 전달되지 않고 가변 참조로 전달되기 때문에,
`Unpin`이 필요합니다. future의 소유권을 취하지 않기 때문에, 미완성된 future는
`select`를 호출한 다음에도 재사용 할 수 있습니다.

비슷하게, `select`는 이미 완성된 future를 poll하면 안되기 때문에, `FusedFuture`
트레잇이 필요합니다. `FusedFuture`는 future에 의해 구현되며, 자신이 완성되었는지
여부를 추적합니다. `FusedFuture`는 아직 완성되지 않은 future만 골라서 폴링할 수
있게 해주기 때문에 `select`를 반복문 안에서 사용할 수 있게 됩니다. 이는 위
예제에서 `a_fut`이나 `b_fut`가 반복문 2회차 때에 완성되는 것을 보면 알 수
있습니다. `future::ready`가 반환한 future가 `FusedFuture`를 구현하기 때문에,
`select`가 그 future를 다시 poll하지 못하게 할 수 있습니다.

스트림은 같은 기능을 하는 `FusedStream` 트레잇을 가지고 있음을 알아두세요.
`FusedStream` 트레잇을 구현하거나, `.fuse()`를 사용하여 래핑한 스트림은
`.next()` / `.try_next()`을 통해 `FusedFuture`를 뱉을 것입니다.

```rust,edition2018
{{#include ../../examples/06_03_select/src/lib.rs:fused_stream}}
```

## `Fuse`와 `FuturesUnordered`를 이용한 `select` 루프 내부에서의 동시성 task

`Fuse:terminated()` 함수는 눈에 잘 띄지는 않지만 유용한 함수입니다. 이 함수는
이미 종료되어 비어있지만, 나중에 필요할 때, 실행할 future를 넣어서 실행할 수 있는
future를 만들어 줍니다.

이 함수는 `select` 루프가 유효한 동안에 실행될 필요가 있지만 `select` 루프 자체
안에서 만들어지는 task가 있을 경우 유용합니다.

`.select_next_some()` 함수의 용도에 유의하세요. 이 함수는 스트림이 반환한
`Some(_)` 값에 대응하는 분기를 실행할 때만 `select`와 함께 사용될 수 있습니다.
`.select_next_some()`함수는 `None`을 무시합니다.(TODO: 이 때, `None`은
무시됩니다)

```rust,edition2018
{{#include ../../examples/06_03_select/src/lib.rs:fuse_terminated}}
```

같은 future의 여러 복사본을 동시에 실행할 필요가 있을 때에는 `FuturesUnordered`
타입을 사용하세요. 아래 예제는 위 예제랑 비슷하지만, `run_on_new_num_fut`의
복사본이 생겨도 중단하지 않고 각 복사본을 완성될때까지 실행한다는 점이 다릅니다.
또한, 아래 예제는 `run_on_new_num_fut`가 반환한 값을 출력할 것입니다.

```rust,edition2018
{{#include ../../examples/06_03_select/src/lib.rs:futures_unordered}}
```
