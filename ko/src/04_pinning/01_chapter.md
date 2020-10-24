# 고정하기(Pinning)

future를 poll하기 위해서는, future가 `Pin<T>`라는 특별한 타입으로 고정되어
있어야 합니다. 이전 ["`Future`와 태스크 실행하기"] 장의 [`Future` 트레잇]을
읽었다면, `Future::poll` 메소드의 정의에서 `self Pin<&mut Self>`에 쓰였던
`Pin`을 보았을 것입니다. 그렇다면 `Pin`은 무슨 의미이고, 왜 필요할까요?

## 왜 고정해야 하나요

`Pin`은 `Unpin` 마커와 쌍으로 작동합니다. 고정하기는 `!Unpin`을 구현하는 객체가
절대 움직이지 않음을 보장하여 줍니다. 이게 왜 필요한지 이해하려면, `async` /
`.await`가 작동하는 방식을 떠올려 보세요. 아래 코드를 살펴봅시다.

```rust,edition2018,ignore
let fut_one = /* ... */;
let fut_two = /* ... */;
async move {
    fut_one.await;
    fut_two.await;
}
```

보이지는 않지만, 위 코드는 `Future`를 구현하는 익명 타입을 만들어, 아래와 같은
`poll` 메소드를 제공합니다.

```rust,ignore
// 위 `async { ... }` 블록이 생성한 `Future` 타입
struct AsyncFuture {
    fut_one: FutOne,
    fut_two: FutTwo,
    state: State,
}

// 위 `async`블록이 될 수 있는 상태의 종류
enum State {
    AwaitingFutOne,
    AwaitingFutTwo,
    Done,
}

impl Future for AsyncFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        loop {
            match self.state {
                State::AwaitingFutOne => match self.fut_one.poll(..) {
                    Poll::Ready(()) => self.state = State::AwaitingFutTwo,
                    Poll::Pending => return Poll::Pending,
                }
                State::AwaitingFutTwo => match self.fut_two.poll(..) {
                    Poll::Ready(()) => self.state = State::Done,
                    Poll::Pending => return Poll::Pending,
                }
                State::Done => return Poll::Ready(()),
            }
        }
    }
}
```

`poll`이 처음 호출되면 `poll`은 `fut_one`을 poll할 것입니다. 만약 `fut_one`이
완성될 수 없다면, `AsyncFuture::poll`은 `Poll::Pending`을 반환할 것입니다.
다시 future의 `poll`을 호출하면 이전 future가 중단된 지점부터 다시 시작할 것입니다.
이 과정은 future가 성공적으로 완성될 때까지 반복될 것입니다.

하지만, `async` 블록이 참조를 사용한다면 어떻게 될까요?

예를 들어:

```rust,edition2018,ignore
async {
    let mut x = [0; 128];
    let read_into_buf_fut = read_into_buf(&mut x);
    read_into_buf_fut.await;
    println!("{:?}", x);
}
```

위 코드는 어떤 구조체로 변환될까요?

```rust,ignore
struct ReadIntoBuf<'a> {
    buf: &'a mut [u8], // 아래 `x`를 가리킴
}

struct AsyncFuture {
    x: [u8; 128],
    read_into_buf_fut: ReadIntoBuf<'what_lifetime?>,
}
```

여기 `ReadIntoBuf` future는 우리 구조체의 다른 필드인 `x`를 가리키는 참조를
가지고 있습니다. 따라서, `AsyncFuture`가 옮겨진다면, `x`의 위치도 같이 움직이면서 
`read_into_buf_fut.buf`에 저장된 포인터도 무효화 될 것입니다.

future를 특정된 메모리 위치에 고정함으로서 이 문제를 방지하고, `async` 블록 안에
있는 값에 대한 참조를 안전하게 만들 수 있습니다.

## 고정하기에 대한 상세설명

조금 더 간단한 예제로 고정하기를 이해해 봅시다. 위의 문제의 핵심은 '러스트에서
자기참조 타입의 참조를 어떻게 다루는가'입니다.

지금부터 우리의 예제는 다음과 같이 바뀔 겁니다.

```rust, ignore
use std::pin::Pin;

#[derive(Debug)]
struct Test {
    a: String,
    b: *const String,
}

impl Test {
    fn new(txt: &str) -> Self {
        Test {
            a: String::from(txt),
            b: std::ptr::null(),
        }
    }

    fn init(&mut self) {
        let self_ref: *const String = &self.a;
        self.b = self_ref;
    }

    fn a(&self) -> &str {
        &self.a
    }

    fn b(&self) -> &String {
        unsafe {&*(self.b)}
    }
}
```

