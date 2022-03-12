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

use std::sync::{Arc, Mutex};

use tokio_example_lib::{
  middleware::SafeFnWrapper,
  my_middleware::{adder_mw, logger_mw, Action},
};

// Integration tests
// Functions that are in scope: <https://stackoverflow.com/a/45151641/2085356>
// About integration tests: <https://doc.rust-lang.org/book/ch11-03-test-organization.html#the-tests-directory>
// Tokio test macro: <https://docs.rs/tokio/latest/tokio/attr.test.html>

#[tokio::test]
async fn test_logger_mw_works() {
  let result = logger_mw().spawn(Action::Add(1, 2)).await.unwrap();
  assert!(result.is_none());
}

#[tokio::test]
async fn test_adder_mw_works() {
  let result = adder_mw().spawn(Action::Add(1, 2)).await.unwrap();
  assert_eq!(result, Some(Action::Result(3)));
}

#[tokio::test]
async fn test_complex_mw_example_works() {
  let stack = Arc::new(Mutex::new(vec![]));
  let stack_ref = stack.clone();
  let adder_lambda = move |action: Action| match action {
    Action::Add(a, b) => {
      let sum = a + b;
      let mut stack_ref = stack_ref.lock().unwrap();
      stack_ref.push(a + b);
      Some(Action::Result(sum))
    }
    _ => None,
  };
  let foo = SafeFnWrapper::new(adder_lambda);
  let result = foo.spawn(Action::Add(1, 2)).await.unwrap();
  assert_eq!(result, Some(Action::Result(3)));
  assert_eq!(stack.lock().unwrap().len(), 1);
}
