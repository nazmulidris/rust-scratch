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

//! # async_con_par
//!
//! This project is an exploration of async concurrent and parallel programming in Rust. The inspiration
//! for this exploration is this blog post: [Tasks are the wrong
//! abstraction[(https://blog.yoshuawuyts.com/tasks-are-the-wrong-abstraction/#tasks-are-the-wrong-abstraction-for-concurrent-async-execution).
//!
//! Here are some other interesting links:
//! 1. [How futures work in runtimes w/ Context and Waker](https://gemini.google.com/app/7cdd1930f56e4b91)
//! 2. [`epoll` (linux), vs `io_uring` (linux), vs `iocp` (windows)](https://gemini.google.com/app/8ee99f90784bd9e8)
//! 3. [`tokio_uring` crate](https://docs.rs/tokio-uring/latest/tokio_uring/)
//! 4. [`io_uring` paper](https://kernel.dk/io_uring.pdf)
//! 5. [`futures_concurrency` crate, which works with any async runtime](https://docs.rs/futures-concurrency/latest/futures_concurrency/)
//!
//! ## Abstract
//!
//! You can write sequential code, that can be run sequentially, or in parallel (using
//! `thread::spawn()`). And there is concurrent code that can be run on a single thread or
//! multiple threads.
//!
//! Concurrency is a way to structure code into separate tasks. This does not define the
//! resources on a machine that will be used to run or execute tasks.
//!
//! Parallelism is a way to specify what resources (CPU cores, or threads) will be used on a
//! machine's operating system to run tasks.
//!
//! These 2 concepts are not the same. They are related but not the same.
//!
//! ## Concurrency on a single thread
//!
//! If you have async code, you can use a `LocalSet` to run the async code, in different
//! tasks, on a single thread. This ensures that any data that you have to pass between these
//! tasks can be `!Send`. Instead of wrapping the shared data in a `Arc` or `Arc<Mutex>`, you
//! can just wrap it in an `Rc`.
//!
//! - Look at the [local_set_tests] for more info.
//! - Read the docs [here](https://docs.rs/tokio/latest/tokio/task/struct.LocalSet.html).
//!
//! ## Concurrency on multiple threads
//!
//! TK: write more on this

/// An exploration of:
/// - [tokio::task::LocalSet]
/// - [tokio::task::spawn_local]
/// - [tokio::task::LocalSet::run_until]
#[cfg(test)]
pub mod local_set_tests {
    use crossterm::style::Stylize;
    use std::rc::Rc;
    use tokio::{task::LocalSet, time::sleep};

    /// Spawn local tasks that uses non-send data. This is not like spawning a task on
    /// another thread.
    #[tokio::test]
    async fn local_set_and_spawn_local() {
        // Can't send this data across threads (not wrapped in `Arc` or `Arc<Mutex>`).
        let non_send_data = Rc::new("!SEND DATA");
        let local_set = LocalSet::new();

        // Spawn a local task (bound to same thread) that uses the non-send data.
        let non_send_data_clone = non_send_data.clone();
        let async_block_1 = async move {
            println!(
                "{:<7} {}", // https://doc.rust-lang.org/std/fmt/index.html#fillalignment
                "start",
                non_send_data_clone.as_ref().yellow().bold(),
            );
        };
        let task_1 = local_set.spawn_local(async_block_1);
        local_set.run_until(task_1).await.ok(); // This is required to run `task_1`.

        // Create a 2nd async block.
        let non_send_data_clone = non_send_data.clone();
        let async_block_2 = async move {
            sleep(std::time::Duration::from_millis(100)).await;
            println!(
                "{:<7} {}", // https://doc.rust-lang.org/std/fmt/index.html#fillalignment
                "middle",
                non_send_data_clone.as_ref().green().bold()
            );
        };
        local_set.run_until(async_block_2).await; // This is required to run `async_block_2`.

        // Spawn another local task (bound to same thread) that uses the non-send data.
        let non_send_data_clone = non_send_data.clone();
        let async_block_3 = async move {
            sleep(std::time::Duration::from_millis(100)).await;
            println!(
                "{:<7} {}", // https://doc.rust-lang.org/std/fmt/index.html#fillalignment
                "end",
                non_send_data_clone.as_ref().cyan().bold()
            );
        };
        let _task_2 = local_set.spawn_local(async_block_3);

        // `_task_2` won't run until this is called.
        local_set.await;
    }
}
