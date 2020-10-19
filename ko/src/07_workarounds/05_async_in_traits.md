# 트레잇 내부의 `async`

현재로서는, 트레잇 안에서 `async fn`를 사용할 수 없습니다. 이유가 좀 복잡한데,
향후 이 제약을 해소하기 위한 계획이 있긴 합니다.

Rust가 언어 차원에서 공식지원하기 전 까지는, [async-trait crate from
crates.io](https://github.com/dtolnay/async-trait)으로 트레잇 안에서 `async
fn`을 사용하면 됩니다.


[async-trait crate from crates.io](https://github.com/dtolnay/async-trait)
메소드는 매 함수호출마다 힙영역에 할당할 것입니다. 이 때 발생하는 성능저하가
전체 어플리케이션에 비하면 별로 대단한 것은 아니지만, 초당 수백만 회의 호출이
예상되는 저수준 공용 API에 사용할 때에는 염두에 두어야 합니다.
