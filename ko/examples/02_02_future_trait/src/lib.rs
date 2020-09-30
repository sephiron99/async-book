// ANCHOR: simple_future
trait SimpleFuture {
    type Output;
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output>;
}

enum Poll<T> {
    Ready(T),
    Pending,
}
// ANCHOR_END: simple_future

struct Socket;
impl Socket {
    fn has_data_to_read(&self) -> bool {
        // 소켓의 데이터가 준비가 되었는지 확인
        true
    }
    fn read_buf(&self) -> Vec<u8> {
        // Read data in from the socket
        vec![]
    }
    fn set_readable_callback(&self, _wake: fn()) {
        // `epoll` 기반의 이벤트 루프와 비슷하게, 소켓에 데이터가
        // 준비되었을 때 `_wake`가 호출될 수 있도록 콜백을 등록
    }
}

// ANCHOR: socket_read
pub struct SocketRead<'a> {
    socket: &'a Socket,
}

impl SimpleFuture for SocketRead<'_> {
    type Output = Vec<u8>;

    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        if self.socket.has_data_to_read() {
            // 소켓에 데이터가 준비됨-- 버퍼에 읽어 들이고 버퍼를 반환
            Poll::Ready(self.socket.read_buf())
        } else {
            // 소켓에 아직 데이터가 준비되지 않음
            //
            // 데이터가 확보될 때, `wake`가 호출될 수 있도록 준비함.
            // 데이터가 확보되면, `wake`가 호출되고, 이 `Future`의 사용자는
            // `poll`을 다시 호출하여 데이터를 읽을 수 있음을 알게 된다.
            // (TODO: 읽을 수 있음을 -> 읽음을?)
            self.socket.set_readable_callback(wake);
            Poll::Pending
        }
    }
}
// ANCHOR_END: socket_read

// ANCHOR: join
/// 두 개의 다른 future를 실행하여 동시에 완성하는 SimpleFuture.
///
/// 각각의 future에 대한 `poll` 함수의 호출이 교차배치될 수 있어, 각 future가
/// 각자의 페이스대로 진행될 수 있게 해준다. 이를 통해 동시성을 얻을 수 있다.
pub struct Join<FutureA, FutureB> {
    // 각 필드는 완성될 때까지 실행되어야 하는 future를 한 개씩 갖을 수 있다.
    // 만약, future가 이미 완성되었다면, 그 필드는 `None`으로 설정된다.
    // 이를 통해, future가 완성된 이후에 폴링하는 `Future` trait 규칙 위반을
    // 예방할 수 있다.
    a: Option<FutureA>,
    b: Option<FutureB>,
}

impl<FutureA, FutureB> SimpleFuture for Join<FutureA, FutureB>
where
    FutureA: SimpleFuture<Output = ()>,
    FutureB: SimpleFuture<Output = ()>,
{
    type Output = ();
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        // future `a`를 완성하려고 시도함.
        if let Some(a) = &mut self.a {
            if let Poll::Ready(()) = a.poll(wake) {
                self.a.take();
            }
        }

        // future `b`를 완성하려고 시도함.
        if let Some(b) = &mut self.b {
            if let Poll::Ready(()) = b.poll(wake) {
                self.b.take();
            }
        }

        if self.a.is_none() && self.b.is_none() {
            // 두 future 모두 완성되었음-- 성공적으로 반환함
            Poll::Ready(())
        } else {
            // 하나 또는 두 개의 future가 `Poll::Pending`을 반환하므로, 아직
            // 해야 할 작업이 남아 있습니다. future(들)은 진행이 가능 할 때
            // `wake()`를 호출할 것입니다.
            Poll::Pending
        }
    }
}
// ANCHOR_END: join

// ANCHOR: and_then
/// 두 개의 future가 완성될 때까지 순차적으로 실행하는 SimpleFuture
//
// 주의: 이 간단한 예제의 목적에 맞도록, `AndThenFut`은 첫 번째와 두 번째
// future 둘 다 생성시에 활성화되었다고 가정합니다. 진짜 `AndThen` 조합자는
// `get_breakfast.and_then(|food| eat(food))`와 같은 식으로 첫 번째 future의
// 결과에 따라 두 번째 future를 만들 수 있습니다.
pub struct AndThenFut<FutureA, FutureB> {
    first: Option<FutureA>,
    second: FutureB,
}

impl<FutureA, FutureB> SimpleFuture for AndThenFut<FutureA, FutureB>
where
    FutureA: SimpleFuture<Output = ()>,
    FutureB: SimpleFuture<Output = ()>,
{
    type Output = ();
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        if let Some(first) = &mut self.first {
            match first.poll(wake) {
                // 첫 번째 future가 완성되었습니다. 첫 번째를 제거하고 두 번째를
                // 시작합니다!
                Poll::Ready(()) => self.first.take(),
                // 첫 번째 future도 완성되지 못했습니다.
                Poll::Pending => return Poll::Pending,
            };
        }
        // 이제 첫 번재 future가 완성되었으니, 두 번째 future를 완성하려고
        // 시도합니다.
        self.second.poll(wake)
    }
}
// ANCHOR_END: and_then

mod real_future {
use std::{
    future::Future as RealFuture,
    pin::Pin,
    task::{Context, Poll},
};

// ANCHOR: real_future
trait Future {
    type Output;
    fn poll(
        // `&mut self`에서 `Pin<&mut Self>`로 변화되었음:
        self: Pin<&mut Self>,
        // `wake: fn()`에서 `cx: &mut Context<'_>`로 변화되었음:
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output>;
}
// ANCHOR_END: real_future

// `Future`가 `RealFuture`에 매치됨을 확인합니다:
impl<O> Future for dyn RealFuture<Output = O> {
    type Output = O;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        RealFuture::poll(self, cx)
    }
}
}
