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

use r3bl_rs_utils::ResultCommon;

use crate::*;

impl LayoutManager for Canvas {
  /// Explicitly set the origin position and size of our box (container).
  fn start(
    &mut self,
    origin_pos: Position,
    canvas_size: Size,
  ) -> ResultCommon<()> {
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
    self.origin_pos = origin_pos;
    self.canvas_size = canvas_size;
    Ok(())
  }

  fn end(&mut self) -> ResultCommon<()> {
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

  fn get_current_layout(
    &mut self,
    err_msg: &str,
  ) -> ResultCommon<&mut Layout> {
    // Expect layout_stack not to be empty!
    if self.layout_stack.is_empty() {
      return Err(LayoutError::new(
        LayoutErrorType::LayoutStackShouldNotBeEmpty,
        LayoutError::format_msg_with_stack_len(&self.layout_stack, &err_msg),
      ));
    }
    Ok(
      self
        .layout_stack
        .last_mut()
        .unwrap(),
    )
  }

  fn next_position(
    &mut self,
    err_msg: &str,
  ) -> ResultCommon<Position> {
    let layout = self.get_current_layout(err_msg)?;
    let new_pos = layout.pos + layout.size;
    let direction_adj_pos = match layout.direction {
      Direction::Vert => new_pos * Pair::new(0, 1),
      Direction::Horiz => new_pos * Pair::new(1, 0),
    };
    Ok(direction_adj_pos)
  }

  fn start_layout(
    &mut self,
    direction: Direction,
  ) -> ResultCommon<()> {
    // Handle first layout to add to stack, explicitly sized & positioned.
    if self.layout_stack.is_empty() {
      let root = Layout::new_root(
        direction,
        self.origin_pos,
        self.canvas_size,
      );
      self.layout_stack.push(root);
      return Ok(());
    }

    // Handle subsequent layout to add to stack. Position and size will be calculated.

    todo!()
  }

  fn end_layout(&mut self) -> ResultCommon<()> {
    todo!()
  }

  fn paint(
    &mut self,
    text: String,
  ) -> ResultCommon<()> {
    todo!()
  }

  fn alloc_space_for_paint(
    &mut self,
    size: Size,
  ) -> ResultCommon<()> {
    todo!()
  }
}
