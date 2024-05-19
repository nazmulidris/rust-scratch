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

//! ## Abstract
//!
//! In Rust, you can write sequential code, and concurrent code:
//! - Sequential code, can be run sequentially, or in parallel (using `thread::spawn()`).
//! - Concurrent code that can be run on a single thread or multiple threads.
//!
//! Concurrency is a way to structure code into separate tasks. This does not define the
//! resources on a machine that will be used to run or execute tasks.
//!
//! Parallelism is a way to specify what resources (CPU cores, or threads) will be used on a
//! machine's operating system to run tasks.
//!
//! These 2 concepts are not the same. They are related but not the same.
//!
//! ## What it is not
//!
//! You can't simply attach `async` as a prefix to a function, when you define it, and
//! postfix `.await` when you call it. In fact, if you don't have at least one `.await` in
//! your async function body, then it probably doesn't need to be async. This project is a
//! deep dive into what async code is, what Rust Futures are, along with what async
//! Runtimes are. Along with some common patterns and anti-patterns when thinking in async
//! Rust.
//!
//! ## References
//!
//! This project is an exploration of async concurrent and parallel programming in Rust.
//! The inspiration for this exploration is this blog post: [Tasks are the wrong
//! abstraction[(https://blog.yoshuawuyts.com/tasks-are-the-wrong-abstraction/#tasks-are-the-wrong-abstraction-for-concurrent-async-execution).
//!
//! Here are some other interesting links:
//! 1. [How futures work in runtimes w/ Context and Waker](https://gemini.google.com/app/7cdd1930f56e4b91)
//! 2. [`epoll` (linux), vs `io_uring` (linux), vs `iocp` (windows)](https://gemini.google.com/app/8ee99f90784bd9e8)
//! 3. [`tokio_uring` crate](https://docs.rs/tokio-uring/latest/tokio_uring/)
//! 4. [`io_uring` paper](https://kernel.dk/io_uring.pdf)
//! 5. [`futures_concurrency` crate, which works with any async runtime](https://docs.rs/futures-concurrency/latest/futures_concurrency/)

#[cfg(test)]
pub mod build_a_timer_future_using_waker;

#[cfg(test)]
pub mod build_an_executor_to_run_future;

#[cfg(test)]
pub mod local_set;

#[cfg(test)]
pub mod demo_join_select_spawn;

#[cfg(test)]
pub mod async_stream;

#[cfg(test)]
pub mod non_blocking_async_event_loops;
