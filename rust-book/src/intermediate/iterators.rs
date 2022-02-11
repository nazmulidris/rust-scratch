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

//! - Rust book: <https://doc.rust-lang.org/book/ch13-02-iterators.html>
//! - Impl iterator type: <https://stackoverflow.com/a/27535594/2085356>
//! - Mock CLI args: <https://stackoverflow.com/a/54594023/2085356>
//! - Performance of iterators: <https://doc.rust-lang.org/book/ch13-04-performance.html>

use std::{env::Args, result::Iter};

pub fn run() {}

#[test]
fn test_use_iterator() {
  let v_1 = vec![1, 2];
  let mut iter = v_1.iter(); // The iterator is mutable.
  assert_eq!(iter.next(), Some(&1)); // Consuming the iterator.
  assert_eq!(iter.next(), Some(&2)); // Consuming the iterator.
  assert_eq!(iter.next(), None); // Until empty.
}

#[test]
fn test_use_iterator_and_own() {
  let v_1 = vec![1, 2];
  let mut iter = v_1.into_iter(); // Take ownership of v_1 itself.
  assert_eq!(iter.next(), Some(1)); // Consuming the iterator.
  assert_eq!(iter.next(), Some(2)); // Consuming the iterator.
  assert_eq!(iter.next(), None); // Until empty.

  // 🧨 Can't access v_1 anymore! The following line won't work.
  // assert_eq!(v_1.len(), 2);
}

#[test]
fn test_use_iterator_and_mutate() {
  let mut v_1 = vec![vec![1], vec![10]]; // v_1 has to be mutable.
  let mut iter = v_1.iter_mut(); // The iterator is mutable.
  iter.next().unwrap().push(2); // Mutate the inner vector.
  assert_eq!(v_1[0], &[1, 2]); // Consuming the iterator.
}

/// https://doc.rust-lang.org/book/ch13-02-iterators.html#methods-that-consume-the-iterator
#[test]
fn test_method_that_consume_iterator_aka_consuming_adaptor() {
  let v_1 = vec![1, 2, 3];
  let iter = v_1.iter();

  // This takes ownership of & consumes the iterator. It is a "consuming adaptor".
  let sum: i64 = iter.sum();

  assert_eq!(sum, 6);
  assert_eq!(v_1.len(), 3); // The vector is still available.
}

/// https://doc.rust-lang.org/book/ch13-02-iterators.html#methods-that-produce-other-iterators
#[test]
fn test_method_that_produce_iterator_aka_iterator_adaptor() {
  let v_1 = vec![1, 2, 3];

  // This creates a new iterator. It is an "iterator adaptor".
  let iter = v_1.iter().map(|x| x + 1);

  // This actually consumes the iterator. They are lazy & don't do anything when created. Collect is
  // really smart and uses "turbofish": `::<>` syntax to specify the type of the collection.
  let v_2 = iter.collect::<Vec<i64>>();

  assert_eq!(v_2, vec![2, 3, 4]);
}

#[test]
fn test_iterators_and_capturing_closure() {
  // Approach 1 - filter a slice from the original vector.
  {
    let v_1 = vec![1, 2, 3, 10, 20, 30];
    let v_1_filtered_slice = v_1.iter().filter(|x| **x > 10).collect::<Vec<&i64>>();
    assert_eq!(v_1.len(), 6);
    assert_eq!(v_1_filtered_slice, vec![&20, &30]);
  }

  // Approach 2 - filter and own from the original vector (rendering it inaccessible afterwards).
  {
    let v_1 = vec![1, 2, 3, 10, 20, 30];
    let v1_filtered = v_1.into_iter().filter(|x| *x > 10).collect::<Vec<i64>>();
    assert_eq!(v1_filtered, vec![20, 30]);
  }
}

#[test]
fn test_simple_custom_iterator() {
  // Struct.
  struct Counter {
    count: u64,
  }
  impl Counter {
    fn new() -> Counter {
      Counter { count: 0 }
    }
  }

  // Impl Iterator trait for Counter.
  impl Iterator for Counter {
    type Item = u64; // Output type.
    fn next(&mut self) -> Option<Self::Item> {
      self.count += 1;
      if self.count < 6 {
        Some(self.count)
      } else {
        None
      }
    }
  }

  let counter = Counter::new();
  let v_1 = counter.into_iter().collect::<Vec<u64>>();
  assert_eq!(v_1, vec![1, 2, 3, 4, 5]);
}

/// Impl iterator type: https://stackoverflow.com/a/27535594/2085356
/// Mock CLI args: https://stackoverflow.com/a/54594023/2085356
#[test]
fn test_parse_command_line_args_via_iterator() {
  /// Struct that represents 3 arguments passed via the command line.
  #[derive(Debug, PartialEq, Clone)]
  struct CLIAppConfig {
    query: String,
    filename: String,
    case_sensitive: bool,
  }

  /// Implement the parsing function using `Iterator<Item = String>`. This works seamlessly with
  /// `mut args: env::Args` which is actually passed to this function in `main`.
  impl CLIAppConfig {
    fn parse<T>(mut args: T) -> Result<CLIAppConfig, String>
    where
      T: Iterator<Item = String>,
    {
      args.next(); // Skip the first arg.

      let query = match args.next() {
        Some(arg) => arg,
        None => return Err("Didn't get a query string.".to_string()),
      };

      let filename = match args.next() {
        Some(arg) => arg,
        None => return Err("Didn't get a file name.".to_string()),
      };

      let case_sensitive = match args.next() {
        Some(arg) => arg == "--case-sensitive",
        None => false,
      };

      Ok(CLIAppConfig {
        query,
        filename,
        case_sensitive,
      })
    }
  }

  // Approach 1 - mock cli args via fn that returns `impl Iterator<Item = String>`.
  fn create_mock_args_iterator<'a>(args: &'a [&str]) -> impl Iterator<Item = String> + 'a {
    args.iter().map(|it| it.to_string())
  }
  let mock_cli_args_1 = create_mock_args_iterator(&["", "query", "filename", "--case-sensitive"]);

  assert_eq!(
    CLIAppConfig::parse(mock_cli_args_1),
    Ok(CLIAppConfig {
      query: "query".to_string(),
      filename: "filename".to_string(),
      case_sensitive: true
    })
  );

  // Approach 2 - mock cli args by wrapping the iterator in a `dyn Box`.
  let mock_cli_args_2: Box<dyn Iterator<Item = String>> = Box::new(
    ["", "query", "filename", "--case-sensitive"]
      .iter()
      .map(|it| it.to_string()),
  );

  assert_eq!(
    CLIAppConfig::parse(mock_cli_args_2).unwrap(),
    CLIAppConfig {
      query: "query".to_string(),
      filename: "filename".to_string(),
      case_sensitive: true,
    }
  );
}

#[test]
fn test_grep_string_with_iterator() {
  fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents
      .lines()
      .filter(|line| line.contains(query))
      .collect()
  }

  let content = "duct\ntape\nmonkey donkey pasta\nducks\nmoney";
  assert_eq!(search("duct", content), vec!["duct"]);
  assert_eq!(search("duc", content), vec!["duct", "ducks"]);
  assert_eq!(search("mon", content), vec!["monkey donkey pasta", "money"]);
}
