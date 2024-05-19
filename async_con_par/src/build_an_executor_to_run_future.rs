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

//! ## Build an executor to run a future.
//!
//! This uses the `futures` crate to build an executor that can run a future.
//! From: <https://rust-lang.github.io/async-book/02_execution/04_executor.html>.

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
    /// Implement `wake` by sending this task back onto the task channel so that it will
    /// be polled again by the executor, since it is now ready.
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
    let (task_sender, task_receiver) = std::sync::mpsc::sync_channel(MAX_TASKS);
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
