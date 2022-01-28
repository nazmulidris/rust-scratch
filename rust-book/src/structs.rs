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

use std::fmt::{Debug, Formatter};
use rust_example_lib::type_utils::type_utils::type_of;

/// Rust book: https://doc.rust-lang.org/book/ch05-01-defining-structs.html
pub fn run() {}

/// https://doc.rust-lang.org/std/fmt/trait.Display.html
/// https://doc.rust-lang.org/std/fmt/trait.Debug.html
/// https://doc.rust-lang.org/std/string/trait.ToString.html
/// https://loige.co/how-to-to-string-in-rust/
/// https://doc.rust-lang.org/book/ch05-01-defining-structs.html#ownership-of-struct-data
#[test]
fn test_simple_struct_with_to_string_trait() {
  /// Name.
  /// String and not &str since struct should own this data.
  struct Name {
    pub first: String,
    pub last: String,
  }
  impl ToString for Name {
    fn to_string(self: &Self) -> String {
      format!("{}, {}", self.first, self.last)
    }
  }

  /// User.
  /// Name and not &Name since struct should own this data.
  /// String and not &str since struct should own this data.
  struct User {
    pub active: bool,
    pub name: Name,
    pub email: String,
    pub sign_in_count: u64,
  }
  impl ToString for User {
    fn to_string(self: &Self) -> String {
      format!("({}, {}, {}, {})", self.active, self.name.to_string(), self.email, self.sign_in_count)
    }
  }

  /// Factory.
  fn build_user(first: &str, last: &str, email: &str) -> User {
    User {
      active: true,
      name: Name { first: first.to_string(), last: last.to_string() },
      email: email.to_string(),
      sign_in_count: 0,
    }
  }

  let u1 = build_user("john", "doe", "jd@gmail");
  assert_eq!(u1.to_string(), "(true, john, doe, jd@gmail, 0)");
  let u2 = User {
    name: Name { first: "un".to_string(), last: "known".to_string() },
    ..u1
  };
  assert_eq!(u2.to_string(), "(true, un, known, jd@gmail, 0)");

  // 🧨 The following line will not work, since we have moved `String` (email) from `u1` -> `u2`.
  // The `bool` and `u64` implement the `Copy` trait which is why they won't be moved.
  // assert_eq!(u1.to_string(), "(true, john, doe, jd@gmail, 0)");
}

#[test]
fn test_tuple_struct() {
  // Color.
  struct Color(i32, i32, i32);
  impl ToString for Color {
    fn to_string(self: &Self) -> String { format!("(r:{}, g:{}, b:{})", self.0, self.1, self.2) }
  }

  // Point2d.
  struct Point2d(i32, i32);

  // Point3d.
  struct Point3d(i32, i32, i32);

  let black = Color(0, 0, 0);
  let origin_2d = Point2d(0, 0);
  let origin_3d = Point3d(0, 0, 0);

  assert_eq!(black.to_string(), "(r:0, g:0, b:0)");
  assert_eq!(origin_2d.0 + origin_2d.1, 0);
  assert_eq!(origin_3d.0 + origin_3d.1 + origin_3d.2, 0);
}

/// typeof: https://stackoverflow.com/a/58119924/2085356
#[test]
fn test_unit_like_struct_with_no_fields() {
  // AlwaysEqual has no fields. Just for identification or narrowing types.
  struct AlwaysEqual;
  impl PartialEq for AlwaysEqual {
    fn eq(self: &Self, _other: &Self) -> bool {
      self == _other
    }
  }
  impl Debug for AlwaysEqual {
    fn fmt(self: &Self, f: &mut Formatter<'_>) -> std::fmt::Result {
      write!(f, "({})", "AlwaysEqual")
    }
  }

  let subject = AlwaysEqual;
  assert_eq!(type_of(&subject), "rust_example::structs::test_unit_like_struct_with_no_fields::AlwaysEqual");
}

/// https://doc.rust-lang.org/book/ch05-02-example-structs.html
#[test]
fn test_struct_that_derives_debug_trait() {
  #[derive(Debug)]
  struct Rect {
    width: u32,
    height: u32,
  }

  let rect1 = dbg!(Rect { width: 30, height: 30 });
  assert_eq!(format!("{:?}", rect1), "Rect { width: 30, height: 30 }");
  assert_eq!(format!("{:#?}", rect1), "Rect {\n    width: 30,\n    height: 30,\n}");
}

/// https://doc.rust-lang.org/book/ch05-03-method-syntax.html
#[test]
fn test_struct_with_methods() {
  #[derive(Debug)]
  struct Rect {
    width: u32,
    height: u32,
  }
  impl Rect {
    fn area(self: &Self) -> u32 { self.width * self.height }

    fn can_hold(self: &Self, other: &Self) -> bool {
      self.width > other.width && self.height > other.height
    }
  }

  let rect1 = Rect { width: 30, height: 30 };
  assert_eq!(rect1.area(), 30 * 30);
  let rect2 = Rect { width: 100, height: 100 };
  assert_eq!(rect2.can_hold(&rect1), true);
}

#[test]
fn test_struct_with_associated_functions() {
  #[derive(Debug)]
  struct Rect {
    pub width: u32,
    pub height: u32,
  }
  impl Rect {
    /// Not a method (no `self: &Self`), but an associated function.
    fn square(size: u32) -> Self {
      Rect { width: size, height: size }
    }
  }

  // This is just like `String::from`.
  let rect1 = Rect::square(100);
  assert_eq!(rect1.width + rect1.height, 200);
}
