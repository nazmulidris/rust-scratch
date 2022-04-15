/*
 *   Copyright (c) 2022 Nazmul Idris
 *   All rights reserved.

 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at

 *   http://www.apache.org/licenses/LICENSE-2.0

 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
*/

//! https://docs.rs/async-trait/0.1.7/async_trait/

/*
╭──────────────────────────────────────────────────────╮
│ Setup                                                │
╰──────────────────────────────────────────────────────╯
*/

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

#[async_trait]
trait Interface {
  async fn run(
    &self,
    arg: String,
  ) -> Option<String>;
  fn new() -> Self
  where
    Self: Sized;
}

struct Impl1;

#[async_trait]
impl Interface for Impl1 {
  async fn run(
    &self,
    arg: String,
  ) -> Option<String> {
    Some(format!("hello {}", arg))
  }
  fn new() -> Self {
    Self {}
  }
}

struct Impl2;

#[async_trait]
impl Interface for Impl2 {
  async fn run(
    &self,
    arg: String,
  ) -> Option<String> {
    Some(format!("hola {}", arg))
  }
  fn new() -> Self {
    Self {}
  }
}

/*
╭──────────────────────────────────────────────────────╮
│ Tests                                                │
╰──────────────────────────────────────────────────────╯
*/

#[tokio::test]
async fn test_vec_of_impl() {
  let impl1 = Impl1::new();
  let impl2 = Impl2::new();
  let vec: Vec<Arc<RwLock<dyn Interface>>> =
    vec![Arc::new(RwLock::new(impl1)), Arc::new(RwLock::new(impl2))];
  for item in vec.iter() {
    let item = item.clone();
    let result = item
      .read()
      .await
      .run("world".to_string())
      .await;
    assert!(result.unwrap().contains("world"));
  }
}

#[tokio::test]
async fn test_impl1() {
  let my_impl = Impl1::new();
  let result = my_impl
    .run("world".to_string())
    .await;
  assert_eq!(
    result,
    Some("hello world".to_string())
  );
}

#[tokio::test]
async fn test_impl2() {
  let my_impl = Impl2::new();
  let result = my_impl
    .run("world".to_string())
    .await;
  assert_eq!(
    result,
    Some("hola world".to_string())
  );
}
