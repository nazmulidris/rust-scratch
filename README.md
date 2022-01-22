# Introduction

Code snippets and experiments for learning Rust.

# Test snippets

[More info](https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html)

## Single / inline

```rust
#[test]
fn test_something() {
  let tuple1: (i32, String) = (100, "123".to_string());
  let (number, text) = tuple1;
  assert_eq!(number, 100);
  assert_eq!(text, "123");
}
```

## Module

```rust
///
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_template() {}
}
```

# References

- [The Rust Programming Language book](https://doc.rust-lang.org/book/)
- [String, &String, &str](https://www.ameyalokare.com/rust/2017/10/12/rust-str-vs-String.html)
- [rust-ansi-term](https://github.com/ogham/rust-ansi-term)
- [std::fmt](https://doc.rust-lang.org/std/fmt/)
- [Primitive data types](https://learning-rust.github.io/docs/a8.primitive_data_types.html)
- [String <-> &str conversions](https://blog.mgattozzi.dev/how-do-i-str-string/)
- [String <-> &str conversions](https://stackoverflow.com/a/29026565/2085356)
- [Unwrap & expect, Result -> Ok, Err](https://learning-rust.github.io/docs/e4.unwrap_and_expect.html)
- [Rust turbofish](https://techblog.tonsser.com/posts/what-is-rusts-turbofish)
- [Unit testing](https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html)
- [Error handling - array index out of bounds](https://users.rust-lang.org/t/array-out-of-bound-error-handling/26939)
- [Range and borrowing limitations, clone instead](https://stackoverflow.com/a/62480671/2085356)
- [Deref and Ref as different type](https://stackoverflow.com/a/41273331/2085356)
