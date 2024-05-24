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

//! # References:
//! - <https://tokio.rs/blog/2021-07-tokio-uring>
//! - <https://github.com/tokio-rs/tokio-uring>
//! - <https://www.scylladb.com/2020/05/05/how-io_uring-and-ebpf-will-revolutionize-programming-in-linux/0/>
//! - <https://www.datadoghq.com/blog/engineering/introducing-glommio/>
//! - <https://lore.kernel.org/io-uring/4af91b50-4a9c-8a16-9470-a51430bd7733@kernel.dk/T/#u>

use crossterm::style::Stylize;
use miette::IntoDiagnostic;
use std::path::Path;
use tokio_uring::fs::File;

fn main() -> miette::Result<()> {
    tokio_uring::start(read_file("Cargo.toml"))?;
    Ok(())
}

async fn read_file(name: impl AsRef<Path>) -> miette::Result<()> {
    let file = File::open(name).await.into_diagnostic()?;

    let buf_move = vec![0; 4096];

    // Read some data, the buffer is passed by ownership and submitted to the kernel. When
    // the operation completes, we get the buffer back.
    let (result, buf_from_kernel) = file.read_at(buf_move, 0).await;
    let bytes_read = result.into_diagnostic()?;

    println!(
        "{}",
        format!("Read {} bytes", bytes_read)
            .yellow()
            .underlined()
            .bold()
    );

    println!(
        "{}\n{}",
        "Data (bytes):".yellow().bold().underlined(),
        format!("{:?}", &buf_from_kernel[..bytes_read])
            .blue()
            .bold()
    );

    println!(
        "{}\n{}",
        "Data (string):".yellow().bold().underlined(),
        String::from_utf8_lossy(&buf_from_kernel[..bytes_read])
            .cyan()
            .bold()
    );

    Ok(())
}
