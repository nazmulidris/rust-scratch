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

//! ## Concurrency on a single thread
//!
//! If you have async code, you can use a `LocalSet` to run the async code, in different
//! tasks, on a single thread. This ensures that any data that you have to pass between these
//! tasks can be `!Send`. Instead of wrapping the shared data in a `Arc` or `Arc<Mutex>`, you
//! can just wrap it in an `Rc`.
//!
//! - Look at the [local_set_tests] for more info.
//! - Read the docs [here](https://docs.rs/tokio/latest/tokio/task/struct.LocalSet.html).
//! An exploration of:
//! - [tokio::task::LocalSet]
//! - [tokio::task::spawn_local]
//! - [tokio::task::LocalSet::run_until]

use crossterm::style::Stylize;
use std::rc::Rc;
use tokio::{task::LocalSet, time::sleep};

/// Spawn local tasks that uses non-send data. This is not like spawning a task on
/// another thread.
#[tokio::test]
async fn run_local_set_and_spawn_local() {
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
    let join_handle_1 = local_set.spawn_local(async_block_1); // Does not run anything.
    let _it = local_set.run_until(join_handle_1).await; // This is required to run `async_block_1`.

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
    let _it = local_set.run_until(async_block_2).await; // This is required to run `async_block_2`.

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
    let _join_handle_3 = local_set.spawn_local(async_block_3); // Does not run anything.

    // `async_block_3` won't run until this is called.
    local_set.await;
}
