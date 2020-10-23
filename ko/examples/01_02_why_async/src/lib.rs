#![cfg(test)]

use {
    futures::{
        executor::block_on,
        join,
    },
    std::thread,
};

fn download(_url: &str) {
    // ...
}

#[test]
// ANCHOR: get_two_sites
fn get_two_sites() {
    // 태스크에 사용될 두 개의 스레드 생성
    let thread_one = thread::spawn(|| download("https://www.foo.com"));
    let thread_two = thread::spawn(|| download("https://www.bar.com"));

    // 두 개의 스레드가 완료될 때까지 기다림
    thread_one.join().expect("thread one panicked");
    thread_two.join().expect("thread two panicked");
}
// ANCHOR_END: get_two_sites

async fn download_async(_url: &str) {
    // ...
}

// ANCHOR: get_two_sites_async
async fn get_two_sites_async() {
    // 완성될때까지 실행된다면, 웹페이지를 비동기적으로 다운로드 할 두 개의 다른
    // "future"를 만들기
    let future_one = download_async("https://www.foo.com");
    let future_two = download_async("https://www.bar.com");

    // 두 개의 future를 완성될때까지 동시에 실행하기
    join!(future_one, future_two);
}
// ANCHOR_END: get_two_sites_async

#[test]
fn get_two_sites_async_test() {
    block_on(get_two_sites_async());
}
