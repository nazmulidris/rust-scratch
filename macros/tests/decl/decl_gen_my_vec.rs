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

//! A `vec`-like macro: <https://gist.github.com/jonhoo/ec57882a976a2d2a92b3138ea25cd45a>

macro_rules! my_vec {
  () => {{
    let vec = Vec::new();
    vec
  }};
  ($($el: expr) => *) => {{
      let mut vec = Vec::new();
      $(
        vec.push($el);
      )*
      vec
  }};
}

#[test]
fn test_empty() {
  let vec: Vec<i32> = my_vec!();
  assert_eq!(vec.len(), 0);
}

#[test]
fn test_double() {
  let vec: Vec<i32> = my_vec![1 => 2 => 3];
  assert_eq!(vec.len(), 3);
}
