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

use crossterm::{cursor::*, style::*, terminal::*, *};
use miette::miette;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::AbortHandle, time::interval};

use crate::TerminalAsync;

// 01: This needs a TerminalAsync instance to work. So this is not the main entry point for the crate.
pub struct ProgressBarAsync {
    pub tick_delay: Duration,
    pub message: String,
    pub abort_handle: Arc<Mutex<Option<AbortHandle>>>,
    pub terminal_async: TerminalAsync,
}

impl ProgressBarAsync {
    pub async fn finish_with_message(&mut self, message: &str) {
        // If task is running, abort it. And print "\n".
        if let Some(abort_handle) = self.abort_handle.lock().await.take() {
            let mut stdout = self.terminal_async.clone_stdout();
            abort_handle.abort();
            let _ = execute!(
                stdout,
                MoveUp(1),
                MoveToColumn(0),
                Clear(ClearType::CurrentLine),
                SetAttribute(Attribute::Bold),
                Print(format!("{}\n", message)),
                SetAttribute(Attribute::Reset),
            );
            self.terminal_async.flush().await;
        }
    }

    pub async fn try_start_task(&mut self) -> miette::Result<AbortHandle> {
        if self.abort_handle.lock().await.is_some() {
            return Err(miette!("Task is already running"));
        }

        let message = self.message.clone();
        let tick_delay = self.tick_delay;
        let abort_handle = self.abort_handle.clone();
        let mut stdout = self.terminal_async.clone_stdout();
        let mut terminal_async = self.terminal_async.clone();

        let join_handle = tokio::spawn(async move {
            let mut interval = interval(tick_delay);
            let mut count = 0;
            let message_clone = message.clone();

            loop {
                // If abort_handle is None (`finish_with_message()` has been called), then
                // break the loop.
                if abort_handle.lock().await.is_none() {
                    break;
                }

                interval.tick().await;
                count += 1;

                let output = format!("{}{}", message_clone.clone(), ".".repeat(count));

                // At least one tick has happened. So, move the cursor up and clear the line.
                if count > 0 {
                    let _ = execute!(
                        stdout,
                        MoveUp(1),
                        MoveToColumn(0),
                        Clear(ClearType::CurrentLine),
                    );
                }

                // Print the output. And make sure to terminate w/ a newline, so that the
                // output is printed.
                let _ = execute!(
                    stdout,
                    MoveToColumn(0),
                    SetAttribute(Attribute::Bold),
                    Print(format!("{}\n", output.clone())),
                    SetAttribute(Attribute::Reset),
                );

                terminal_async.flush().await;
            }
        });

        Ok(join_handle.abort_handle())
    }

    pub async fn try_new_and_start(
        message: String,
        tick_delay: Duration,
        terminal_async: TerminalAsync,
    ) -> miette::Result<ProgressBarAsync> {
        let mut bar = ProgressBarAsync {
            message,
            tick_delay,
            terminal_async,
            abort_handle: Arc::new(Mutex::new(None)),
        };

        // Start task and get the abort_handle.
        let abort_handle = bar.try_start_task().await?;

        // Save the abort_handle.
        *bar.abort_handle.lock().await = Some(abort_handle);

        Ok(bar)
    }
}
