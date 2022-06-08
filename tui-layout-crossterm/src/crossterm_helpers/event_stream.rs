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
use async_trait::async_trait;
use crossterm::event::*;
use futures_util::{FutureExt, StreamExt};
use r3bl_rs_utils::*;

#[async_trait]
pub trait EventStreamExt {
  /// Read an [Event] from the [EventStream]. This is a non-blocking call. It returns an
  /// [InputEvent] wrapped in a [Option].
  async fn get_input_event(&mut self) -> CommonResult<Option<InputEvent>>;
}

#[async_trait]
impl EventStreamExt for EventStream {
  async fn get_input_event(&mut self) -> CommonResult<Option<InputEvent>> {
    let option_result_event = self.next().fuse().await;
    match option_result_event {
      Some(Ok(event)) => Ok(Some(event.into())),

      Some(Err(e)) => {
        log!(ERROR, "Error: {:?}", e);
        Ok(None)
      }

      None => Ok(None),
    }
  }
}
