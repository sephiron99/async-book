#![cfg(test)]

mod stream_trait {
use {
    futures::stream::{Stream as RealStream},
    std::{
        pin::Pin,
        task::{Context, Poll},
    },
};

// ANCHOR: stream_trait
trait Stream {
    /// 스트림이 양보하는 값의 타입
    type Item;

    /// 스트림에 있는 다음 아이템을 해결하려 한다.
    /// 아직 준비가 안 됐으면 `Poll::Pending`, 준비가 되었으면 `Poll::Ready(Some(x))`
    /// , 끝났으면 `Poll::Ready(None)`을 반환한다.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<Option<Self::Item>>;
}
// ANCHOR_END: stream_trait

// `Stream`은 `RealStream`과 같아야 합니다.
impl<I> Stream for dyn RealStream<Item = I> {
    type Item = I;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<Option<Self::Item>>
    {
        RealStream::poll_next(self, cx)
    }
}
}

mod channels {
use {
    futures::{
        channel::mpsc,
        prelude::*,
    },
};

// ANCHOR: channels
async fn send_recv() {
    const BUFFER_SIZE: usize = 10;
    let (mut tx, mut rx) = mpsc::channel::<i32>(BUFFER_SIZE);

    tx.send(1).await.unwrap();
    tx.send(2).await.unwrap();
    drop(tx);

    // `StreamExt::next` 는 `Iterator::next`와 같지만, 
    // `Future<Output = Option<T>>`을 구현한 타입을 반환합니다.
    assert_eq!(Some(1), rx.next().await);
    assert_eq!(Some(2), rx.next().await);
    assert_eq!(None, rx.next().await);
}
// ANCHOR_END: channels

#[test]
fn run_send_recv() { futures::executor::block_on(send_recv()) }
}
