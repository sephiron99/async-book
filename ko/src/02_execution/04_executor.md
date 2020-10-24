# 응용: executor 구현하기

러스트 `Future`는 지연계산됩니다. 완성시키기 위해 실제로 구동하기 전까지
future는 아무것도 하지 않을 것입니다. future를 완성까지 구동하는 한 가지 방법은
`async` 함수 안에서 future를 `.await`하는 것입니다. 다만, 그렇게 하면 문제가
하나 생깁니다: 누가 최상위 `async` 함수로부터 반환된 future를 실행할 것인가라는
문제입니다. 그리고 그 해답은 `Future` executor입니다.

`Future` executor는 최상위 `Future`의 집합을 받아 `Future`가 진행할 수 있을
때마다 `poll`을 호출해서 완성될 때까지 실행합니다. 일반적으로, executor는
시작하면서 future를 한 번 `poll`합니다. `Future`가 `wake()`를 호출하여 진행할
준비가 되었음을 알릴 때, future는 큐 뒤에 넣어지고, `poll`이 다시 호출됩니다.
이는 `Future`가 완성될 때까지 반복됩니다.

이 장에서 우리는 수많은 최상위 future를 완성될 때까지 동시에 실행할 수 있는
간단한 executor를 만들 것입니다.

이 예제에서 `Waker`를 쉽게 만들 수 있게 도와주는 `ArcWake` 트레잇 때문에
`futures` 크레잇 의존성이 필요합니다.

```toml
[package]
name = "xyz"
version = "0.1.0"
authors = ["XYZ Author"]
edition = "2018"

[dependencies]
futures = "0.3"
```

다음은, `src/main.rs`의 맨 위에 아래와 같이 import합니다.

```rust,ignore
{{#include ../../examples/02_04_executor/src/lib.rs:imports}}
```

실행할 태스크를 채널을 통해 보내면 우리의 executor가 작동할 겁니다. executor는
채널에서 이벤트를 당겨와서 실행합니다. 만약, 어떤 태스크가 조금 더 일 할 준비가
됐다면(즉, 깨워진다면), 그 태스크는 자기가 다시 poll될 수 있게 채널에 자기
스스로를 넣습니다.

이러한 설계 덕분에, executor는 그저 태스크 채널의 수신 단말만 있으면 됩니다.
유저에게는 송신 단말이 주어지므로, 새로은 future를 만들 수 있습니다. 태스크라는
것은 결국 스스로를 다시 스케쥴링할 수 있는 future일 뿐입니다. 따라서, 우리는
태스크들을 송신자(sender)와 짝지운 future의 형태로 저장할 것입니다. 송신자는
태스크가 자기자신을 큐에 넣는데 사용됩니다.

```rust,ignore
{{#include ../../examples/02_04_executor/src/lib.rs:executor_decl}}
```

새 future를 만들기 쉽게 메소드 한 개를 더 spawner에 추가합시다. 이 메소드는
future 타입을 받아서, box로 감싸고, 새 `Arc<Task>`로 만들 것입니다.
새로 만든 `Arch<Task>`는 excutor에게 enqueue될 것입니다.

```rust,ignore
{{#include ../../examples/02_04_executor/src/lib.rs:spawn_fn}}
```

future를 poll하기 위해서는, `Waker`를 생성해야 합니다. [태스크 깨우기 section]에서
설명했듯이, `Waker`는 `wake`가 호출되면 태스크가 다시 poll될 수 있도록
스케쥴링합니다. `Waker`들은 executor에게 정확히 어떤 태스크가 준비되었는지
알려주기 때문에, executor는 진행할 준비가 된 future들만 poll한다는 점을
기억하십시오. 새로운 `Waker`를 만드는 가장 쉬운 방법은 `ArcWake` 트레잇을
구현하고, `waker_ref`나 `.into_waker()` 함수를 이용하여 `Arc<impl ArcWake>`를
`Waker`로 변경하는 것입니다. 우리의 태스크를 위한 `ArcWake`를 구현하여 `Waker`로
변경하고 깨워봅시다.

```rust,ignore
{{#include ../../examples/02_04_executor/src/lib.rs:arcwake_for_task}}
```

`Arc<Task>`로부터 만들어진 `Waker`의 `wake()`를 호출하면 `Arc`의 복사본이 태스크
채널로 송신될 것이다. 그러면 우리의 executor는 그 태스크를 집어 poll해야 한다.
구현해 봅시다.

```rust,ignore
{{#include ../../examples/02_04_executor/src/lib.rs:executor_run}}
```

축하합니다! future executor를 완성하였습니다. 여러분이 만든 executor를
`asycn/.await` 코드나 우리가 아까 만든 `TimeFuture`같은 커스텀 future를
실행하는데도 사용할 수 있습니다.

```rust,edition2018,ignore
{{#include ../../examples/02_04_executor/src/lib.rs:main}}
```

[태스크 깨우기 section]: ./03_wakeups.md
