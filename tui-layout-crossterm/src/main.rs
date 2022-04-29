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

use r3bl_rs_utils::{init_file_logger_once, log, ResultCommon};

fn main() {
  run().unwrap();
}

fn run() -> ResultCommon<()> {
  log!(INFO, "This is a info message");
  log!(WARN, "This is a warning message");
  log!(ERROR, "This is a error message");
  println!("Hello, world!");
  Ok(())
}
