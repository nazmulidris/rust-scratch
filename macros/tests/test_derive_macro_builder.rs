//! # Watch macro expansion
//!
//! To watch for changes run this script:
//! `./cargo-watch-macro-expand-one-test.fish test_derive_macro_builder`
//!
//! # Watch test output
//!
//! To watch for test output run this script:
//! `./cargo-watch-one-test.fish test_derive_macro_builder`

#![allow(dead_code)]

use my_proc_macros_lib::Builder;

#[test]
fn test_proc_macro() {
  #[derive(Builder,)]
  struct MyStruct {
    my_string: String,
    my_enum: MyEnum,
    my_number: i32,
  }

  #[derive(Builder,)]
  enum MyEnum {
    MyVariant1,
  }
}

fn test_proc_macro_generics() {
  #[derive(Builder,)]
  struct Point<X, Y,>
  where
    X: std::fmt::Display,
    Y: std::fmt::Display,
  {
    x: X,
    y: Y,
  }
}
