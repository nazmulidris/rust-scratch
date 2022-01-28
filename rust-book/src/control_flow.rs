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

/// Rust book: https://doc.rust-lang.org/book/ch03-05-control-flow.html
pub fn run() {}

/// Return values from loop.
#[test]
fn test_loop_1() {
  let mut counter = 0;
  let result = loop {
    counter += 1;
    if counter == 10 {
      break counter * 2; // Return this expression (no terminating semicolon).
    }
  };
  assert_eq!(result, 20);
}

/// Labelled loops and breaking out of them.
#[test]
fn test_loop_2() {
  let mut outer_count = 0;
  let mut inner_count = 0;
  'OUTER: loop {
    outer_count += 1;
    'INNER: loop {
      inner_count += 1;
      if inner_count == 9 { break 'INNER; }
      if outer_count == 2 { break 'OUTER; }
    };
  };
  assert_eq!(inner_count, 10);
  assert_eq!(outer_count, 2);
}

/// for loop.
#[test]
fn test_for_loop() {
  let array = [0, 10, 20];
  for element in array { assert!(array.contains(&element)); }
}

/// for each loop.
#[test]
fn test_for_each_loop() {
  let array = [0, 10, 20];
  array.iter()
    .enumerate()
    .for_each(|(_index, value)| {
      assert!(array.contains(value))
    });
}

/// Range and for loop.
/// Range and borrowing limitations: https://stackoverflow.com/a/62480671/2085356
#[test]
fn test_range_for_loop() {
  let range = 1..4;
  let rev_range = range.clone().rev();
  for number in rev_range {
    assert!(range.contains(&number))
  }
}
