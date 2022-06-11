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

use crate::*;
use tui_layout_crossterm::{EventStreamExt, *};

const DEBUG: bool = true;

pub async fn run() -> CommonResult<()> {
  raw_mode!({
    let mut state = State::new()?;
    call_if_true!(DEBUG, state.dump_to_log("Startup"));

    loop {
      let maybe_input_event = state.event_stream.get_input_event().await;
      if let Some(input_event) = maybe_input_event {
        let loop_continuation = handle_input_event(input_event, &mut state).await;
        if let LoopContinuation::Exit = loop_continuation {
          break;
        }
      }
    }
  })
}
