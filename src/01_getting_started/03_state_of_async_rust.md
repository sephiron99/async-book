# 비동기적 러스트의 현황

비동기적 러스트 생태계는 장시간에 걸쳐 많은 진화를 이루어냈기 떄문에, 어떤
도구를 사용해야 할지, 어떤 라이브러리에 투자할지, 또는 어떤 문서를 읽어야 할 지
판단하기 어려울 수도 있습니다.


The asynchronous Rust ecosystem has undergone a lot of evolution
over time, so it can be hard to know what tools to use, what libraries to invest
in, or what documentation to read. However, the `Future` trait inside the
standard library and the `async`/`await` language feature has recently been
stabilized. The ecosystem as a whole is therefore in the midst of migrating to
the newly-stabilized API, after which point churn will be significantly reduced.

At the moment, however, the ecosystem is still undergoing rapid development
and the asynchronous Rust experience is unpolished. Most libraries still
use the 0.1 definitions of the `futures` crate, meaning that to interoperate
developers frequently need to reach for the `compat` functionality from the
0.3 `futures` crate. The `async`/`await` language feature is still new.
Important extensions like `async fn` syntax in trait methods are still
unimplemented, and the current compiler error messages can be difficult to
parse.

That said, Rust is well on its way to having some of the most performant
and ergonomic support for asynchronous programming around, and if you're not
afraid of doing some spelunking, enjoy your dive into the world of
asynchronous programming in Rust!

