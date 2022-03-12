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

use crate::middleware::SafeFnWrapper;

/// Does not capture context or return anything.
pub fn logger_mw() -> SafeFnWrapper<Action> {
  let logger_lambda = |action: Action| {
    println!("logging: {:?}", action);
    None
  };
  SafeFnWrapper::new(logger_lambda)
}

/// Captures context and returns a `Future<Action>`.
pub fn adder_mw() -> SafeFnWrapper<Action> {
  let mut stack: Vec<i32> = Vec::new();
  let adder_lambda = move |action: Action| match action {
    Action::Add(a, b) => {
      let sum = a + b;
      stack.push(a + b);
      Some(Action::Result(sum))
    }
    _ => None,
  };
  SafeFnWrapper::new(adder_lambda)
}

/// Action enum.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Action {
  Add(i32, i32),
  Result(i32),
}
