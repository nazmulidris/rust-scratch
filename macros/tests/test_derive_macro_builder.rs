#![allow(dead_code)]

//! # Watch macro expansion
//!
//! To watch for changes run this script:
//! `./cargo-watch-macro-expand-one-test.fish test_derive_macro_builder`
//!
//! # Watch test output
//!
//! To watch for test output run this script:
//! `./cargo-watch-one-test.fish test_derive_macro_builder`

use my_proc_macros_lib::Builder;

#[test]
fn test_proc_macro_struct_and_enum() {
  #[derive(Builder)]
  struct MyStruct {
    my_string: String,
    my_enum: MyEnum,
    my_number: i32,
  }

  enum MyEnum {
    MyVariant1,
  }

  impl Default for MyEnum {
    fn default() -> Self { MyEnum::MyVariant1 }
  }
}

#[test]
fn test_proc_macro_no_where_clause() {
  #[derive(Builder)]
  struct Point<X, Y> {
    x: X,
    y: Y,
  }

  let my_pt: Point<i32, i32> = PointBuilder::new()
    .set_x(1 as i32)
    .set_y(2 as i32)
    .build();

  assert_eq!(my_pt.x, 1);
  assert_eq!(my_pt.y, 2);
}

#[test]
fn test_proc_macro_generics() {
  #[derive(Builder)]
  struct Point<X, Y>
  where
    X: std::fmt::Display + Clone,
    Y: std::fmt::Display + Clone,
  {
    x: X,
    y: Y,
  }

  let my_pt: Point<i32, i32> = PointBuilder::new()
    .set_x(1 as i32)
    .set_y(2 as i32)
    .build();

  assert_eq!(my_pt.x, 1);
  assert_eq!(my_pt.y, 2);
}
