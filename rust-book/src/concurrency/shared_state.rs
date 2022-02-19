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

//! Rust book: <https://doc.rust-lang.org/book/ch16-03-shared-state.html>

use std::{
  sync::{Arc, Mutex, RwLock},
  thread::{self, JoinHandle},
};

pub fn run() {}

#[test]
fn test_threads_arc_rwlock() {
  type JoinHandleVec = Vec<JoinHandle<u32>>;
  type CounterType = Arc<RwLock<u32>>;

  let shared_counter: CounterType = Arc::new(RwLock::new(0));
  let mut handles: JoinHandleVec = vec![];

  for _ in 0..10 {
    let counter = Arc::clone(&shared_counter);
    let handle = thread::spawn(move || {
      let mut num = counter.write().unwrap();
      *num += 1;
      return *num;
    });
    handles.push(handle);
  }

  let mut future_results = 0;
  for handle in handles {
    future_results += handle.join().unwrap();
  }

  let counter_value = *shared_counter.read().unwrap();

  println!("result: {}, result_2: {}", counter_value, future_results);
  assert_eq!(counter_value, 10);
  assert_eq!(future_results, 55);
}
