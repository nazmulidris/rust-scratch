/*
 *   Copyright (c) 2024 Nazmul Idris
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

//! ## Implementing the `Future` trait.
//!
//! From: <https://rust-lang.github.io/async-book/02_execution/03_wakeups.html>.
//! - Build a timer that wakes up a task after a certain amount of time, to explore how
//!   `Waker` works.
//! - For the sake of the example, we'll just spin up a new thread when the timer is
//!   created, sleep for the required time, and then signal the timer future when the time
//!   window has elapsed.
//! Links:
//! 1. https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html
//! 2. https://doc.rust-lang.org/std/future/trait.Future.html
//! 3. https://rust-lang.github.io/async-book/02_execution/05_io.html

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

use crossterm::style::Stylize;

#[derive(Default)]
pub struct SharedState {
    pub completed: bool,
    pub waker: Option<Waker>,
}

#[derive(Default)]
pub struct TimerFuture {
    pub shared_state: Arc<Mutex<SharedState>>,
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        match shared_state.completed {
            true => {
                eprintln!("{}", "TimerFuture is completed".to_string().green());
                Poll::Ready(())
            }
            false => {
                eprintln!("{}", "TimerFuture is not completed".to_string().red());
                // Importantly, we have to update the Waker every time the future is
                // polled because the future may have moved to a different task with a
                // different Waker. This will happen when futures are passed around
                // between tasks after being polled.
                shared_state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let new_instance = TimerFuture::default();

        let shared_state_clone = new_instance.shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = shared_state_clone.lock().unwrap();
            shared_state.completed = true;
            shared_state.waker.take().unwrap().wake();
        });

        new_instance
    }
}

#[tokio::test]
async fn run_timer_future_with_tokio() {
    let timer_future = TimerFuture::new(Duration::from_millis(10));
    let shared_state = timer_future.shared_state.clone();
    assert!(!shared_state.lock().unwrap().completed);
    timer_future.await;
    assert!(shared_state.lock().unwrap().completed);
}
