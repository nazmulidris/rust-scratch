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
use r3bl_rs_utils::{debug, ResultCommon};

/// Represents a rectangular area of the terminal screen, and not necessarily the full
/// terminal screen.
#[derive(Clone, Debug, Default)]
pub struct Canvas {
  pub origin_pos: Position,
  pub canvas_size: Size,
  pub layout_stack: Vec<Layout>,
  pub output_commands: Vec<String>, // TODO: String is a placeholder for now, replace w/ enum
}

/// API interface to create nested & responsive layout based UIs.
pub trait LayoutManager {
  /// Set the origin pos (x, y) & canvas size (width, height) of our box (container).
  fn start(
    &mut self,
    pos: (Unit, Unit),
    size: (Unit, Unit),
  ) -> ResultCommon<()>;

  fn end(&mut self) -> ResultCommon<()>;

  /// Add a new layout on the stack w/ the direction & (width, height) percentages.
  fn start_layout(
    &mut self,
    dir: Direction,
    sizes_pc: (u8, u8),
  ) -> ResultCommon<()>;

  fn end_layout(&mut self) -> ResultCommon<()>;

  /// Painting operations.
  fn print(
    &mut self,
    text: &str,
  ) -> ResultCommon<()>;
}

impl LayoutManager for Canvas {
  fn start(
    &mut self,
    pos: (Unit, Unit),
    size: (Unit, Unit),
  ) -> ResultCommon<()> {
    // Expect layout_stack to be empty!
    if !self.layout_stack.is_empty() {
      LayoutError::new_err(
        LayoutErrorType::MismatchedStart,
        LayoutError::format_msg_with_stack_len(
          &self.layout_stack,
          "Layout stack should be empty",
        ),
      )?
    }
    self.origin_pos = Position::new(pos.0, pos.1);
    self.canvas_size = Size::new(size.0, size.1);
    Ok(())
  }

  fn end(&mut self) -> ResultCommon<()> {
    // Expect layout_stack to only have 1 element!
    if self.layout_stack.len() != 1 {
      LayoutError::new_err(
        LayoutErrorType::MismatchedEnd,
        LayoutError::format_msg_with_stack_len(
          &self.layout_stack,
          "Layout stack should only have 1 element",
        ),
      )?
    }
    self.layout_stack.pop();
    Ok(())
  }

  fn start_layout(
    &mut self,
    dir: Direction,
    sizes_pc: (u8, u8),
  ) -> ResultCommon<()> {
    debug!(self);

    let (width_pc, height_pc) = match convert_to_percent(sizes_pc) {
      Some(result) => result,
      None => LayoutError::new_err(
        LayoutErrorType::InvalidLayoutSizePercentage,
        LayoutError::format_msg_with_stack_len(
          &self.layout_stack,
          "Invalid layout size percentages",
        ),
      )?,
    };

    // 🌳 Root: Handle first layout to add to stack, explicitly sized & positioned.
    if self.layout_stack.is_empty() {
      self
        .layout_stack
        .push(Layout::make_root_layout(
          self.canvas_size,
          self.origin_pos,
          width_pc,
          height_pc,
          dir,
        ));
      return Ok(());
    }

    // TODO:
    // 🍀 Non-root: Handle layout to add to stack. Position and size will be calculated.
    todo!()
  }

  // TODO:
  fn end_layout(&mut self) -> ResultCommon<()> {
    todo!()
  }

  // TODO:
  fn print(
    &mut self,
    text: &str,
  ) -> ResultCommon<()> {
    todo!()
  }
}

impl Canvas {
  // TODO:
  /// Calculate the position of where the next layout can be added to the stack.
  fn calc_next_layout_pos_on_stack(
    &mut self,
    err_msg: &str,
  ) -> ResultCommon<Position> {
    todo!();
    // let layout = self.get_current_layout(err_msg)?;
    // let new_pos: Position = layout.position + layout.content_size;
    // let direction_adj_pos: Position = match layout.direction {
    //   Direction::Vert => new_pos * Pair::new(0, 1),
    //   Direction::Horiz => new_pos * Pair::new(1, 0),
    // };
    // Ok(direction_adj_pos)
  }

  /// Get the last layout on the stack (if none found then return Err).
  fn get_current_layout(
    &mut self,
    err_msg: &str,
  ) -> ResultCommon<&mut Layout> {
    // Expect layout_stack not to be empty!
    if self.layout_stack.is_empty() {
      LayoutError::new_err(
        LayoutErrorType::LayoutStackShouldNotBeEmpty,
        LayoutError::format_msg_with_stack_len(&self.layout_stack, &err_msg),
      )?
    }
    Ok(
      self
        .layout_stack
        .last_mut()
        .unwrap(),
    )
  }

  // TODO:
  fn alloc_space_for_print(
    &mut self,
    size: Size,
  ) -> ResultCommon<()> {
    todo!()
  }
}
