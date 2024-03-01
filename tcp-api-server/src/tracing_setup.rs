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
use miette::IntoDiagnostic;
use std::{path::PathBuf, str::FromStr};
use tracing::info;
use tracing_subscriber::fmt::writer::MakeWriterExt;

/// Fields:
/// - `writers`: Vec<[tracing_writer_config::Writer]> - Zero or more writers to use for
///   tracing.
/// - `level`: [tracing_log_level_config::Level] - The log level to use for tracing.
/// - `tracing_log_file_path_and_prefix`: [String] - The file path and prefix to use for
///   the log file. Eg: `/tmp/tcp_api_server` or `tcp_api_server`.
#[derive(Clone, Debug, PartialEq)]
pub struct TracingConfig {
    pub writers: Vec<tracing_writer_config::Writer>,
    pub level: tracing::Level,
    pub tracing_log_file_path_and_prefix: String,
}

mod tracing_config_impl {
    use super::*;

    impl Default for TracingConfig {
        fn default() -> Self {
            Self {
                writers: vec![
                    tracing_writer_config::Writer::File,
                    tracing_writer_config::Writer::Stdout,
                ],
                level: tracing::Level::DEBUG,
                tracing_log_file_path_and_prefix: "tracing_log_file_debug".to_string(),
            }
        }
    }
}

/// Initialize the global tracing subscriber with the given writers, level, and file path.
///
///
/// More info:
/// - [Setup tracing](https://tokio.rs/tokio/topics/tracing)
/// - [Configure
///   subscriber](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html#configuration)
/// - [Rolling file appender](https://docs.rs/tracing-appender/latest/tracing_appender/)
/// - [Examples](https://github.com/tokio-rs/tracing/blob/master/examples/examples/appender-multifile.rs)
pub fn init(tracing_config: TracingConfig) -> miette::Result<()> {
    let TracingConfig {
        writers,
        level,
        tracing_log_file_path_and_prefix,
    } = tracing_config;

    if writers.is_empty() {
        return Ok(());
    }

    let builder = tracing_subscriber::fmt()
        // .pretty() /* multi line pretty output */
        .with_max_level(level)
        .compact()
        .without_time()
        .with_thread_ids(true)
        .with_ansi(true)
        .with_target(false)
        .with_file(false)
        .with_line_number(false);

    // Both file & stdout writer.
    if writers.contains(&tracing_writer_config::Writer::File)
        & writers.contains(&tracing_writer_config::Writer::Stdout)
    {
        let rolling_file_appender =
            init_impl::try_create_rolling_file_appender(tracing_log_file_path_and_prefix.as_str())?;
        let writer = std::io::stdout
            .with_max_level(level)
            .and(rolling_file_appender.with_max_level(level));
        let subscriber = builder.with_writer(writer).finish();
        tracing::subscriber::set_global_default(subscriber).into_diagnostic()?;
    }
    // File writer.
    else if writers.contains(&tracing_writer_config::Writer::File) {
        let rolling_file_appender =
            init_impl::try_create_rolling_file_appender(tracing_log_file_path_and_prefix.as_str())?;
        let subscriber = builder
            .with_writer(rolling_file_appender.with_max_level(level))
            .finish();
        tracing::subscriber::set_global_default(subscriber).into_diagnostic()?;
    }
    // Stdout writer.
    else if writers.contains(&tracing_writer_config::Writer::Stdout) {
        let subscriber = builder
            .with_writer(std::io::stdout.with_max_level(level))
            .finish();
        tracing::subscriber::set_global_default(subscriber).into_diagnostic()?;
    } else {
        let subscriber = builder.finish();
        tracing::subscriber::set_global_default(subscriber).into_diagnostic()?;
    }

    info!(
        "tracing enabled {}",
        format!(
            "{:?}, {:?}, {:?}",
            writers, level, tracing_log_file_path_and_prefix
        )
        .cyan()
        .bold()
    );

    Ok(())
}

mod init_impl {
    use super::*;

    /// Note that if you wrap this up in a non blocking writer, as shown here, it doesn't work:
    /// `tracing_appender::non_blocking(try_create_rolling_file_appender("foo")?);`
    pub fn try_create_rolling_file_appender(
        path_str: &str,
    ) -> miette::Result<tracing_appender::rolling::RollingFileAppender> {
        let path = PathBuf::from(&path_str);

        let parent = path.parent().ok_or_else(|| {
        miette::miette!(
            format!("Can't access current folder {}. It might not exist, or don't have required permissions.", path.display())
        )
    })?;

        let file_stem = path.file_stem().ok_or_else(|| {
            miette::miette!(format!(
            "Can't access file name {}. It might not exist, or don't have required permissions.",
            path.display()
        ))
        })?;

        Ok(tracing_appender::rolling::never(parent, file_stem))
    }
}

pub mod tracing_writer_config {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    pub enum Writer {
        Stdout,
        File,
    }

    impl FromStr for Writer {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "stdout" => Ok(Writer::Stdout),
                "file" => Ok(Writer::File),
                _ => Err(format!("{} is not a valid tracing writer", s)),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_from_str() {
            assert_eq!(Writer::from_str("stdout").unwrap(), Writer::Stdout);
            assert_eq!(Writer::from_str("file").unwrap(), Writer::File);
        }

        #[test]
        fn test_invalid_from_str() {
            assert!(Writer::from_str("invalid").is_err());
        }
    }
}
