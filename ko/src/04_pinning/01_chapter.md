# 고정하기(Pinning)

future를 poll하기 위해서는, future가 `Pin<T>`라는 특별한 타입으로 고정되어
있어야 합니다. 이전 ["`Future`와 task 실행하기"] 장의 [`Future` 트레잇]을
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
완성될 수 없다면, `AsyncFuture::poll`은 `Poll::Pending`을 반환하며 종료될
것입니다. future에 대한 `poll` 호출들은 이전에 중단된 지점(역주: `self.state`를
`match`하여)부터 다시 시작할 것입니다(TODO: 재번역 필요). 이 과정은 future가
성공적으로 완성될 때까지 반복될 것입니다.

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
#     // We need an `init` method to actually set our self-reference
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

It's easy to get this to show UB and fail in other spectacular ways as well.

## Pinning in Practice

Let's see how pinning and the `Pin` type can help us solve this problem.

The `Pin` type wraps pointer types, guaranteeing that the values behind the
pointer won't be moved. For example, `Pin<&mut T>`, `Pin<&T>`,
`Pin<Box<T>>` all guarantee that `T` won't be moved if `T: !Unpin`.

Most types don't have a problem being moved. These types implement a trait
called `Unpin`. Pointers to `Unpin` types can be freely placed into or taken
out of `Pin`. For example, `u8` is `Unpin`, so `Pin<&mut u8>` behaves just like
a normal `&mut u8`.

However, types that can't be moved after they're pinned has a marker called
`!Unpin`. Futures created by async/await is an example of this.

### Pinning to the Stack

Back to our example. We can solve our problem by using `Pin`. Let's take a look at what
our example would look like we required a pinned pointer instead:

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

Pinning an object to the stack will always be `unsafe` if our type implements
`!Unpin`. You can use a crate like [`pin_utils`][pin_utils] to avoid writing
our own `unsafe` code when pinning to the stack.

Below, we pin the objects `test1` and `test2` to the stack:

```rust
pub fn main() {
    // test1 is safe to move before we initialize it
    let mut test1 = Test::new("test1");
    // Notice how we shadow `test1` to prevent it from being accessed again
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
#             // This makes our type `!Unpin`
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

Now, if we try to move our data now we get a compilation error:

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
#             _marker: PhantomPinned, // This makes our type `!Unpin`
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

The type system prevents us from moving the data.

> It's important to note that stack pinning will always rely on guarantees
> you give when writing `unsafe`. While we know that the _pointee_ of `&'a mut T`
> is pinned for the lifetime of `'a` we can't know if the data `&'a mut T`
> points to isn't moved after `'a` ends. If it does it will violate the Pin
> contract.
>
> A mistake that is easy to make is forgetting to shadow the original variable
> since you could drop the `Pin` and move the data after `&'a mut T`
> like shown below (which violates the Pin contract):
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

### Pinning to the Heap

Pinning an `!Unpin` type to the heap gives our data a stable address so we know
that the data we point to can't move after it's pinned. In contrast to stack
pinning, we know that the data will be pinned for the lifetime of the object.

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

Some functions require the futures they work with to be `Unpin`. To use a
`Future` or `Stream` that isn't `Unpin` with a function that requires
`Unpin` types, you'll first have to pin the value using either
`Box::pin` (to create a `Pin<Box<T>>`) or the `pin_utils::pin_mut!` macro
(to create a `Pin<&mut T>`). `Pin<Box<Fut>>` and `Pin<&mut Fut>` can both be
used as futures, and both implement `Unpin`.

For example:

```rust,edition2018,ignore
use pin_utils::pin_mut; // `pin_utils` is a handy crate available on crates.io

// A function which takes a `Future` that implements `Unpin`.
fn execute_unpin_future(x: impl Future<Output = ()> + Unpin) { /* ... */ }

let fut = async { /* ... */ };
execute_unpin_future(fut); // Error: `fut` does not implement `Unpin` trait

// Pinning with `Box`:
let fut = async { /* ... */ };
let fut = Box::pin(fut);
execute_unpin_future(fut); // OK

// Pinning with `pin_mut!`:
let fut = async { /* ... */ };
pin_mut!(fut);
execute_unpin_future(fut); // OK
```

## Summary

1. If `T: Unpin` (which is the default), then `Pin<'a, T>` is entirely
equivalent to `&'a mut T`. in other words: `Unpin` means it's OK for this type
to be moved even when pinned, so `Pin` will have no effect on such a type.

2. Getting a `&mut T` to a pinned T requires unsafe if `T: !Unpin`.

3. Most standard library types implement `Unpin`. The same goes for most
"normal" types you encounter in Rust. A `Future` generated by async/await is an exception to this rule.

4. You can add a `!Unpin` bound on a type on nightly with a feature flag, or
by adding `std::marker::PhantomPinned` to your type on stable.

5. You can either pin data to the stack or to the heap.

6. Pinning a `!Unpin` object to the stack requires `unsafe`

7. Pinning a `!Unpin` object to the heap does not require `unsafe`. There is a shortcut for doing this using `Box::pin`.

8. For pinned data where `T: !Unpin` you have to maintain the invariant that its memory will not
get invalidated or repurposed _from the moment it gets pinned until when drop_ is called. This is
an important part of the _pin contract_.

["Executing `Future`s and Tasks"]: ../02_execution/01_chapter.md
[the `Future` trait]: ../02_execution/02_future.md
[pin_utils]: https://docs.rs/pin-utils/