`Test`는 `a`와 `b` 필드의 값에 대한 참조를 얻는 메소드를 제공합니다. `b`는 `a`에
대한 참조이기 때문에 `b`에 포인터를 사용합니다. 왜냐하면, 러스트의 빌림규칙에
따라 이 라이프타임을 정의할 수 없기 때문입니다. 이 구조체가 바로 자기-참조
구조체라고 불리는 것입니다.

아래 예제를 실행하면 알 수 있듯이, 어느 데이타도 여기저기 움직이지 않는다면 위
예제는 잘 작동할 겁니다.


```rust
fn main() {
    let mut test1 = Test::new("test1");
    test1.init();
    let mut test2 = Test::new("test2");
    test2.init();

    println!("a: {}, b: {}", test1.a(), test1.b());
    println!("a: {}, b: {}", test2.a(), test2.b());

}
# use std::pin::Pin;
# #[derive(Debug)]
# struct Test {
#     a: String,
#     b: *const String,
# }
#
# impl Test {
#     fn new(txt: &str) -> Self {
#         Test {
#             a: String::from(txt),
#             b: std::ptr::null(),
#         }
#     }
#
#     // 자기-참조를 실제로 설정할 `init` 메소드
#     fn init(&mut self) {
#         let self_ref: *const String = &self.a;
#         self.b = self_ref;
#     }
#
#     fn a(&self) -> &str {
#         &self.a
#     }
#
#     fn b(&self) -> &String {
#         unsafe {&*(self.b)}
#     }
# }
```
예상한 대로 출력됩니다.

```rust, ignore
a: test1, b: test1
a: test2, b: test2
```

그럼 `test1`과 `test2`를 스왑하여 데이터를 움직여보고, 무슨 일이 생기는 지 봅시다.

```rust
fn main() {
    let mut test1 = Test::new("test1");
    test1.init();
    let mut test2 = Test::new("test2");
    test2.init();

    println!("a: {}, b: {}", test1.a(), test1.b());
    std::mem::swap(&mut test1, &mut test2);
    println!("a: {}, b: {}", test2.a(), test2.b());

}
# use std::pin::Pin;
# #[derive(Debug)]
# struct Test {
#     a: String,
#     b: *const String,
# }
#
# impl Test {
#     fn new(txt: &str) -> Self {
#         Test {
#             a: String::from(txt),
#             b: std::ptr::null(),
#         }
#     }
#
#     fn init(&mut self) {
#         let self_ref: *const String = &self.a;
#         self.b = self_ref;
#     }
#
#     fn a(&self) -> &str {
#         &self.a
#     }
#
#     fn b(&self) -> &String {
#         unsafe {&*(self.b)}
#     }
# }
```

단순하게 생각하면, 아래처럼 두 번 다 `test1`의 디버그 내용이 출력될 것이라
생각하기 십상입니다:

```rust, ignore
a: test1, b: test1
a: test1, b: test1
```

하지만 실제 출력은 다음과 같습니다:

```rust, ignore
a: test1, b: test1
a: test1, b: test2
```

스왑 이후에도, `test2.b`에 대한 포인터는 여전히 지금 `test1` 내부에 있는 옛
위치를 가리킵니다.(TODO: 의역으로 재번역) 이 구조체는 더 이상 자기-참조적이지
않으며, 다른 객체 안에 있는 필드를 가리키는 포인터를 가지게 됩니다. 즉,
`test2`의 라이프타임에 매여있는 `test2.b`의 라이프타임을 더이상 신뢰할 수 없다는
뜻입니다.

만약 아직도 이해가 되지 않는다면, 아래 코드가 확실히 이해시켜 줄 것입니다.

```rust
fn main() {
    let mut test1 = Test::new("test1");
    test1.init();
    let mut test2 = Test::new("test2");
    test2.init();

    println!("a: {}, b: {}", test1.a(), test1.b());
    std::mem::swap(&mut test1, &mut test2);
    test1.a = "I've totally changed now!".to_string();
    println!("a: {}, b: {}", test2.a(), test2.b());

}
# use std::pin::Pin;
# #[derive(Debug)]
# struct Test {
#     a: String,
#     b: *const String,
# }
#
# impl Test {
#     fn new(txt: &str) -> Self {
#         Test {
#             a: String::from(txt),
#             b: std::ptr::null(),
#         }
#     }
#
#     fn init(&mut self) {
#         let self_ref: *const String = &self.a;
#         self.b = self_ref;
#     }
#
#     fn a(&self) -> &str {
#         &self.a
#     }
#
#     fn b(&self) -> &String {
#         unsafe {&*(self.b)}
#     }
# }
```

