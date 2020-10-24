# `Future` 트레잇
`Future` 트레잇은 러스트 비동기 프로그래밍의 핵심입니다. `Future`는 비동기
연산의 일종으로, 한 개의 값을 산출할 수 있습니다(그 값이 `()`같은 빈
값일지라도요). *단순화된* 버전의 future 트레잇은 다음과 같은 형태라고 할 수
있습니다.

```rust
{{#include ../../examples/02_02_future_trait/src/lib.rs:simple_future}}
```

Future는 `poll` 함수를 호출하면 진행됩니다. future가 완성될 때까지, `poll` 함수는 
그때그때 가능한 만큼 future를 구동시킬 것입니다. 만약 Future가 완성된다면, Future는
`Poll::Ready(result)`를 반환합니다. Future가 아직 완성될 수 없다면, Future는
`Poll::Pending`을 반환하고, `Future`가 좀 더 진행될 때를 대비하여 `wake()`함수가
호출될 수 있게 준비합니다. `wake()` 함수가 호출되었을 때, 해당 `Future`를
구동(drive)하는 executor는 `poll`을 다시 호출하여 `Future`가 더 진행될 수 있게
합니다.

`wake()`가 없다면, executor는 어떤 future가 진행할 준비가 되었는지를 알 방법이
없어서, 아마 끊임없이 모든 future를 폴링(polling)해야만 할 것입니다. `wake()`
덕분에, executor는 어떤 future가 `poll` 될 수 있는지 정확히 알 수 있습니다.

예를 들어, 데이터를 제공할 준비가 됐는지 알 수 없는 소켓에서 데이터를 읽어야
하는 사례를 생각해봅시다. 만약 데이터가 있다면, 우리는 데이터를 읽어들여서
`Poll::Ready(data)`를 반환하면 됩니다. 하지만, 데이터가 준비되지 않았다면,
future는 블록될 것이고, 더 이상 진행할 수 없을 것입니다. 데이터가 준비되지
않았을 때에는, `wake`를 등록하여 소켓에 데이터가 준비되었을 때 `wake`가 호출될
수 있도록 합니다. 이렇게 등록하면 executor에게 우리의 future가 진행될 준비가
되었음을 알릴 수 있습니다. 간단한 `SocketRead` future는 다음과 같은 형태라고 할
수 있습니다.

```rust,ignore
{{#include ../../examples/02_02_future_trait/src/lib.rs:socket_read}}
```

여러 `Future`에 대한 아래와 같은 구조는 여러 개의 비동기 동작들을 중간 할당 없이
한꺼번에 조율할 수 있게 해줍니다. 여러개의 future를 한 번에 실행하거나
연쇄실행하는 방식은 다음과 같이 할당 없는 상태기계로 구현될 수 있습니다.

```rust,ignore
{{#include ../../examples/02_02_future_trait/src/lib.rs:join}}
```

위 예제는 여러개의 future가 각각에 대한 할당 없이도 어떻게 동시에 실행 될 수
있는지 보여줍니다. 이는 보다 효율적인 비동기 프로그램입니다. 마찬가지로, 순서가
있는 여러개의 future는 아래와 같이 한 개 한 개 씩 실행될 수 있습니다.

```rust,ignore
{{#include ../../examples/02_02_future_trait/src/lib.rs:and_then}}
```

위의 예제들은 `Future` 트레잇이 여러개의 할당된 객체나 반복중첩된(deeply nested)
콜백 없이 비동기 흐름 제어를 구현하는 방법을 보여줍니다. 기본적인 흐름제어에
대한 설명은 이쯤에서 마치고, 진짜 `Future` 트레잇은 실제로 어떻게 생겼고, 무엇이 다른지
살펴봅시다.

```rust,ignore
{{#include ../../examples/02_02_future_trait/src/lib.rs:real_future}}
```

여러분이 확인하게 된 첫 번째 변화는 `self` 타입이 더 이상 `&mut Self`가 아니고,
`Pin<&mut Self>`로 바뀌었다는 점입니다. [다른 장][pinning]에서 고정하기에
대해 더 다루겠지만, 지금은 이동불가한 future를 만들 수 있게 해준다는 점만 알아
두십시오. 이동불가한 객체는 `struct MyFut { a: i32, ptr_to_a: *const i32 }` 처럼
필드에 포인터를 저장할 수 있습니다. 고정하기는 async와 await를 활성화하기
위해 필요합니다.

두 번째로, `wake: fn()`은 `&mut Context<'_>`으로 바뀌었습니다.
`SimpleFuture`에서는 future executor에게 진행할 준비가 된 것으로 보이는 future가
poll되어야 한다고 알려주기 위해 함수포인터(`fn()`)에 대한 호출을
사용하였습니다. 하지만, `fn()`은 단지 함수포인터일 뿐, *어떤* `Future`가
`wake`를 호출했는지에 대한 정보를 저장할 수 없습니다.

현실적인 시나리오에서, 웹 서버 같은 복잡한 어플리케이션에는 수 천개의 커넥션이
있을 것이고, 각 연결에 대한 wakeup은 모두 개별적으로 관리되어야 할 것입니다.
`Waker` 타입의 값에 대한 접근을 제공하는 `Context` 타입을 이용하여 이를 해결하는
데, 이 `Context`로 특정한 태스크를 깨울 수 있습니다. 


[pinning]: ../04_pinning/01_chapter.md
