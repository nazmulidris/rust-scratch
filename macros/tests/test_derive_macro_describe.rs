//! Here's the command to see cargo expand's out out of this test:
//! `cargo expand --test test_derive_macro_describe`

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
  struct Point<T> {
    x: T,
    y: T,
  }

  // Code that should be generated:
  // impl<T: std::fmt::Display> Point<T> {
  //   fn describe(&self) -> String {
  //     format!("Point<i32> with x: {} and y: {}", self.x, self.y)
  //   }
  // }

  let my_pt: Point<i32> = Point { x: 1, y: 2 };
  assert_eq!(
    my_pt.describe(),
    "Point is a struct with these named fields: x, y"
  );
}