아래 그림은 이 내용들을 도식화합니다.

**Fig 1: 스왑 전 후**
![swap_problem](../assets/swap_problem.jpg)

다른 특별한 방법도 있겠지만, 그림으로 정의되지 않은 동작과 실패를 표현하면
이해하기 쉽습니다.

## 실전에서 고정하기

고정하기와 `Pin` 타입으로 이 문제를 해결하는지 알아봅시다.

`Pin` 타입은 포인터를 감싸서 포인터 뒤에 있는 값이 움직이지 않음을 보장해줍니다.
예를 들어, `Pin<&mut T>`, `Pin<&T>`, `Pin<Box<T>>`들은 모두 `T: !Unpin`이라면
`T`가 움직이지 않음을 보장합니다.

대부분의 타입들에게는 이동되는 문제점이 없습니다. 이러한 타입들은 `Unpin`이라는
트레잇을 구현합니다. `Unpin` 타입에 대한 포인터들은 자유롭게 `Pin` 안에 넣거나
꺼낼 수 있습니다. 예를 들어, `u8`은 `Unpin`이어서 `Pin<&mut u8>`은 그냥 평범한
`&mut u8`처럼 작동합니다.

하지만, 고정된 다음에는 움직일 수 없는 타입들은 `!Unpin`이라는 마커를 가지고
있습니다. async/await에 의해 만들어진 future들이 그 예시입니다.

### 스택에 고정하기

다시 예제로 돌아가서, `Pin`을 이용하여 문제를 해결할 수 있습니다. 고정된
포인터를 사용하면 우리의 예제가 어떻게 바뀌는지 살펴봅시다.(TODO: 검토)

```rust, ignore
use std::pin::Pin;
use std::marker::PhantomPinned;

#[derive(Debug)]
struct Test {
    a: String,
    b: *const String,
    _marker: PhantomPinned,
}


impl Test {
    fn new(txt: &str) -> Self {
        Test {
            a: String::from(txt),
            b: std::ptr::null(),
            _marker: PhantomPinned, // This makes our type `!Unpin`
        }
    }
    fn init<'a>(self: Pin<&'a mut Self>) {
        let self_ptr: *const String = &self.a;
        let this = unsafe { self.get_unchecked_mut() };
        this.b = self_ptr;
    }

    fn a<'a>(self: Pin<&'a Self>) -> &'a str {
        &self.get_ref().a
    }

    fn b<'a>(self: Pin<&'a Self>) -> &'a String {
        unsafe { &*(self.b) }
    }
}
```

우리의 타입이 `!Unpin`을 구현한다면 객체를 스택에 고정하는 것은 항상 `unsafe`할
것입니다. 여러분은 스택에 고정할 때 `unsafe` 코드를 직접 사용하지 않으려면
[`pin_utils`][pin_utils] 같은 크레잇을 사용할 수 있습니다.

아래처럼, 객체 `test1`과 `test2`를 스택에 고정합시다.

```rust
pub fn main() {
    // test1은 초기화되기 전에 움직여도 안전합니다.
    let mut test1 = Test::new("test1");
    // `test1`이 다시 액세스되는 것을 막기 위해 어떻게 `test1`을 쉐도우하는지 확인해 두세요
    let mut test1 = unsafe { Pin::new_unchecked(&mut test1) };
    Test::init(test1.as_mut());

    let mut test2 = Test::new("test2");
    let mut test2 = unsafe { Pin::new_unchecked(&mut test2) };
    Test::init(test2.as_mut());

    println!("a: {}, b: {}", Test::a(test1.as_ref()), Test::b(test1.as_ref()));
    println!("a: {}, b: {}", Test::a(test2.as_ref()), Test::b(test2.as_ref()));
}
# use std::pin::Pin;
# use std::marker::PhantomPinned;
#
# #[derive(Debug)]
# struct Test {
#     a: String,
#     b: *const String,
#     _marker: PhantomPinned,
# }
#
#
# impl Test {
#     fn new(txt: &str) -> Self {
#         Test {
#             a: String::from(txt),
#             b: std::ptr::null(),
#             // 우리의 타입을 `!Unpin`으로 만듭니다.
#             _marker: PhantomPinned,
#         }
#     }
#     fn init<'a>(self: Pin<&'a mut Self>) {
#         let self_ptr: *const String = &self.a;
#         let this = unsafe { self.get_unchecked_mut() };
#         this.b = self_ptr;
#     }
#
#     fn a<'a>(self: Pin<&'a Self>) -> &'a str {
#         &self.get_ref().a
#     }
#
#     fn b<'a>(self: Pin<&'a Self>) -> &'a String {
#         unsafe { &*(self.b) }
#     }
# }
```

