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

//! - Rust book: <https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html>
//! - ? operator: <https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-question-mark-operator>
//! - Handling errors: <https://stevedonovan.github.io/rust-gentle-intro/6-error-handling.html>
//! - Test that expects panic: <https://stackoverflow.com/questions/26469715/how-do-i-write-a-rust-unit-test-that-ensures-that-a-panic-has-occurred>

use std::fs::File;

pub fn run() {}

/// https://stackoverflow.com/a/26470361/2085356
#[test]
#[should_panic]
fn test_should_panic() {
  let file_result: Result<File, std::io::Error> = File::open("does not exist.txt");
  let _response = match file_result {
    Ok(file) => format!("{:?}", file.metadata()),
    Err(error) => panic!("{:?}", error),
  };
}

/// https://doc.rust-lang.org/book/ch11-01-writing-tests.html#using-resultt-e-in-tests
#[test]
fn test_similar_to_should_panic() -> Result<(), String> {
  let file_result: Result<File, std::io::Error> = File::open("does not exist.txt");
  match file_result {
    Ok(_) => Err(String::from("File exists - but it should not!")),
    Err(_) => Ok(()), // This is expected to fail.
  }
}

/// https://stackoverflow.com/a/42649833/2085356
#[test]
fn test_should_panic_alternative() {
  fn should_panic() {
    let file_result: Result<File, std::io::Error> = File::open("does not exist.txt");
    let _response = match file_result {
      Ok(file) => format!("{:?}", file.metadata()),
      Err(error) => panic!("{:?}", error),
    };
  }
  let result = std::panic::catch_unwind(|| should_panic());
  assert!(result.is_err());
}

#[test]
fn test_simple_error_handling_via_result() {
  let file_result: Result<File, std::io::Error> = File::open("does not exist.txt");
  let response = match file_result {
    Ok(file) => format!("{:?}", file.metadata()),
    Err(error) => format!("There was a problem opening the file: {:?}", error),
  };
  assert!(response.contains("There was a problem opening the file"));
}

#[test]
fn test_fine_grained_error_handling_via_result() {
  let file_result: Result<File, std::io::Error> = File::open("does not exist.txt");
  let response_string: String = match &file_result {
    Ok(file) => format!("{:?}", file.metadata()),
    Err(error) => match error.kind() {
      std::io::ErrorKind::NotFound => "File not found.".to_string(),
      _ => "Some other problem opening the file.".to_string(),
    },
  };
  assert!(file_result.is_err());
  assert!(response_string.contains("File not found."));
}

#[test]
fn test_fine_grained_error_handling_via_question_mark_operator_and_result() {
  /// Returns Ok or Err.
  fn get_file_metadata_as_string(file_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(format!("{:?}", File::open(file_name)?.metadata()?))
  }
  let result = get_file_metadata_as_string("file does not exist.txt");
  assert!(result.is_err());
  assert!(result
    .unwrap_err()
    .to_string()
    .contains("No such file or directory"));
}

#[test]
fn test_function_that_returns_nothing_but_might_have_error_in_result() {
  fn try_to_get_metadata_silently_fail(file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(file_name)?;
    let metadata = file.metadata()?;
    println!("{:?}", metadata);
    Ok(())
  }
  let result = try_to_get_metadata_silently_fail("file does not exist.txt");
  assert!(result.is_err());
}

#[test]
fn test_fine_grained_error_handling_via_question_mark_operator_and_option() {
  /// Returns Some or None.
  fn get_file_metadata_as_string(file_name: &str) -> Option<String> {
    Some(format!(
      "{:?}",
      File::open(file_name).ok()?.metadata().ok()?
    ))
  }
  let option = get_file_metadata_as_string("file does not exist.txt");
  assert!(option.is_none());
  assert_eq!(option.unwrap_or("-1".to_string()), "-1");
}
