#![cfg(test)]
#![recursion_limit="128"]

mod example {
// ANCHOR: example
use futures::{
    future::FutureExt, // `.fuse()`에 필요
    pin_mut,
    select,
};

async fn task_one() { /* ... */ }
async fn task_two() { /* ... */ }

async fn race_tasks() {
    let t1 = task_one().fuse();
    let t2 = task_two().fuse();

    pin_mut!(t1, t2);

    select! {
        () = t1 => println!("task one completed first"),
        () = t2 => println!("task two completed first"),
    }
}
// ANCHOR_END: example
}

mod default_and_complete {
// ANCHOR: default_and_complete
use futures::{future, select};

async fn count() {
    let mut a_fut = future::ready(4);
    let mut b_fut = future::ready(6);
    let mut total = 0;

    loop {
        select! {
            a = a_fut => total += a,
            b = b_fut => total += b,
            complete => break,
            default => unreachable!(), // 실행되지 않음(future들은 준비되자마자 완성됨)
        };
    }
    assert_eq!(total, 10);
}
// ANCHOR_END: default_and_complete

#[test]
fn run_count() {
    futures::executor::block_on(count());
}
}

mod fused_stream {
// ANCHOR: fused_stream
use futures::{
    stream::{Stream, StreamExt, FusedStream},
    select,
};

async fn add_two_streams(
    mut s1: impl Stream<Item = u8> + FusedStream + Unpin,
    mut s2: impl Stream<Item = u8> + FusedStream + Unpin,
) -> u8 {
    let mut total = 0;

    loop {
        let item = select! {
            x = s1.next() => x,
            x = s2.next() => x,
            complete => break,
        };
        if let Some(next_num) = item {
            total += next_num;
        }
    }

    total
}
// ANCHOR_END: fused_stream
}

mod fuse_terminated {
// ANCHOR: fuse_terminated
use futures::{
    future::{Fuse, FusedFuture, FutureExt},
    stream::{FusedStream, Stream, StreamExt},
    pin_mut,
    select,
};

async fn get_new_num() -> u8 { /* ... */ 5 }

async fn run_on_new_num(_: u8) { /* ... */ }

async fn run_loop(
    mut interval_timer: impl Stream<Item = ()> + FusedStream + Unpin,
    starting_num: u8,
) {
    let run_on_new_num_fut = run_on_new_num(starting_num).fuse();
    let get_new_num_fut = Fuse::terminated();
    pin_mut!(run_on_new_num_fut, get_new_num_fut);
    loop {
        select! {
            () = interval_timer.select_next_some() => {
                // 타이머가 경과되었음. 아직 실행되지 않고 있는 future가 있다면,
                // 새 `get_new_num_fut`를 시작
                if get_new_num_fut.is_terminated() {
                    get_new_num_fut.set(get_new_num().fuse());
                }
            },
            new_num = get_new_num_fut => {
                // 새 숫자가 도착함-- 새 `run_on_new_num_fut`를 시작하고 예전
                // 것을 드랍함.
                run_on_new_num_fut.set(run_on_new_num(new_num).fuse());
            },
            // `run_on_new_num_fut`를 실행
            () = run_on_new_num_fut => {},
            // 모든 future가 완성되었다면 패닉. 왜냐하면 `indefinitely`는 값들을
            // 무기한으로 내야(yield) 함
            complete => panic!("`interval_timer` completed unexpectedly"),
        }
    }
}
// ANCHOR_END: fuse_terminated
}

mod futures_unordered {
// ANCHOR: futures_unordered
use futures::{
    future::{Fuse, FusedFuture, FutureExt},
    stream::{FusedStream, FuturesUnordered, Stream, StreamExt},
    pin_mut,
    select,
};

async fn get_new_num() -> u8 { /* ... */ 5 }

async fn run_on_new_num(_: u8) -> u8 { /* ... */ 5 }

// `get_new_num`로부터 나온 마지막 숫자를 가지고 `run_on_new_num`를 실행
//
// `get_new_num`은 타이머가 경과될 때마다 즉시 현재 실행중인 `run_on_new_num`을
// 취소하고 새 반환값으로 대체하면서 재시작됨.
async fn run_loop(
    mut interval_timer: impl Stream<Item = ()> + FusedStream + Unpin,
    starting_num: u8,
) {
    let mut run_on_new_num_futs = FuturesUnordered::new();
    run_on_new_num_futs.push(run_on_new_num(starting_num));
    let get_new_num_fut = Fuse::terminated();
    pin_mut!(get_new_num_fut);
    loop {
        select! {
            () = interval_timer.select_next_some() => {
                // 타이머 경과됨. 실행중인 `get_new_num_fut`이 없다면 새로
                // 시작함.
                if get_new_num_fut.is_terminated() {
                    get_new_num_fut.set(get_new_num().fuse());
                }
            },
            new_num = get_new_num_fut => {
                // 새 숫자가 도착함-- 새 `run_on_new_num_fut`를 시작함.
                run_on_new_num_futs.push(run_on_new_num(new_num));
            },
            // `run_on_new_num_futs`를 실행하고 완성된 `run_on_new_num_futs`가
            // 있는 지 확인함
            res = run_on_new_num_futs.select_next_some() => {
                println!("run_on_new_num_fut returned {:?}", res);
            },
            // 모든 것이 완성되었다면 패닉. 왜냐하면 `interval_timer`는 값을 무기한으로 내야 함
            complete => panic!("`interval_timer` completed unexpectedly"),
        }
    }
}

// ANCHOR_END: futures_unordered
}