자, 만약 지금 우리가 데이터를 움직이려고 하면, 컴파일 에러가 발생합니다.

```rust, compile_fail
pub fn main() {
    let mut test1 = Test::new("test1");
    let mut test1 = unsafe { Pin::new_unchecked(&mut test1) };
    Test::init(test1.as_mut());

    let mut test2 = Test::new("test2");
    let mut test2 = unsafe { Pin::new_unchecked(&mut test2) };
    Test::init(test2.as_mut());

    println!("a: {}, b: {}", Test::a(test1.as_ref()), Test::b(test1.as_ref()));
    std::mem::swap(test1.get_mut(), test2.get_mut());
    println!("a: {}, b: {}", Test::a(test2.as_ref()), Test::b(test2.as_ref()));
}
# use std::pin::Pin;
# use std::marker::PhantomPinned;
#
# #[derive(Debug)]
# struct Test {
#     a: String,
#     b: *const String,
#     _marker: PhantomPinned,
# }
#
#
# impl Test {
#     fn new(txt: &str) -> Self {
#         Test {
#             a: String::from(txt),
#             b: std::ptr::null(),
#             _marker: PhantomPinned, // 우리의 타입을 `!Unpin`으로 만듭니다.
#         }
#     }
#     fn init<'a>(self: Pin<&'a mut Self>) {
#         let self_ptr: *const String = &self.a;
#         let this = unsafe { self.get_unchecked_mut() };
#         this.b = self_ptr;
#     }
#
#     fn a<'a>(self: Pin<&'a Self>) -> &'a str {
#         &self.get_ref().a
#     }
#
#     fn b<'a>(self: Pin<&'a Self>) -> &'a String {
#         unsafe { &*(self.b) }
#     }
# }
```

타입 시스템은 우리가 데이터를 움직이지 못하게 막아줍니다.

> 스택에 고정하기는 `unsafe`를 사용하므로 항상 여러분이 보장해야 한다는 점을
> 명심하세요. `&'a mut T`가 _가리키는 값_은 `'a` 라이프타임에 동안 고정된지만,
> `&'a mut T`가 가리키는 데이터가 `'a`가 끝난 다음에도 움직이지 않았는지 알 수는
> 없습니다.(TODO: 재번역) 만약 `&'a mut T`가 가리키는 데이터가 `'a`가 끝난
> 다음에 움직인다면 Pin 규칙을 어기게 될 것입니다.

> 원 변수를 쉐도우하는 것을 깜빡하기 쉽습니다. 왜냐하면, (Pin 규칙을 어기는)
> 아래 코드처럼, `Pin`을 드랍하고 나서, `&'a mut T` 다음에 데이타를 움직일
> 가능성이 있기 때문입니다.
>
> ```rust
> fn main() {
>    let mut test1 = Test::new("test1");
>    let mut test1_pin = unsafe { Pin::new_unchecked(&mut test1) };
>    Test::init(test1_pin.as_mut());
>    drop(test1_pin);
>    println!(r#"test1.b points to "test1": {:?}..."#, test1.b);
>    let mut test2 = Test::new("test2");
>    mem::swap(&mut test1, &mut test2);
>    println!("... and now it points nowhere: {:?}", test1.b);
> }
> # use std::pin::Pin;
> # use std::marker::PhantomPinned;
> # use std::mem;
> #
> # #[derive(Debug)]
> # struct Test {
> #     a: String,
> #     b: *const String,
> #     _marker: PhantomPinned,
> # }
> #
> #
> # impl Test {
> #     fn new(txt: &str) -> Self {
> #         Test {
> #             a: String::from(txt),
> #             b: std::ptr::null(),
> #             // This makes our type `!Unpin`
> #             _marker: PhantomPinned,
> #         }
> #     }
> #     fn init<'a>(self: Pin<&'a mut Self>) {
> #         let self_ptr: *const String = &self.a;
> #         let this = unsafe { self.get_unchecked_mut() };
> #         this.b = self_ptr;
> #     }
> #
> #     fn a<'a>(self: Pin<&'a Self>) -> &'a str {
> #         &self.get_ref().a
> #     }
> #
> #     fn b<'a>(self: Pin<&'a Self>) -> &'a String {
> #         unsafe { &*(self.b) }
> #     }
> # }
> ```

### 힙 역역에 고정하기

`!Unpin`타입을 힙에 고정하면 우리 데이타에 안정적인 주소를 부여하게 됩니다.
그래서 우리가 가리키는 데이터는 고정되고 나면 움직일 수 없습니다. 스택에
고정하기와 대조적으로, 데이터가 객체의 수명주기동안 고정됩니다.

