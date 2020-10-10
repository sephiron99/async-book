#![allow(unused)]
#![cfg(test)]

mod async_fn_and_block_examples {
use std::future::Future;
// ANCHOR: async_fn_and_block_examples

// `foo()`는 `Future<Output = u8>`을 구현한 타입을 반환합니다.
// `foo().await`은 `u8` 타입의 값을 나타낼 것입니다.
async fn foo() -> u8 { 5 }

fn bar() -> impl Future<Output = u8> {
    // 이 `async` 블록은`Future<Output = u8>`을 구현한
    // 타입을 반환합니다.
    async {
        let x: u8 = foo().await;
        x + 5
    }
}
// ANCHOR_END: async_fn_and_block_examples
}

mod async_lifetimes_examples {
use std::future::Future;
// ANCHOR: lifetimes_expanded
// 이 함수는
async fn foo(x: &u8) -> u8 { *x }

// 이 함수와 같습니다.
fn foo_expanded<'a>(x: &'a u8) -> impl Future<Output = u8> + 'a {
    async move { *x }
}
// ANCHOR_END: lifetimes_expanded

async fn borrow_x(x: &u8) -> u8 { *x }

#[cfg(feature = "never_compiled")]
// ANCHOR: static_future_with_borrow
fn bad() -> impl Future<Output = u8> {
    let x = 5;
    borrow_x(&x) // ERROR: `x`가 더 이상 쓸 수 없습니다.
}

fn good() -> impl Future<Output = u8> {
    async {
        let x = 5;
        borrow_x(&x).await
    }
}
// ANCHOR_END: static_future_with_borrow
}

mod async_move_examples {
use std::future::Future;
// ANCHOR: async_move_examples
/// `async` 블록:
///
/// 여러 `async` 블록은 같은 지역 변수에 접근할 수 있고
/// 변수의 구역 안에서 실행될 수 있습니다.
async fn blocks() {
    let my_string = "foo".to_string();

    let future_one = async {
        // ...
        println!("{}", my_string);
    };

    let future_two = async {
        // ...
        println!("{}", my_string);
    };

    // 두 future를 완전히 실행해 "foo"를 두 번 출력합니다:
    let ((), ()) = futures::join!(future_one, future_two);
}

/// `async move` 블록:
///
/// `async move` 블록으로부터 나온 `Future`가 캡처를 옮긴 이후로는 `async move` 
/// 블록 하나만 같은 캡처된 변수에 접근할 수 있습니다. 허나 이러면 `Future`를 원래 
/// 그 변수의 구역에서보다 오래 살게 합니다.
fn move_block() -> impl Future<Output = ()> {
    let my_string = "foo".to_string();
    async move {
        // ...
        println!("{}", my_string);
    }
}
// ANCHOR_END: async_move_examples
}
