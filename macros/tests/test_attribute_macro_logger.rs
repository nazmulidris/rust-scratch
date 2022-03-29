/*
 *   Copyright (c) 2022 Nazmul Idris
 *   All rights reserved.

 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at

 *   http://www.apache.org/licenses/LICENSE-2.0

 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
*/

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

  let result_str = foo();
  // Strange way to test for these 3 strings to exist in `result_str` since the order is
  // not guaranteed.
  vec!["a", "b", "c"]
    .iter()
    .for_each(|ident| {
      assert!(result_str.contains(ident));
    });
}
