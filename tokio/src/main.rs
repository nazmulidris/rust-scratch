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

// Imports.
use tokio_example_lib::{my_middleware::{adder_mw, logger_mw, Action}, middleware::SafeFnWrapper};

#[tokio::main]
async fn main() {
  {
    let mw_fun: SafeFnWrapper<Action> = logger_mw();
    mw_fun.spawn(Action::Add(1, 2)).await.unwrap();
    mw_fun.spawn(Action::Add(1, 2)).await.unwrap();
  }

  {
    let mw_fun: SafeFnWrapper<Action> = adder_mw();
    println!("{:?}", mw_fun.spawn(Action::Add(1, 2)).await.unwrap());
    println!("{:?}", mw_fun.spawn(Action::Add(1, 2)).await.unwrap());
  }
}
