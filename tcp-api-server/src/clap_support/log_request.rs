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

use std::str::FromStr;

use r3bl_rs_utils_core::{DisplayPreference, WriterConfig};

/// Converts the parsed command line arguments into a [WriterConfig].
pub fn convert_args_into_writer_config(
    args: &[LogClapArg],
    tracing_log_file_path_and_prefix: String,
    display_preference: DisplayPreference,
) -> WriterConfig {
    let contains_file_writer = args.contains(&LogClapArg::File);
    let contains_stdout_writer = args.contains(&LogClapArg::Stdout);
    match (contains_file_writer, contains_stdout_writer) {
        (true, true) => {
            WriterConfig::DisplayAndFile(display_preference, tracing_log_file_path_and_prefix)
        }
        (true, false) => WriterConfig::File(tracing_log_file_path_and_prefix),
        (false, true) => WriterConfig::Display(display_preference),
        (false, false) => WriterConfig::None,
    }
}

/// Used to parse the command line arguments (provided by `clap` crate.
#[derive(Clone, Debug, PartialEq)]
pub enum LogClapArg {
    Stdout,
    File,
    None,
}

/// Handle converting parsed command line arguments (via `clap` crate) into a [LogArgClap].
impl FromStr for LogClapArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stdout" => Ok(LogClapArg::Stdout),
            "file" => Ok(LogClapArg::File),
            "none" => Ok(LogClapArg::None),
            "" => Ok(LogClapArg::None),
            _ => Err(format!("{} is not a valid tracing writer", s)),
        }
    }
}

/// Tests for LogArgClap FromStr.
#[cfg(test)]
mod test_from_str {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(LogClapArg::from_str("stdout").unwrap(), LogClapArg::Stdout);
        assert_eq!(LogClapArg::from_str("file").unwrap(), LogClapArg::File);
    }

    #[test]
    fn test_invalid_from_str() {
        assert!(LogClapArg::from_str("invalid").is_err());
    }
}
