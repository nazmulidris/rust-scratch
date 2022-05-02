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

// TODO: impl this & collect pseudo "output commands" in self.output_commands for testing
// TODO: impl all the todo!()s in this file

use crate::*;

impl LayoutManager for Canvas {
  fn start(
    &mut self,
    position: Position,
    size: Size,
  ) -> r3bl_rs_utils::ResultCommon<()> {
    // Expect layout_stack to be empty!
    if !self.layout_stack.is_empty() {
      return Err(LayoutError::new(
        LayoutErrorType::MismatchedStart,
        LayoutError::format_msg_with_stack_len(
          &self.layout_stack,
          "Layout stack should be empty",
        ),
      ));
    }
    self.origin = position;
    self.size = size;
    Ok(())
  }

  fn end(&mut self) -> r3bl_rs_utils::ResultCommon<()> {
    // Expect layout_stack to only have 1 element!
    if self.layout_stack.len() != 1 {
      return Err(LayoutError::new(
        LayoutErrorType::MismatchedEnd,
        LayoutError::format_msg_with_stack_len(
          &self.layout_stack,
          "Layout stack should only have 1 element",
        ),
      ));
    }
    self.layout_stack.pop();
    Ok(())
  }

  fn next_position(&mut self) -> r3bl_rs_utils::ResultCommon<Position> {
    // Expect layout_stack not to be empty!
    if !self.layout_stack.is_empty() {
      return Err(LayoutError::new(
        LayoutErrorType::LayoutStackUnderflow,
        LayoutError::format_msg_with_stack_len(
          &self.layout_stack,
          "Layout stack should not be empty",
        ),
      ));
    }
    let layout = self
      .layout_stack
      .last_mut()
      .unwrap();
    let new_pos = layout.position + layout.size;
    let direction_adj_pos = match layout.direction {
      Direction::Vert => new_pos * Pair::new(0, 1),
      Direction::Horiz => new_pos * Pair::new(1, 0),
    };
    Ok(direction_adj_pos)
  }

  fn start_box(
    &mut self,
    direction: Direction,
  ) -> r3bl_rs_utils::ResultCommon<()> {
    todo!()
  }

  fn end_box(&mut self) -> r3bl_rs_utils::ResultCommon<()> {
    todo!()
  }

  fn paint_text(
    &mut self,
    text: String,
  ) -> r3bl_rs_utils::ResultCommon<()> {
    todo!()
  }
}
