## 중복되지 않게 번역하려는 파일이나 챕터를 먼저 이 README.md에 pull request해주세요.
| 대상              | ID         | 상태   |
|-------------------|------------|--------|
| 04_Pinning | sephiron99 | 번역중 |
| 03_async_await 장 | carnoxen   | 번역중 |

# async-book 한국어 번역
러스트 비동기 프로그래밍 

## 요구사항
이 async-book은 [`mdbook`]으로 만들어졌습니다. [`mdbook`]은 cargo로 설치할 수
있습니다.

```
cargo install mdbook
cargo install mdbook-linkcheck
```

[`mdbook`]: https://github.com/rust-lang/mdBook

## 빌드하기
`mdbook build`을 실행하면 `book/` 디렉토리 아래에 완성본이 만들어집니다.
```
mdbook build
```

## 개발
문서작성 도중에 변경사항을 볼 수 있으면 편리하기 때문에, `mdbook serve`로
로컬 웹서버를 실행하여 책을 보여줄 수 있습니다.
```
mdbook serve
```
