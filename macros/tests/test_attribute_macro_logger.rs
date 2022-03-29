#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

//! # Watch macro expansion
//!
//! To watch for changes run this script:
//! `./cargo-watch-macro-expand-one-test.fish test_attribute_macro_logger`
//!
//! # Watch test output
//!
//! To watch for test output run this script:
//! `./cargo-watch-one-test.fish test_attribute_macro_logger`

use my_proc_macros_lib::{attrib_macro_logger_1, attrib_macro_logger_2};

#[test]
fn test_attribute_macro_logger_1() {
  #[attrib_macro_logger_1(key = "value")]
  fn this_fn_will_be_consumed_and_replaced() -> i32 { 42 }
  assert_eq!(key(), "value");
}

#[test]
fn test_attribute_macro_logger_2() {
  #[attrib_macro_logger_2(a + b + c)]
  fn foo() -> i32 { 42 }
  assert_eq!(foo(), 42);
}
