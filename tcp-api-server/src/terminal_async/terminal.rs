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

use crossterm::style::Stylize;
use futures_util::FutureExt;
use miette::IntoDiagnostic;
use rustyline_async::{Readline, ReadlineEvent, SharedWriter};
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Mutex;

// 01: this is probably the main entry point for the crate ... need docs & examples
#[derive(Clone)]
pub struct TerminalAsync {
    readline: Arc<Mutex<Readline>>,
    stdout: SharedWriter,
}

impl TerminalAsync {
    /// Create a new instance of `TerminalAsync`. Example of `prompt` is `"> "`.
    pub fn try_new(prompt: &str) -> miette::Result<TerminalAsync> {
        let (readline, stdout) = Readline::new(prompt.to_owned()).into_diagnostic()?;
        Ok(TerminalAsync {
            readline: Arc::new(Mutex::new(readline)),
            stdout,
        })
    }

    pub fn clone_stdout(&self) -> SharedWriter {
        self.stdout.clone()
    }

    /// Replacement for [std::io::Stdin::read_line()] (this is async and non blocking).
    pub async fn get_readline_event(&mut self) -> miette::Result<ReadlineEvent> {
        let mut readline = self.readline.lock().await;
        readline.readline().fuse().await.into_diagnostic()
    }

    /// Don't change the `content`. Print it as is. This works concurrently and is async
    /// and non blocking. And it is compatible w/ the
    /// [get_readline_event](TerminalAsync::get_readline_event) method.
    pub async fn println_raw_and_flush<T>(&mut self, content: T)
    where
        T: std::fmt::Display,
    {
        let _ = writeln!(self.stdout, "{}", content);
        self.flush().await;
    }

    /// Don't change the `content`. Print it as is.
    pub async fn print_raw_and_flush<T>(&mut self, content: T)
    where
        T: std::fmt::Display,
    {
        let _ = write!(self.stdout, "{}", content);
        self.flush().await;
    }

    /// Prefix the `content` with a color and special characters, then print it.
    pub async fn print_output_and_flush<T>(&mut self, content: T)
    where
        T: std::fmt::Display,
    {
        let _ = writeln!(
            self.stdout,
            "{} {}",
            " > ".red().bold().on_dark_grey(),
            content
        );
        self.flush().await;
    }

    /// Simply flush the buffer. If there's a newline in the buffer, it will be printed.
    /// Otherwise it won't.
    pub async fn flush(&mut self) {
        let _ = self.readline.lock().await.flush();
    }
}
