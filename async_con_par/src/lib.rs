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

/// ## Implementing the `Future` trait.
///
/// From: <https://rust-lang.github.io/async-book/02_execution/03_wakeups.html>.
/// - Build a timer that wakes up a task after a certain amount of time, to explore how
///   `Waker` works.
/// - For the sake of the example, we'll just spin up a new thread when the timer is
///   created, sleep for the required time, and then signal the timer future when the time
///   window has elapsed.
/// Links:
/// 1. https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html
/// 2. https://doc.rust-lang.org/std/future/trait.Future.html
/// 3. https://rust-lang.github.io/async-book/02_execution/05_io.html
pub mod build_a_timer_future_using_waker {
    use std::{
        future::Future,
        pin::Pin,
        sync::{Arc, Mutex},
        task::{Context, Poll, Waker},
        thread,
        time::Duration,
    };

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
                true => Poll::Ready(()),
                false => {
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
}

/// ## Build an executor to run a future.
///
/// This uses the `futures` crate to build an executor that can run a future.
/// From: <https://rust-lang.github.io/async-book/02_execution/04_executor.html>.
pub mod build_an_executor_to_run_future {
    use crossterm::style::Stylize;
    use futures::{
        future::{BoxFuture, FutureExt},
        task::{waker_ref, ArcWake},
    };
    use std::{
        future::Future,
        sync::{
            mpsc::{sync_channel, Receiver, SyncSender},
            Arc, Mutex,
        },
        task::Context,
    };

    pub struct Executor {
        pub task_receiver: Receiver<Arc<Task>>,
    }

    impl Executor {
        #[allow(clippy::while_let_loop)]
        pub fn run(&self) {
            // Remove task from receiver, or block if nothing available.

            loop {
                eprintln!("{}", "executor loop".to_string().red());

                // Remove the task from the receiver. If it is pending, then the ArcWaker
                // will add it back to the channel.
                match self.task_receiver.recv() {
                    Ok(arc_task) => {
                        eprintln!(
                            "{}",
                            "running task - start, got task from receiver"
                                .to_string()
                                .red()
                        );
                        let mut future_in_task = arc_task.future.lock().unwrap();
                        match future_in_task.take() {
                            Some(mut future) => {
                                let waker = waker_ref(&arc_task);
                                let context = &mut Context::from_waker(&waker);
                                let poll_result = future.as_mut().poll(context);
                                eprintln!(
                                    "{}",
                                    format!("poll_result: {:?}", poll_result).to_string().red()
                                );
                                if poll_result.is_pending() {
                                    // We're not done processing the future, so put it
                                    // back in its task to be run again in the future.
                                    *future_in_task = Some(future);
                                    eprintln!("{}", "putting task back in slot".to_string().red());
                                } else {
                                    eprintln!("{}", "task is done".to_string().red());
                                }
                            }
                            None => {
                                panic!("this never runs");
                            }
                        }
                        eprintln!("{}", "running task - end".to_string().red());
                    }
                    Err(_) => {
                        eprintln!("no more tasks to run, breaking out of loop");
                        break;
                    }
                }
            }
        }
    }

    #[derive(Clone)]
    pub struct Spawner {
        pub task_sender: SyncSender<Arc<Task>>,
    }

    impl Spawner {
        pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
            let pinned_boxed_future = future.boxed();
            let task = Arc::new(Task {
                future: Mutex::new(Some(pinned_boxed_future)),
                task_sender: self.task_sender.clone(),
            });
            eprintln!(
                "{}",
                "sending task to executor, adding to channel"
                    .to_string()
                    .blue()
            );
            self.task_sender
                .send(task)
                .expect("too many tasks in channel");
        }
    }

    pub struct Task {
        pub future: Mutex<Option<BoxFuture<'static, ()>>>,
        pub task_sender: SyncSender<Arc<Task>>,
    }

    impl ArcWake for Task {
        /// Implement `wake` by sending this task back onto the task channel so that it
        /// will be polled again by the executor.
        fn wake_by_ref(arc_self: &Arc<Self>) {
            let cloned = arc_self.clone();
            arc_self
                .task_sender
                .send(cloned)
                .expect("too many tasks in channel");
            eprintln!(
                "{}",
                "task woken up, added back to channel"
                    .to_string()
                    .underlined()
                    .green()
                    .bold()
            );
        }
    }

    pub fn new_executor_and_spawner() -> (Executor, Spawner) {
        const MAX_TASKS: usize = 10_000;
        let (task_sender, task_receiver) = sync_channel(MAX_TASKS);
        (Executor { task_receiver }, Spawner { task_sender })
    }

    #[test]
    fn run_executor_and_spawner() {
        use super::build_a_timer_future_using_waker::TimerFuture;

        let results = Arc::new(std::sync::Mutex::new(Vec::new()));

        let (executor, spawner) = new_executor_and_spawner();

        let results_clone = results.clone();
        spawner.spawn(async move {
            results_clone.lock().unwrap().push("hello, start timer!");
            TimerFuture::new(std::time::Duration::from_millis(10)).await;
            results_clone.lock().unwrap().push("bye, timer finished!");
        });

        drop(spawner);

        executor.run();

        assert_eq!(
            *results.lock().unwrap(),
            vec!["hello, start timer!", "bye, timer finished!"]
        );
    }
}

