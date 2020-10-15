# `join!`

`futures::join` 매크로는 여러개의 다른 future를 동시에 실행하여 모두 완성될
때까지 기다리게 해줍니다.

# `join!`

여러개의 비동기 작업을 진행할 때, 단순하게 `.await`를 순차적으로 사용하는 식으로
만들기 쉽습니다.

```rust,edition2018,ignore
{{#include ../../examples/06_02_join/src/lib.rs:naiive}}
```

그런데, 이렇게 하면 필요한 만큼 성능을 낼 수 없습니다. 왜냐하면, `get_book`이
완성될 때까지 `get_music`을 시작하려 하지 않을 것이기 때문입니다. 몇몇 다른
언어에서는, future가 완성될 때까지 주변적으로(ambiently) 실행되는 방식입니다. 이
방식에서는 처음부터 각 `async fn`을 호출하여 future들을 시작하고, 둘 모두를
기다림으로써, 두 작업이 동시에 실행될 수 있습니다.

```rust,edition2018,ignore
{{#include ../../examples/06_02_join/src/lib.rs:other_langs}}
```

하지만, 실제 러스트의 future는 `.await`될 때까지 아무것도 하지 않습니다. 따라서,
위의 두 코드 스니펫들은 둘 다 `book_future`와 `music_future`를 동시가 아닌
순차적으로 실행한다는 의미입니다. 두 future를 진짜 동시에 실행하려면
`futures::join!`을 사용하세요: 

```rust,edition2018,ignore
{{#include ../../examples/06_02_join/src/lib.rs:join}}
```

`join!`이 반환한 값은 각 `Future`가 출력한 값으로 구성된 튜플입니다.

## `try_join!`

`Result`를 반환하는 future들에는 `join!`말고 `try_join!`을 사용하는 게 좋습니다.
`join!`은 모든 하위 future들이 완성되었을 때에만 완성되므로, 하위 future 들 중
하나가 `Err`을 반환하였더라도 나머지 future들을 계속 처리할 것입니다. 

`join!`과 다르게, `try_join!`은 하위 future 중 하나가 에러를 반환하면 즉시 완성될 것입니다.

```rust,edition2018,ignore
{{#include ../../examples/06_02_join/src/lib.rs:try_join}}
```

`try_join!`에 전달된 future들은 모두 같은 타입의 에러를 반환해야 한다는 점을
명심하세요. 에러 타입을 일치시키기 위해 `futures::future::TryFutureExt` 모듈의
`.map_err(|e| ...)`과 `.err_into()` 함수를 사용해 보세요.


```rust,edition2018,ignore
{{#include ../../examples/06_02_join/src/lib.rs:try_join_map_err}}
```
