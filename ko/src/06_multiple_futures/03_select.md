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

## Interaction with `Unpin` and `FusedFuture`
## `Unpin`과 `FusedFuture`로 소통하기

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

## Concurrent tasks in a `select` loop with `Fuse` and `FuturesUnordered`

One somewhat hard-to-discover but handy function is `Fuse::terminated()`,
which allows constructing an empty future which is already terminated,
and can later be filled in with a future that needs to be run.

This can be handy when there's a task that needs to be run during a `select`
loop but which is created inside the `select` loop itself.

Note the use of the `.select_next_some()` function. This can be
used with `select` to only run the branch for `Some(_)` values
returned from the stream, ignoring `None`s.

```rust,edition2018
{{#include ../../examples/06_03_select/src/lib.rs:fuse_terminated}}
```

When many copies of the same future need to be run simultaneously,
use the `FuturesUnordered` type. The following example is similar
to the one above, but will run each copy of `run_on_new_num_fut`
to completion, rather than aborting them when a new one is created.
It will also print out a value returned by `run_on_new_num_fut`.

```rust,edition2018
{{#include ../../examples/06_03_select/src/lib.rs:futures_unordered}}
```
