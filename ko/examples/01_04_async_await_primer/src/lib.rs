#![cfg(test)]

use futures::executor::block_on;

mod first {
// ANCHOR: hello_world
// `block_on` executor는 제공받은 future가 실행되어 완성될 때까지 현재의
// 스레드를 블록한다. 다른 종류의 executor들은 여러 개의 future를 같은 스레드
// 안에서 스케줄링을 한다던가 하는 식으로 보다 복잡하게 동작한다.
use futures::executor::block_on;

async fn hello_world() {
    println!("hello, world!");
}

fn main() {
    let future = hello_world(); // 아무것도 출력되지 않음
    block_on(future); // `future` 가 실행되어 "hello, world!"가 출력됨
}
// ANCHOR_END: hello_world

#[test]
fn run_main() { main() }
}

struct Song;
async fn learn_song() -> Song { Song }
async fn sing_song(_: Song) {}
async fn dance() {}

mod second {
use super::*;
// ANCHOR: block_on_each
fn main() {
    let song = block_on(learn_song());
    block_on(sing_song(song));
    block_on(dance());
}
// ANCHOR_END: block_on_each

#[test]
fn run_main() { main() }
}

mod third {
use super::*;
// ANCHOR: block_on_main
async fn learn_and_sing() {
    // 노래를 부르기 전에 노래를 배울 때까지 기다림.
    // 스레드를 블록하지 않기 위해 `block_on` 대신에 `.await`을 사용한다. 이렇게
    // 하면, `춤`을 동시에 출 수 있다.
    let song = learn_song().await;
    sing_song(song).await;
}

async fn async_main() {
    let f1 = learn_and_sing();
    let f2 = dance();

    // `join!`은 `.await`와 비슷하지만 여러 개의 future를 동시에 기다릴 수 있다.
    // `learn_and_sing` future에서 일시적으로 블록되었더라도, `dance` future는
    // 현재의 스레드를 가져올 것이다. `dance`가 블록되면, `learn_and_sing`은 다시
    // 스레드를 가져올 수 있다. 둘 다 블록되면, `async_main`이 블록되고,
    // executor에게 스레드를 양보할 것이다.
    futures::join!(f1, f2);
}

fn main() {
    block_on(async_main());
}
// ANCHOR_END: block_on_main

#[test]
fn run_main() { main() }
}
