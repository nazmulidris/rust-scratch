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

//! Rust book: <https://doc.rust-lang.org/book/ch16-01-threads.html>
//! 1-1 Threading, which is 1 OS thread corresponding to 1 Rust thread. There are crates that do
//! green threading, but that is not part of this example.

use rust_book_lib::utils::{style_dimmed, style_primary, style_prompt};
use std::{
  sync::Arc,
  thread::{self, JoinHandle},
  time::Duration,
};

pub fn run() {}

#[test]
fn test_simple() {
  // Spawn a new thread (which takes longer than the main thread to finish running).
  let spawned_thread = thread::spawn(|| {
    for i in 1..10 {
      println!("{} # {}", style_prompt("Hello from thread!"), i);
      thread::sleep(Duration::from_millis(10));
    }
  });

  // Run code on the main thread (which finishes before the spawned thread).
  for i in 1..5 {
    println!("{} # {}", style_primary("Hello from main thread!"), i);
    thread::sleep(Duration::from_millis(5));
  }

  // Wait for spawned thread to finish. Without this, it would be terminated prematurely.
  spawned_thread.join().unwrap();
}

#[test]
fn test_with_move() {
  let v = vec![1, 2, 3];

  let spawned_thread = thread::spawn(move || {
    for i in v.iter() {
      println!("{} # {}", style_primary("Hello from thread!"), i);
      thread::sleep(Duration::from_millis(10));
    }
  });

  spawned_thread.join().unwrap();
}

#[test]
fn test_with_arc_and_move() {
  let v = Arc::new(vec![1, 2, 3]);

  type Handles = Vec<JoinHandle<()>>;
  let mut handles: Handles = vec![];

  let v1 = v.clone();
  let spawned_thread = thread::spawn(move || {
    for i in v1.clone().iter() {
      println!("{} # {}", style_primary("Hello from thread 1!"), i);
      thread::sleep(Duration::from_millis(10));
    }
  });
  handles.push(spawned_thread);

  let v2 = v.clone();
  let spawned_thread_2 = thread::spawn(move || {
    for i in v2.clone().iter() {
      println!("{} # {}", style_dimmed("Hello from thread 2!"), i);
      thread::sleep(Duration::from_millis(10));
    }
  });
  handles.push(spawned_thread_2);

  for handle in handles {
    handle.join().unwrap();
  }
}