/// ## Concurrency on a single thread
///
/// If you have async code, you can use a `LocalSet` to run the async code, in different
/// tasks, on a single thread. This ensures that any data that you have to pass between these
/// tasks can be `!Send`. Instead of wrapping the shared data in a `Arc` or `Arc<Mutex>`, you
/// can just wrap it in an `Rc`.
///
/// - Look at the [local_set_tests] for more info.
/// - Read the docs [here](https://docs.rs/tokio/latest/tokio/task/struct.LocalSet.html).
/// An exploration of:
/// - [tokio::task::LocalSet]
/// - [tokio::task::spawn_local]
/// - [tokio::task::LocalSet::run_until]
#[cfg(test)]
pub mod local_set {
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
}

#[cfg(test)]
pub mod demo_join_select_spawn {
    use std::time::Duration;
    use tokio::time::sleep;

    pub async fn task_1() {
        sleep(Duration::from_millis(100)).await;
        println!("task_1");
    }

    pub async fn task_2() {
        sleep(Duration::from_millis(200)).await;
        println!("task_2");
    }

    pub async fn task_3() {
        sleep(Duration::from_millis(300)).await;
        println!("task_3");
    }

    #[tokio::test]
    async fn test_join() {
        tokio::join!(task_1(), task_2(), task_3());
        println!("all tasks done");
    }

    #[tokio::test]
    async fn test_select() {
        tokio::select! {
            _ = task_1() => println!("task_1 done"),
            _ = task_2() => println!("task_2 done"),
            _ = task_3() => println!("task_3 done"),
        }
        println!("all tasks done");
    }

    #[tokio::test]
    async fn test_spawn() {
        let handle_1 = tokio::spawn(task_1());
        let handle_2 = tokio::spawn(task_2());
        let handle_3 = tokio::spawn(task_3());

        handle_1.await.unwrap();
        handle_2.await.unwrap();
        handle_3.await.unwrap();
        println!("all tasks done");
    }
}

#[cfg(test)]
pub mod async_stream {
    use futures::Stream;
    use futures::StreamExt;
    use std::pin::Pin;
    pub type PinnedInputStream = Pin<Box<dyn Stream<Item = Result<String, String>>>>;

    pub fn gen_input_stream() -> PinnedInputStream {
        let it = async_stream::stream! {
            for event in get_input_vec() {
                yield Ok(event);
            }
        };
        Box::pin(it)
    }

    pub fn get_input_vec() -> Vec<String> {
        vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ]
    }

    #[tokio::test]
    async fn test_stream() {
        let mut count = 0;
        let mut it = gen_input_stream();
        while let Some(event) = it.next().await {
            let lhs = event.unwrap();
            let rhs = get_input_vec()[count].clone();
            assert_eq!(lhs, rhs);
            count += 1;
        }
    }
}

#[cfg(test)]
pub mod non_blocking_async_event_loops {
    /// More info: <https://docs.rs/tokio/latest/tokio/attr.test.html>
    #[tokio::test(flavor = "multi_thread", worker_threads = 5)]
    async fn test_main_loop() {
        // Register tracing subscriber.
        tracing_subscriber::fmt()
            .without_time()
            .compact()
            .with_target(false)
            .with_line_number(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .init();

        // Create channels for events and shutdown signals.
        let event_channel = tokio::sync::mpsc::channel::<String>(1_000);
        let (event_sender, mut event_receiver) = event_channel;

        let shutdown_channel = tokio::sync::broadcast::channel::<()>(1_000);
        let (shutdown_sender, _) = shutdown_channel;

        // Spawn the main event loop.
        let mut shutdown_receiver = shutdown_sender.subscribe();
        let safe_count: std::sync::Arc<std::sync::Mutex<usize>> = Default::default();
        let safe_count_clone = safe_count.clone();
        let join_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    event = event_receiver.recv() => {
                        tracing::info!(?event, "task got event: event");
                        let mut count = safe_count_clone.lock().unwrap();
                        *count += 1;
                    }
                    _ = shutdown_receiver.recv() => {
                        tracing::info!("task got shutdown signal");
                        break;
                    }
                }
            }
        });

        // Send events, in parallel.
        let mut handles = vec![];
        for i in 0..10 {
            let event_sender_clone = event_sender.clone();
            let join_handle = tokio::spawn(async move {
                tracing::info!(i, "sending event");
                let event = format!("event {}", i);
                let _ = event_sender_clone.send(event).await;
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            });
            handles.push(join_handle);
        }

        // Wait for all events to be sent using tokio.
        futures::future::join_all(handles).await;

        // Shutdown the event loops.
        shutdown_sender.send(()).unwrap();

        // Wait for the event loop to shutdown.
        join_handle.await.unwrap();

        // Assertions.
        assert_eq!(shutdown_sender.receiver_count(), 1);
        assert_eq!(*safe_count.lock().unwrap(), 10);
    }
}
