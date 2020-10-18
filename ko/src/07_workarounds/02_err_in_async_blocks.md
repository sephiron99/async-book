# `async` 블록 안에서의 `?`

`async fn` 안에서와 마찬가지로, `async` 블록들 안에서 `?`의 사용도 보편적입니다.
하지만, `async` 블록의 반환 타입은 명시적으로 규정되지 않았기 때문에, 컴파일러가
`async`블록의 에러 타입을 추론하는 데 실패할 것입니다.

예를 들어, 아래 코드는

```rust,edition2018
# struct MyError;
# async fn foo() -> Result<(), MyError> { Ok(()) }
# async fn bar() -> Result<(), MyError> { Ok(()) }
let fut = async {
    foo().await?;
    bar().await?;
    Ok(())
};
```

아래 에러를 발생시킬 것입니다.

```
error[E0282]: type annotations needed
 --> src/main.rs:5:9
  |
4 |     let fut = async {
  |         --- consider giving `fut` a type
5 |         foo().await?;
  |         ^^^^^^^^^^^^ cannot infer type
```

불행하게도, "`fut`에 타입을 부여하는" 방법이나, 명시적으로 `async`블록의 반환
타입을 지정하는 방법은 현재 존재하지 않습니다. 이 문제를 해결하기 위해서,
`async`블록에 성공과 에러의 타입을 제공하기 위해 아래와 같이 "turbofish"
연산자를 사용하세요.

```rust,edition2018
# struct MyError;
# async fn foo() -> Result<(), MyError> { Ok(()) }
# async fn bar() -> Result<(), MyError> { Ok(()) }
let fut = async {
    foo().await?;
    bar().await?;
    Ok::<(), MyError>(()) // <- 이 곳의 명시적 타입 주해에 유의할 것.
};
```

