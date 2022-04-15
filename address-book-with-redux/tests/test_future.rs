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

//! https://www.reddit.com/r/rust/comments/qv1sx2/pass_async_function_pointer_to_async_closure/
//! https://stackoverflow.com/a/59158908/2085356

use std::future::Future;

async fn run() {
  let a = 1;
  let lambda = move || async move {
    println!("\nrun: a={}", a);
    a
  };
  exec_lambda(lambda).await;
}

async fn run_2() {
  let a = 2;
  let lambda: Box<dyn Fn() -> _ + Send + 'static> = Box::new(move || async move {
    println!("\nrun_2: a={}", a);
    a
  });
  exec_lambda(lambda).await;
}

async fn exec_lambda<FnArg, Fut, FutRet>(lambda: FnArg)
where
  FnArg: FnOnce() -> Fut + Send + 'static,
  Fut: Future<Output = FutRet> + Send,
{
  tokio::spawn(async {
    lambda().await;
  })
  .await
  .unwrap();
}

fn call_run() -> impl Future<Output = ()> {
  run()
}

#[tokio::test]
async fn test_run() {
  run().await;
}

#[tokio::test]
async fn test_run_2() {
  run_2().await;
}

#[tokio::test]
async fn test_list_of_run_calls() {
  let list = vec![call_run(), call_run(), call_run()];
  for f in list {
    f.await;
  }
}
