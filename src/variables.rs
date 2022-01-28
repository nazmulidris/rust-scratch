/*
 * Copyright (c) 2022 Nazmul Idris. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */


use rust_example_lib::color_text::print_header;

/// https://stackoverflow.com/a/25877389/2085356
#[allow(dead_code)]
const MS_IN_MIN: i32 = 60 * 1_000; // Must not be something that has to be calculated at runtime.

/// Rust book - https://doc.rust-lang.org/book/ch03-02-data-types.html
pub fn run() {
  print_header("variables");
}

#[test]
fn test_shadowing() {
  let _x = 2 * MS_IN_MIN; // Use underscore in variables to bypass warning it isn't used.
  let _x = 10 * _x; // Shadowing.
  {
    let _x = 1_000.to_string();
    assert_eq!(_x, "1000");
  }
  assert_eq!(_x, 10 * 2 * MS_IN_MIN);
}

#[test]
fn test_tuples_simple() {
  let tuple1: (i32, String) = (100, "123".to_string());
  let (number, text) = tuple1;
  assert_eq!(number, 100);
  // assert_eq!(tuple1.0, 100); // Works because i32 gets copied.
  assert_eq!(text, "123");
  // assert_eq!(tuple1.1, "123"); // Fails because tuple1.1 (String) gets moved above.
}

#[test]
fn test_tuples_complex_1() {
  let tuple1: (i32, &String) = (100, &"123".to_string());
  let (number, text_ptr) = tuple1;

  assert_eq!(number, 100);
  assert_eq!(text_ptr, "123");
  assert_eq!(*text_ptr, "123");

  assert_eq!(tuple1.0, 100);
  assert_eq!(tuple1.1, "123");
  assert_eq!(*tuple1.1, "123");
}

#[test]
fn test_tuples_complex_2() {
  let tuple1: (&i32, &String) = (&100, &"123".to_string());
  let (number_ptr, text_ptr) = tuple1;

  assert_eq!(*number_ptr, 100);
  assert_eq!(*text_ptr, "123");

  assert_eq!(*tuple1.0, 100);
  assert_eq!(*tuple1.1, "123");
}

#[test]
fn test_array_1() {
  let mut weekdays = ["Mon", "Tue", "Wed", "Thr", "Fri", ];
  assert_eq!(weekdays.len(), 5);
  assert_eq!(weekdays[0], "Mon");
  weekdays.rotate_left(1);
  assert_eq!(weekdays[0], "Tue");
}

#[test]
fn test_array_2() {
  let num_ray: [i32; 5] = [0, 1, 2, 3, 4];
  assert_eq!(num_ray.len(), 5)
}

#[test]
fn test_array_3() {
  let num_ray: [i32; 5] = [/* initial value*/ -1; /* length */ 5];
  assert_eq!(num_ray.len(), 5);
  num_ray.iter()
    .enumerate()
    .for_each(|(index, value)| {
      assert_eq!(*value, -1);
      assert_eq!(*value, num_ray[index]);
    });
}

/// Array index out of bounds & error handling via, `match`, `Option` from `get()` / `get_mut()`.
/// - https://doc.rust-lang.org/std/primitive.slice.html#method.get_mut
/// - https://users.rust-lang.org/t/array-out-of-bound-error-handling/26939
#[test]
fn test_array_4() {
  let num_ray: [i32; 5] = [/* initial value*/ -1; /* length */ 5];
  assert_eq!(num_ray.len(), 5);
  num_ray.iter()
    .enumerate()
    .for_each(|(index, _value)| {
      match num_ray.get(index) {
        None => { /* Index out of bounds handled here. */ }
        Some(value) => { assert_eq!(*value, num_ray[index]); }
      }
    });
}
