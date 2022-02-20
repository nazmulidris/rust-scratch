/*
 Copyright 2022 Nazmul Idris

 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at

      https://www.apache.org/licenses/LICENSE-2.0

 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
*/

//! Rust book: <https://doc.rust-lang.org/book/ch19-05-advanced-functions-and-closures.html>

pub fn run() {}

#[test]
fn test_use_fn_pointer_type() {
  fn add_one(x: i32) -> i32 {
    x + 1
  }

  type FnPtr = fn(i32) -> i32;
  type Pair = (i32, i32);
  fn call_twice(f: FnPtr, arg: i32) -> Pair {
    (f(arg), f(arg))
  }

  let (a, b) = call_twice(add_one, 5);
  assert_eq!(a, 6);
}

#[test]
fn test_pass_fn_ptr_instead_of_closure() {
  let list_of_numbers = vec![1, 2, 3];
  let list_of_strings: Vec<String> = list_of_numbers.iter().map(|i| i.to_string()).collect();
  assert_eq!(list_of_strings, vec!["1", "2", "3"]);

  // Can pass the function instead of a closure, just like in TypeScript.
  let list_of_strings: Vec<String> = list_of_numbers.iter().map(ToString::to_string).collect();
  assert_eq!(list_of_strings, vec!["1", "2", "3"]);
}

#[test]
fn test_fn_that_returns_closure() {
  type MyFn = dyn Fn(i32) -> i32;
  fn returns_closure() -> Box<MyFn> {
    Box::new(|x| x + 1)
  }

  assert_eq!(returns_closure()(1), 2);
}
