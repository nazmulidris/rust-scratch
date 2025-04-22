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

use std::time::Duration;

use futures_util::StreamExt as _;
use miette::IntoDiagnostic;
use r3bl_tui::test_fixtures::gen_input_stream_with_delay;
use r3bl_tui::{ok, PinnedInputStream};
use smallvec::smallvec;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt as _};

pub mod constants {
    pub const HOST: &str = "localhost";
    pub const PORT: u16 = 8080;
    pub const SERVER_NAME: &str = "r3bl.com";
}

/// Can't use [tokio::io::stdin] because it is blocking on a thread and can't be
/// terminated, which results in the program hanging when the user presses [Ctrl+C] and
/// then Enter. Without the Enter, the program will stay hung.
///
/// Please refer to the [tokio::io::stdin] documentation for more information. Here's a
/// excerpt:
/// - This handle is best used for non-interactive uses, such as when a file is piped into
///   the application.
/// - For technical reasons, stdin is implemented by using an ordinary blocking read on a
///   separate thread, and it is impossible to cancel that read.
/// - This can make shutdown of the runtime hang until the user presses enter. For
///   interactive uses, it is recommended to spawn a thread dedicated to user input and
///   use blocking IO directly in that thread.
///
/// In this function, a [PinnedInputStream] is used instead of reading user typed input
/// from [tokio::io::stdin]. It simulates user input typing "one", "two", "three", with a
/// delay in between each input.
pub async fn read_write<R, W>(mut reader: R, writer: W) -> miette::Result<()>
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
{
    let input_stream = gen_input_stream_with_delay(
        smallvec!["one\n", "two\n", "three\n", "\n"],
        Duration::from_millis(500),
    );

    let mut stdout = tokio::io::stdout();

    tokio::select! {
        _ = read_from_input_stream_until_empty_and_write_to_writer(input_stream, writer) => {},
        _ = tokio::io::copy(&mut reader, &mut stdout) => {},
    }

    ok!()
}

async fn read_from_input_stream_until_empty_and_write_to_writer<W: AsyncWrite + Unpin>(
    mut input_stream: PinnedInputStream<&str>,
    mut writer: W,
) -> miette::Result<()> {
    while let Some(item) = input_stream.next().await {
        writer.write_all(item.as_bytes()).await.into_diagnostic()?;
        writer.flush().await.into_diagnostic()?;
    }
    ok!()
}
