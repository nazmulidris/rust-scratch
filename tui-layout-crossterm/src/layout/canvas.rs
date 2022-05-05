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
use r3bl_rs_utils::{with, ResultCommon};

/// Represents a rectangular area of the terminal screen, and not necessarily the full
/// terminal screen.
#[derive(Clone, Debug, Default)]
pub struct Canvas {
  pub origin_pos: Position,
  pub canvas_size: Size,
  pub layout_stack: Vec<Layout>,
  // TODO: impl this & collect pseudo "output commands" in self.output_commands for testing
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
    id: &str,
    dir: Direction,
    sizes_pc: (u8, u8),
  ) -> ResultCommon<()>;

  fn end_layout(&mut self) -> ResultCommon<()>;

  /// Painting operations.
  fn print(
    &mut self,
    text_vec: Vec<&str>,
  ) -> ResultCommon<()>;
}

#[derive(Clone, Debug, Default)]
struct LayoutProps {
  pub id: String,
  pub dir: Direction,
  pub req_size: RequestedSize,
}

impl LayoutManager for Canvas {
  fn start(
    &mut self,
    pos: (Unit, Unit),
    size: (Unit, Unit),
  ) -> ResultCommon<()> {
    // Expect layout_stack to be empty!
    if !self.is_layout_stack_empty() {
      LayoutError::new_err_with_msg(
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
    // Expect layout_stack to be empty!
    if !self.is_layout_stack_empty() {
      LayoutError::new_err_with_msg(
        LayoutErrorType::MismatchedEnd,
        LayoutError::format_msg_with_stack_len(
          &self.layout_stack,
          "Layout stack should be empty",
        ),
      )?
    }
    Ok(())
  }

  fn start_layout(
    &mut self,
    id: &str,
    dir: Direction,
    sizes_pc: (u8, u8),
  ) -> ResultCommon<()> {
    let (width_pc, height_pc) = unwrap_or_return_with_err! {
      convert_to_percent(sizes_pc),
      LayoutErrorType::InvalidLayoutSizePercentage
    };

    with! {
      LayoutProps {
        id: id.to_string(),
        dir,
        req_size: RequestedSize::new(width_pc, height_pc),
      },
      as it,
      run {
        match self.is_layout_stack_empty() {
          true => self.add_root_layout(it),
          false => self.add_normal_layout(it),
        }?;
      }
    }

    Ok(())
  }

  fn end_layout(&mut self) -> ResultCommon<()> {
    // Expect layout_stack not to be empty!
    if self.is_layout_stack_empty() {
      LayoutError::new_err_with_msg(
        LayoutErrorType::MismatchedEndLayout,
        LayoutError::format_msg_with_stack_len(
          &self.layout_stack,
          "Layout stack should not be empty",
        ),
      )?
    }
    self.pop_layout();
    Ok(())
  }

  fn print(
    &mut self,
    text_vec: Vec<&str>,
  ) -> ResultCommon<()> {
    with! {
      self.get_current_layout()?,
      as current_layout,
      run {
        let mut pos:Position = match current_layout.content_cursor_pos {
          Some(value) => value,
          None => Position::new(0, 0),
        };
        pos.y += text_vec.len() as Unit;
        current_layout.content_cursor_pos = Some(pos);
      }
    };
    Ok(())
  }
}

impl Canvas {
  fn is_layout_stack_empty(&self) -> bool {
    self.layout_stack.is_empty()
  }

  fn push_layout(
    &mut self,
    layout: Layout,
  ) {
    self.layout_stack.push(layout);
  }

  fn pop_layout(&mut self) {
    self.layout_stack.pop();
  }

  /// Calculate and return the position of where the next layout can be added to the
  /// stack. This updates the `layout_cursor_pos` of the current layout.
  fn calc_next_layout_cursor_pos(
    &mut self,
    allocated_size: Size,
  ) -> ResultCommon<Position> {
    let current_layout = self.get_current_layout()?;
    let layout_cursor_pos = current_layout.layout_cursor_pos;

    let layout_cursor_pos = unwrap_or_return_with_err! {
      layout_cursor_pos,
      LayoutErrorType::ErrorCalculatingNextLayoutPos
    };

    let new_pos: Position = layout_cursor_pos + allocated_size;

    // Adjust `new_pos` using Direction.
    let new_pos: Position = match current_layout.dir {
      Direction::Vertical => new_pos * Pair::new(0, 1),
      Direction::Horizontal => new_pos * Pair::new(1, 0),
    };

    Ok(new_pos)
  }

  fn update_layout_cursor_pos(
    &mut self,
    new_pos: Position,
  ) -> ResultCommon<()> {
    self
      .get_current_layout()?
      .layout_cursor_pos = new_pos.as_some();
    Ok(())
  }

  /// Get the last layout on the stack (if none found then return Err).
  fn get_current_layout(&mut self) -> ResultCommon<&mut Layout> {
    // Expect layout_stack not to be empty!
    if self.layout_stack.is_empty() {
      LayoutError::new_err(LayoutErrorType::LayoutStackShouldNotBeEmpty)?
    }
    Ok(
      self
        .layout_stack
        .last_mut()
        .unwrap(),
    )
  }

  /// 🌳 Root: Handle first layout to add to stack, explicitly sized & positioned.
  fn add_root_layout(
    &mut self,
    props: LayoutProps,
  ) -> ResultCommon<()> {
    let LayoutProps { id, dir, req_size } = props;
    let RequestedSize {
      width: width_pc,
      height: height_pc,
    } = req_size;
    self.push_layout(Layout::make_root_layout(
      id.to_string(),
      self.canvas_size,
      self.origin_pos,
      width_pc,
      height_pc,
      dir,
    ));
    Ok(())
  }

  /// 🍀 Non-root: Handle layout to add to stack. Position and Size will be calculated.
  fn add_normal_layout(
    &mut self,
    props: LayoutProps,
  ) -> ResultCommon<()> {
    let LayoutProps { id, dir, req_size } = props;
    let RequestedSize {
      width: width_pc,
      height: height_pc,
    } = req_size;
    let container_bounds = unwrap_or_return_with_err! {
      self.get_current_layout()?.bounds_size,
      LayoutErrorType::ContainerBoundsNotDefined
    };

    let requested_size_allocation = Size::new(
      calc_percentage(width_pc, container_bounds.width),
      calc_percentage(height_pc, container_bounds.height),
    );

    let old_position = unwrap_or_return_with_err! {
      self.get_current_layout()?.layout_cursor_pos,
      LayoutErrorType::LayoutCursorPositionNotDefined
    };

    let new_pos = self.calc_next_layout_cursor_pos(requested_size_allocation)?;
    self.update_layout_cursor_pos(new_pos)?;

    self.push_layout(Layout::make_layout(
      id.to_string(),
      dir,
      container_bounds,
      old_position,
      width_pc,
      height_pc,
    ));
    Ok(())
  }
}
