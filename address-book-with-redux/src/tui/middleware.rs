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

use r3bl_rs_utils::print_header;
use rand::Rng;

use crate::{
  address_book::Action,
  tui::{MIN_DELAY, MAX_DELAY},
};

pub fn logger_mw(action: Action) -> Option<Action> {
  // Artificial delay before calling the function.
  let delay_ms = rand::thread_rng().gen_range(MIN_DELAY..MAX_DELAY) as u64;
  std::thread::sleep(tokio::time::Duration::from_millis(delay_ms));

  // Log the action.
  println!("");
  print_header("middleware");
  println!("action: {:?}", action);
  None
}
