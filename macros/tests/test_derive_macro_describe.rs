//! # Watch macro expansion
//!
//! To watch for changes run this script:
//! `./cargo-watch-macro-expand-one-test.fish test_derive_macro_describe`
//!
//! # Watch test output
//!
//! To watch for test output run this script:
//! `./cargo-watch-one-test.fish test_derive_macro_describe`

#![allow(dead_code)]

use my_proc_macros_lib::Describe;

#[test]
fn test_proc_macro() {
  #[derive(Describe)]
  struct MyStruct {
    my_string: String,
    my_enum: MyEnum,
    my_number: i32,
  }

  #[derive(Describe)]
  enum MyEnum {
    MyVariant1,
  }

  let foo = MyStruct {
    my_string: "Hello".to_string(),
    my_enum: MyEnum::MyVariant1,
    my_number: 42,
  };
  let foo = foo.describe();
  assert_eq!(
    foo,
    "MyStruct is a struct with these named fields: my_string, my_enum, my_number"
  );
}

#[test]
fn test_proc_macro_2() {
  #[derive(Describe)]
  struct Point<X, Y>
  where
    X: std::fmt::Display,
    Y: std::fmt::Display,
  {
    x: X,
    y: Y,
  }

  let my_pt: Point<i32, i32> = Point {
    x: 1 as i32,
    y: 2 as i32,
  };
  assert_eq!(
    my_pt.describe(),
    "Point is a struct with these named fields: x, y"
  );
}
