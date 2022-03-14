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
use tokio_example_lib::{
  middleware::{Future, SafeFnWrapper},
  my_middleware::{adder_mw, logger_mw, Action},
};

#[tokio::main]
async fn main() {
  let mut handles = Vec::<Future<Option<Action>>>::new();

  // Spawn tasks and don't await their completion - fire and forget so to speak.
  {
    let mw_fun: SafeFnWrapper<Action> = logger_mw();
    handles.push(mw_fun.spawn(Action::Add(1, 2)));
    handles.push(mw_fun.spawn(Action::Add(1, 2)));
  }

  // Spawn tasks and await their completion.
  {
    let mw_fun: SafeFnWrapper<Action> = adder_mw();
    println!("{:?}", mw_fun.spawn(Action::Add(1, 2)).await.unwrap());
    println!("{:?}", mw_fun.spawn(Action::Add(1, 2)).await.unwrap());
  }

  // Needed to wait for all the spawned futures to complete, otherwise the tokio runtime spawned in
  // `main()` before the spawned futures complete.
  // More info: https://tokio.rs/tokio/topics/bridging
  for handle in handles {
    handle.await.unwrap();
  }
}
