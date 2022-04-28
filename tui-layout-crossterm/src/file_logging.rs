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

use log::{LevelFilter, info, warn, trace, error};
use std::{error::Error, sync::Once, io::Error as IoError};

const FILE_PATH: &str = "log.txt";

static mut FILE_LOGGER_INIT: bool = false;
static INIT_ONCE: Once = Once::new();

/// Simply open the [`FILE_PATH`] file and write the log message to it. This will be
/// opened once per session (i.e. program execution). It is destructively opened, meaning
/// that it will be rewritten when used in the next session.
pub fn init_file_logger() -> Result<bool, Box<dyn Error>> {
  unsafe {
    INIT_ONCE.call_once(|| {
      match simple_logging::log_to_file(FILE_PATH, LevelFilter::max()) {
        Ok(_) => FILE_LOGGER_INIT = true,
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
  (TRACE, $($arg:tt)*) => {{
    init_file_logger()?;
    trace!($($arg)*);
    return Ok(())
  }};
  (ERROR, $($arg:tt)*) => {{
    init_file_logger()?;
    error!($($arg)*);
    return Ok(())
  }};
}

pub fn info(msg: &str) -> Result<(), Box<dyn Error>> {
  log!(INFO, "{}", msg);
}

pub fn warn(msg: &str) -> Result<(), Box<dyn Error>> {
  log!(WARN, "{}", msg);
}

pub fn trace(msg: &str) -> Result<(), Box<dyn Error>> {
  log!(TRACE, "{}", msg);
}

pub fn error(msg: &str) -> Result<(), Box<dyn Error>> {
  log!(ERROR, "{}", msg);
}
