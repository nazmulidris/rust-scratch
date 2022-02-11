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

//! Rust book: <https://doc.rust-lang.org/book/ch10-00-generics.html>

use std::fmt::Display;

pub fn run() {}

#[test]
fn test_generics_in_function() {
  fn to_string<T: Display>(arg: T) -> String {
    format!("{}", arg)
  }
  assert!(to_string("hello").contains("hello"));
  assert!(to_string(12).contains("12"));
  assert!(to_string(12.56).contains("12.56"));
}

#[test]
fn test_generics_in_struct() {
  #[derive(Debug)]
  struct Point<T, U> {
    x: T,
    y: U,
  }

  // Fully generic impl.
  impl<T: Copy, U: Copy> Point<T, U> {
    fn x(self: &Self) -> T {
      self.x.clone()
    }
    fn y(self: &Self) -> U {
      self.y.clone()
    }
  }

  // Partially generic impl / partially specific impl.
  impl<T> Point<f64, T> {
    fn x_as_i64(self: &Self) -> i64 {
      self.x.round() as i64
    }
  }

  // Different generic struct and impl method types.
  impl<T: Copy, U: Copy> Point<T, U> {
    fn mix_up_with<V: Copy, W: Copy>(self: &Self, other: &Point<V, W>) -> Point<T, W> {
      Point {
        x: self.x.clone(),
        y: other.y.clone(),
      }
    }
  }

  let p1 = Point::<i64, i64> { x: 1, y: 2 }; // Using turbofish.
  assert_eq!(p1.x() + p1.y(), 3);

  let p2 = Point::<f64, i64> { x: 1.2, y: 2 };
  assert_eq!(p2.x_as_i64() as i64 + p2.y(), 3);

  let p3 = p1.mix_up_with(&p2);
  assert_eq!(p3.x(), 1);
  assert_eq!(p3.y(), 2);
}
