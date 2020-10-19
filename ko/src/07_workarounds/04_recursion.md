# 재귀

내부적으로, `async fn`은 `.await`하는 하위 `Future`를 갖는 상태기계 타입을
만듭니다. 때문에, `async fn`을 재귀적으로 사용하기 살짝 까다롭습니다. 왜냐하면,
결과를 도출할 상태기계 타입이 그 자신을 포함해야 하기 때문입니다.

```rust,edition2018
# async fn step_one() { /* ... */ }
# async fn step_two() { /* ... */ }
# struct StepOne;
# struct StepTwo;
// This function:
async fn foo() {
    step_one().await;
    step_two().await;
}
// generates a type like this:
enum Foo {
    First(StepOne),
    Second(StepTwo),
}

// So this function:
async fn recursive() {
    recursive().await;
    recursive().await;
}

// generates a type like this:
enum Recursive {
    First(Recursive),
    Second(Recursive),
}
```

위 예제는 무한한 크기의 타입을 만들기 때문에 작동하지 않습니다. 컴파일러 오류는 다음과 같습니다.

```
error[E0733]: recursion in an `async fn` requires boxing
 --> src/lib.rs:1:22
  |
1 | async fn recursive() {
  |                      ^ an `async fn` cannot invoke itself directly
  |
  = note: a recursive `async fn` must be rewritten to return a boxed future.
```

제대로 작동하게 하기 위해서는, `Box`를 이용해 우회접근해야 합니다. 불행하게도,
컴파일러의 제한에 따라 `Box::pin`으로 `recursive()` 호출을 감싸는 것만으로는
충분하지 않습니다. 제대로 하려면은, 아래 예제처럼 `recursive`를 `.boxed()`된 비
`async` 블록 안에 넣어야 합니다.

```rust,edition2018
{{#include ../../examples/07_05_recursion/src/lib.rs:example}}
```