```rust, edition2018
use std::pin::Pin;
use std::marker::PhantomPinned;

#[derive(Debug)]
struct Test {
   a: String,
    b: *const String,
    _marker: PhantomPinned,
}

impl Test {
    fn new(txt: &str) -> Pin<Box<Self>> {
        let t = Test {
            a: String::from(txt),
            b: std::ptr::null(),
            _marker: PhantomPinned,
        };
        let mut boxed = Box::pin(t);
        let self_ptr: *const String = &boxed.as_ref().a;
        unsafe { boxed.as_mut().get_unchecked_mut().b = self_ptr };

        boxed
    }

    fn a<'a>(self: Pin<&'a Self>) -> &'a str {
        &self.get_ref().a
    }

    fn b<'a>(self: Pin<&'a Self>) -> &'a String {
        unsafe { &*(self.b) }
    }
}

pub fn main() {
    let mut test1 = Test::new("test1");
    let mut test2 = Test::new("test2");

    println!("a: {}, b: {}",test1.as_ref().a(), test1.as_ref().b());
    println!("a: {}, b: {}",test2.as_ref().a(), test2.as_ref().b());
}
```

몇몇 함수들은 future가 `Unpin` 타입일 것을 요구합니다. `Unpin`이 아닌 `Future`나
`Stream`을 `Unpin` 타입을 요구하는 함수와 함께 사용하기 위해서는, 먼저
(`Pin<Box<T>>`을 만든다면) `Box::pin`이나 (`Pin<&mut T>`를 만든다면)
`pin_utils::pin_mut!` 매크로를 사용하여 값을 고정해야 합니다. `Pin<Box<Fut>>`와
`Pin<&mut Fut>` 둘 다 future처럼 사용될 수 있으며, 둘 다 `Unpin`을 구현합니다.

예를 들어:

```rust,edition2018,ignore
use pin_utils::pin_mut; // `pin_utils`는 crates.io에 있는 가벼운 crate입니다.

// `Unpin`을 구현하는 `Future`를 취하는 함수
fn execute_unpin_future(x: impl Future<Output = ()> + Unpin) { /* ... */ }

let fut = async { /* ... */ };
execute_unpin_future(fut); // 오류: `fut`은 `Unpin` 트레잇을 구현하지 않음

// Pinning with `Box`:
let fut = async { /* ... */ };
let fut = Box::pin(fut);
execute_unpin_future(fut); // OK

// Pinning with `pin_mut!`:
let fut = async { /* ... */ };
pin_mut!(fut);
execute_unpin_future(fut); // OK
```

## 정리

1. `T: Unpin`(기본값)이라면 `Pin<'a, T>`는 `&'a mut T`와 전적으로 동일합니다.
   다르게 표현하자면, `Unpin`은 "이 타입은 고정되었을지라도 움직여도 됨"을
   의미합니다. 따라서 `Pin`은 해당 타입에 대해 효과가 없습니다.

2. `T: !Unpin`일 때, 고정된 T에 대하여 `&mut T`를 얻으려면 unsafe가 필요합니다.

3. 대부분의 표준 라이브러리 타입들은 `Unpin`을 구현합니다. 여러분이 러스트에서
   사용할 대부분의 "평범한" 타입들도 마찬가지입니다. async/await에 의해 생성된
   `Future`는 이 규칙에 예외입니다.

4. nightly에서는 feature flag를 설정하면 어떤 타입을 `!Unpin`할 수
   있습니다(TODO: add bound를 다시 번역). stable에서는 타입에
   `std::marker::PhantomPinned`를 추가하면 됩니다.

5. 데이타를 스택이나 힙에 고정할 수 있습니다.

6. `!Unpin` 객체를 스택에 고정하려면 `unsafe`가 필요합니다.

7. `!Unpin` 객체를 힙에 고정할 때는 `unsafe`가 필요 없습니다. `Box::pin`을
   사용하면 간단하게 할 수 있습니다.

8. `T: !Unpin`인 고정된 데이터에 대해서는, 여러분이 그 데이터의 메모리가
_고정되어 drop이 호출되기 전_까지 무효화되거나 용도변경되지 않음(불변성)을
유지할 책임이 있습니다. 이는 _고정 규칙_에서 중요한 부분입니다.

["`Future`와 태스크 실행하기"]: ../02_execution/01_chapter.md
[`Future` 트레잇]: ../02_execution/02_future.md
[pin_utils]: https://docs.rs/pin-utils/
