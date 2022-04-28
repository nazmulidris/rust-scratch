/*
 *   Copyright (c) 2022 Nazmul Idris
 *   All rights reserved.

 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at

 *   http://www.apache.org/licenses/LICENSE-2.0

 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
*/

use log::{LevelFilter, info, warn, error};
use std::{error::Error, sync::Once, io::Error as IoError};
use simplelog::*;
use std::fs::File;

const FILE_PATH: &str = "log.txt";

static mut FILE_LOGGER_INIT: bool = false;
static INIT_ONCE: Once = Once::new();

pub type MainResult<T> = std::result::Result<T, Box<dyn Error>>;

/// Simply open the [`FILE_PATH`] file and write the log message to it. This will be
/// opened once per session (i.e. program execution). It is destructively opened, meaning
/// that it will be rewritten when used in the next session.
///
/// # Docs
/// - [`CombinedLogger`], [`WriteLogger`], [`ConfigBuilder`]: https://github.com/drakulix/simplelog.rs
/// - [`format_description!`]: https://time-rs.github.io/book/api/format-description.html
pub fn init_file_logger() -> MainResult<bool> {
  unsafe {
    INIT_ONCE.call_once(|| {
      let file_result = File::create(FILE_PATH);
      match file_result {
        Ok(file) => {
          let config = match ConfigBuilder::new().set_time_offset_to_local() {
            Ok(builder) => {
              let formatted_time =
                format_description!("[hour repr:12]:[minute] [period]");
              builder.set_time_format_custom(formatted_time);
              // To use the default use instead: `builder.set_time_format_rfc2822();`
              builder.build()
            }
            Err(_) => Config::default(),
          };
          let log_open_result =
            CombinedLogger::init(vec![WriteLogger::new(LevelFilter::Info, config, file)]);
          match log_open_result {
            Ok(_) => FILE_LOGGER_INIT = true,
            Err(_) => FILE_LOGGER_INIT = false,
          }
        }
        Err(_) => FILE_LOGGER_INIT = false,
      }
    });
    match FILE_LOGGER_INIT {
      true => Ok(true),
      false => Err(Box::new(IoError::new(
        std::io::ErrorKind::Other,
        "Failed to initialize file logger",
      ))),
    }
  }
}

/// # Docs
/// - [`info!`], [`warn!`], [`error!`]: https://docs.rs/log/latest/log/
#[macro_export]
macro_rules! log {
  (INFO, $($arg:tt)*) => {{
    init_file_logger()?;
    info!($($arg)*);
    return Ok(())
  }};
  (WARN, $($arg:tt)*) => {{
    init_file_logger()?;
    warn!($($arg)*);
    return Ok(())
  }};
  (ERROR, $($arg:tt)*) => {{
    init_file_logger()?;
    error!($($arg)*);
    return Ok(())
  }};
}

pub fn info(msg: &str) -> MainResult<()> {
  log!(INFO, "{}", msg);
}

pub fn warn(msg: &str) -> MainResult<()> {
  log!(WARN, "{}", msg);
}

pub fn error(msg: &str) -> MainResult<()> {
  log!(ERROR, "{}", msg);
}
