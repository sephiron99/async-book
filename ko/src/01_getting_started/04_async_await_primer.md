# `async`/`.await` 기초

`async`/`.await`는 동기적 코드처럼 보이는 비동기 함수들을 작성하는 데 쓰이는
러스트 내장 도구입니다. `async`는 코드 블록을 `Future`라는 트레잇을 구현하는
유한상태기계로 변환해줍니다. 동기적 메소드 안에서 블록하는 함수를 호출한다면
전체 스레드가 블록되지만, `Future`는 블록되어도 스레드를 잡아놓지 않아 다른
`Future`가 작동할 수 있습니다.

`Cargo.toml` 파일에 의존성을 추가해 봅시다.

```toml
{{#include ../../examples/01_04_async_await_primer/Cargo.toml:9:10}}
```

비동기 함수를 만들기 위해, `async fn` 문법을 사용합니다.

```rust,edition2018
async fn do_something() { /* ... */ }
```

`async fn`이 반환하는 값은 한 개의 `Future` 객체입니다. 코드가 실제로 동작하게
하려면, executor로 `Future` 객체를 실행해야 합니다.

```rust,edition2018
{{#include ../../examples/01_04_async_await_primer/src/lib.rs:hello_world}}
```

`async fn` 안에서 `.await`을 사용하면 `Future` 트레잇을 구현한 다른 타입이
완성(예시: 다른 `async fn`의 출력)될 때까지 기다릴 수 있습니다. `block on`과
달리, `.await`는 현재의 스레드를 블록하지 않고, 대신에 이 future가 완성될 때까지
비동기적으로 기다립니다. 이렇게 하면 이 future가 현재 진행될 수 없는 상황에서도
다른 태스크들이 실행될 수 있습니다.

예를 들어, 세 개의 `async fn`(`learn_song`, `sing_song` 그리고 `dance`)이  있다고
칩시다.

```rust,ignore
async fn learn_song() -> Song { /* ... */ }
async fn sing_song(song: Song) { /* ... */ }
async fn dance() { /* ... */ }
```

노래를 배우고 부르며, 춤을 추기위한 방법 중에 하나는 각각을 수행할 때마다
블록하는 것입니다.

```rust,ignore
{{#include ../../examples/01_04_async_await_primer/src/lib.rs:block_on_each}}
```

그러나, 이 방법으로는 최선의 성능을 낼 수 없습니다. 오직 한 번에 한 가지만
한다구요! 우리가 노래를 부르기 전에 먼저 노래를 배워야 하는 것은 맞지만, 춤은
노래를 배우거나 부르면서도 출 수 있습니다. 이를 위해, 우리는 동시에 수행될 수
있는 두 개의 다른 `async fn`을 만들면 됩니다.

```rust,ignore
{{#include ../../examples/01_04_async_await_primer/src/lib.rs:block_on_main}}
```

이 예제에서, 노래 배우기는 노래 부르기보다 먼저 동작해야 하지만, 노래 배우기와
부르기는 춤추기와 같은 시간에 동작할 수 있습니다. 만약 `learn_and_sing`안에서
`learn_song().await`말고, `block_on(learn_song())`을 사용했다면, 해당
스레드는 `learn_song`이 동작하는 동안에는 아무것도 할 수 없었을 것이고, 그렇다면
춤추기를 노래와 동시에 수행할 수 없었을 것입니다. 하지만 우리는 `learn_song`
future를 `.await`함으로써, `learn_song`이 블록되었을지라도 다른 태스크들이 현재의
스래드에서 실행되게 할 수 있습니다. 이 방법으로, 여러개의 future를 한 개의
스레드에서 동시에 실행하여 완성할 수 있습니다.
