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

//! Rust book: <https://doc.rust-lang.org/book/ch16-02-message-passing.html>
//!
//! In a way, channels in any programming language are similar to single ownership, because once you
//! transfer a value down a channel, you can no longer use that value.
//!
//! You can move messages between threads. The messages are sent using channels. Messages are values
//! that are in memory.
//! 1. Each message is moved when sent.
//! 2. Then it is moved again when received.
//! 3. There can be many senders, but only one receiver.
//!
//! In contrast, shared memory concurrency is like multiple ownership: multiple threads can access
//! the same memory location at the same time.

use std::{
  sync::mpsc::{self, Sender},
  thread::{self, JoinHandle},
  time::Duration,
};

use r3bl_rs_utils::utils::{style_error, style_primary, style_prompt};

pub fn run() {}

#[test]
fn test_one_sender_one_receiver() {
  let (send, recv) = mpsc::channel();

  type Handles = Vec<JoinHandle<()>>;
  let mut handles: Handles = vec![];

  let payload_to_send = "hi!".to_string();
  handles.push(parallel_send_task(send.clone(), payload_to_send));
  // Can no longer access `payload_to_send` as it has been moved.

  let payload_to_recv = blocking_single_receive_task(recv);
  println!("Got: {}", style_prompt(&payload_to_recv));

  wait_for_all(handles); // No need for this, just a safety.

  // Helper functions.
  fn parallel_send_task(
    send: Sender<String>,
    payload: String,
  ) -> JoinHandle<()> {
    thread::spawn(move || {
      send.send(payload).unwrap();
      // Can no longer access `payload` as it has moved to the other thread.
      println!("{}", style_primary("Sent message!"));
    })
  }

  fn blocking_single_receive_task(recv: mpsc::Receiver<String>) -> String {
    let received = recv.recv().unwrap();
    received
  }

  fn wait_for_all(handles: Handles) {
    for handle in handles {
      handle.join().unwrap();
    }
  }
}

#[test]
fn test_multiple_sender_one_receiver() {
  let (send, recv) = mpsc::channel();

  type Handles = Vec<JoinHandle<()>>;
  let mut sender_handles: Handles = vec![];

  // Parallel 1.
  sender_handles.push(parallel_send_task(
    send.clone(),
    vec!["+ hi", "++ there", "+++ from", "++++ the", "+++++ thread"]
      .into_iter()
      .map(|s| format!("🎱{}", s))
      .collect(),
  ));

  // Parallel 2.
  sender_handles.push(parallel_send_task(
    send.clone(),
    vec!["+ hi", "++ there", "+++ from", "++++ the", "+++++ thread"]
      .into_iter()
      .rev()
      .map(|s| format!("{}🎈", s))
      .collect(),
  ));

  // Parallel 3.
  let receiver_handle = parallel_recv_task(recv);

  wait_for_all(sender_handles);

  drop(send); // Break the channel. If this isn't dropped the receiver will wait forever.

  receiver_handle.join().unwrap();

  fn parallel_recv_task(recv: mpsc::Receiver<String>) -> JoinHandle<()> {
    thread::spawn(move || {
      for received in recv.iter() {
        println!("{}", style_primary(received.as_str()));
      }
    })
  }

  fn parallel_send_task(
    send: Sender<String>,
    values: Vec<String>,
  ) -> JoinHandle<()> {
    thread::spawn(move || {
      for val in values {
        send.send(val).unwrap();
        thread::sleep(Duration::from_millis(50));
      }
    })
  }

  fn wait_for_all(handles: Handles) {
    for handle in handles {
      handle.join().unwrap();
    }
  }
}
