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

use std::{fmt::Display, result, sync::Barrier};

use chrono::{DateTime, Utc};

/// Rust book:
/// 1. https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html
/// 2. https://doc.rust-lang.org/book/ch11-03-test-organization.html#the-tests-module-and-cfgtest
pub fn run() {}

/// Using testing module.
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_lifetime_simple() {
    fn longest_str<'a>(first: &'a str, second: &'a str) -> &'a str {
      if first.len() > second.len() {
        first
      } else {
        second
      }
    }

    // Simple lifetime example - string_1 and string_2 are have the same lifetime.
    {
      let string_1 = "hellooo".to_string();
      let string_2 = "world".to_string();
      let result = longest_str(&string_1, &string_2);
      assert_eq!(result, "hellooo");
    }

    // Adding just a little more complexity - string_1 and string_2 have different lifetimes.
    {
      let string_1 = "hello".to_string();
      {
        let string_2 = "wrld".to_string();
        let result = longest_str(&string_1, &string_2);
        assert_eq!(result, "hello");
      }
    }

    // 🧨 The following won't work, as `string_2` won't live long enough for `result`.
    // {
    //   let string_1 = "hola".to_string();
    //   let result;
    //   {
    //     let string_2 = "world".to_string();
    //     result = longest_str(&string_1, &string_2);
    //   } // <---------------------------- 💧 `string_2` is dropped here.
    //   assert_eq!(result, "world"); // <- 💀 `result` is referring to `string_2`, gone by here.
    // }
  }

  #[test]
  fn test_lifetime_and_generics_in_structs() {
    struct HoldsReference<'a, T> {
      reference: &'a T,
      timestamp: &'a DateTime<Utc>,
    }
    impl<'a, T> HoldsReference<'a, T> {
      fn get_reference(&self) -> &'a T {
        self.reference
      }
      fn get_timestamp(&self) -> &'a DateTime<Utc> {
        self.timestamp
      }
    }

    let payload = "hello world foo bar".to_string();
    let first_word = payload.split(" ").next().unwrap(); // <- `first_word` lifetime is `'a`.

    let reference_holder = HoldsReference {
      reference: &first_word, // <- `reference` lifetime is `'a`, aka, `first_word` lifetime.
      timestamp: &Utc::now(),
    };

    assert_eq!(reference_holder.reference, &"hello"); // <- `first_word` in scope, `reference` valid.
    assert_eq!(reference_holder.get_reference(), &"hello");
  }

  #[test]
  fn test_static_lifetime() {
    // All string literals have a 'static lifetime, meaning they're around for the entire lifetime of
    // the program. The text of these strings is stored in the program's binary so they are around for
    // the entire time the program is running. Marking something w/ the static lifetime is a way to
    // make global variable - use with care.
    let s: &'static str = "hello";
    assert_eq!(s, "hello");
  }
}
