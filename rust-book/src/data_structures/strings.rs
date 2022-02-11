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

//! Rust book: <https://doc.rust-lang.org/book/ch08-02-strings.html>

pub fn run() {}

#[test]
fn test_basic_strings_with_simple_trait() {
  // Person struct.
  #[derive(Debug)]
  struct Person {
    name: String,
    age: u64,
  }

  impl Person {
    fn new(name: &str, age: u64) -> Person {
      Person {
        name: String::from(name), // Copy => Owns name and returns string slice via getter.
        age,                      // Copy.
      }
    }

    fn get_name<'a>(self: &'a Self) -> &'a str {
      &self.name
    }

    fn get_age(self: &Self) -> u64 {
      self.age
    }
  }

  // WithName trait.
  trait WithName {
    fn use_name(self: &Self) -> &str;
  }

  // Person implements WithName trait.
  impl WithName for Person {
    fn use_name(self: &Self) -> &str {
      self.get_name()
    }
  }

  // Extension function on str via implementing WithName trait.
  impl WithName for str {
    fn use_name(self: &Self) -> &str {
      self
    }
  }

  // Function that works with "anything" that implements WithName trait.
  fn accept_with_name(with_name: &dyn WithName) -> &str {
    with_name.use_name()
  }

  // Tests.
  let p1: Person = Person::new("John", 42);
  let p2: Person = Person::new("Jane", 42);

  assert_eq!("John".use_name(), "John"); // Test extension function on str.

  assert_eq!(accept_with_name(&p1), "John");
  assert_eq!(p1.use_name(), "John");
  assert_eq!(p1.get_name(), "John");
  assert_eq!(p1.get_age(), 42);

  assert_eq!(accept_with_name(&p2), "Jane");
  assert_eq!(p2.use_name(), "Jane");
  assert_eq!(p2.get_name(), "Jane");
  assert_eq!(p2.get_age(), 42);
}

#[test]
fn test_concatenate_string() {
  let mut s1 = format!("{}, {}!", "hello", "world");
  s1.push(' ');
  s1.push_str("foo");
  s1 = s1 + "bar";
  s1 += "baz";
  assert_eq!(s1, "hello, world! foobarbaz");
}

#[test]
fn test_iterate_over_string() {
  // Access contents as chars.
  for char in "hello".chars() {
    assert!("hello".contains(char));
  }
  assert_eq!("hello".chars().count(), 5);

  // Access contents as bytes.
  assert_eq!("hello".bytes().count(), 5);
}
