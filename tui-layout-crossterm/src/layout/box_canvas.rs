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

// TODO: impl this & collect pseudo "output commands" in self.output_commands for testing

impl LayoutManager for Canvas {
  fn start(
    &mut self,
    position: Position,
    size: Size,
  ) -> r3bl_rs_utils::ResultCommon<()> {
    todo!()
  }

  fn end(&mut self) -> r3bl_rs_utils::ResultCommon<()> {
    todo!()
  }

  fn start_box(
    &mut self,
    orientation: Orientation,
  ) -> r3bl_rs_utils::ResultCommon<()> {
    todo!()
  }

  fn end_box(&mut self) -> r3bl_rs_utils::ResultCommon<()> {
    todo!()
  }

  fn next_position() -> r3bl_rs_utils::ResultCommon<Position> {
    todo!()
  }

  fn paint_text(text: String) -> r3bl_rs_utils::ResultCommon<()> {
    todo!()
  }
}
