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

use crate::layout::*;
use crate::*;
use r3bl_rs_utils::*;

/// Represents a rectangular area of the terminal screen, and not necessarily the full
/// terminal screen.
#[derive(Clone, Debug, Default)]
pub struct Canvas {
  pub origin_pos: Position,
  pub canvas_size: Size,
  pub layout_stack: Vec<Layout>,
  pub stylesheet: Stylesheet,
  pub output_commands: Vec<String>,
}

impl LayoutManager for Canvas {
  fn canvas_start(&mut self, CanvasProps { pos, size }: CanvasProps) -> CommonResult<()> {
    throws!({
      // Expect layout_stack to be empty!
      if !self.layout_stack.is_empty() {
        LayoutError::new_err_with_msg(
          LayoutErrorType::MismatchedCanvasStart,
          LayoutError::format_msg_with_stack_len(&self.layout_stack, "Layout stack should be empty"),
        )?
      }
      self.origin_pos = pos;
      self.canvas_size = size;
    });
  }

  fn canvas_end(&mut self) -> CommonResult<()> {
    throws!({
      // Expect layout_stack to be empty!
      if !self.layout_stack.is_empty() {
        LayoutError::new_err_with_msg(
          LayoutErrorType::MismatchedCanvasEnd,
          LayoutError::format_msg_with_stack_len(&self.layout_stack, "Layout stack should be empty"),
        )?
      }
    });
  }

  fn layout_start(&mut self, layout_props: LayoutProps) -> CommonResult<()> {
    throws!({
      match self.layout_stack.is_empty() {
        true => self.add_root_layout(layout_props),
        false => self.add_layout(layout_props),
      }?
    });
  }

  fn layout_end(&mut self) -> CommonResult<()> {
    throws!({
      // Expect layout_stack not to be empty!
      if self.layout_stack.is_empty() {
        LayoutError::new_err_with_msg(
          LayoutErrorType::MismatchedLayoutEnd,
          LayoutError::format_msg_with_stack_len(&self.layout_stack, "Layout stack should not be empty"),
        )?
      }
      self.layout_stack.pop();
    });
  }

  fn paint(&mut self, text_vec: Vec<&str>) -> CommonResult<()> {
    throws!({ self.calc_where_to_insert_new_content_in_layout((0, text_vec.len()).into())? });
  }
}

impl PerformPositioningAndSizing for Canvas {
  /// 🌳 Root: Handle first layout to add to stack, explicitly sized & positioned.
  fn add_root_layout(
    &mut self,
    LayoutProps {
      id,
      dir,
      req_size: RequestedSizePercent {
        width: width_pc,
        height: height_pc,
      },
      styles,
    }: LayoutProps,
  ) -> CommonResult<()> {
    throws!({
      self.layout_stack.push(Layout::make_root_layout(
        id.to_string(),
        self.canvas_size,
        self.origin_pos,
        width_pc,
        height_pc,
        dir,
        Stylesheet::compute(styles),
      ));
    });
  }

  /// 🍀 Non-root: Handle layout to add to stack. [Position] and [Size] will be calculated.
  fn add_layout(
    &mut self,
    LayoutProps {
      id,
      dir,
      req_size: RequestedSizePercent {
        width: width_pc,
        height: height_pc,
      },
      styles,
    }: LayoutProps,
  ) -> CommonResult<()> {
    throws!({
      let current_layout = self.current_layout()?;

      let container_bounds = unwrap_or_err! {
        current_layout.bounds_size,
        LayoutErrorType::ContainerBoundsNotDefined
      };

      let requested_size_allocation = Size::from((
        calc_percentage(width_pc, container_bounds.width),
        calc_percentage(height_pc, container_bounds.height),
      ));

      let old_position = unwrap_or_err! {
        current_layout.layout_cursor_pos,
        LayoutErrorType::LayoutCursorPositionNotDefined
      };

      self.calc_where_to_insert_new_layout_in_canvas(requested_size_allocation)?;

      self.layout_stack.push(Layout::make_layout(
        id.to_string(),
        dir,
        container_bounds,
        old_position,
        width_pc,
        height_pc,
        Stylesheet::compute(styles),
      ));
    });
  }

  /// Must be called *before* the new [Layout] is added to the stack otherwise
  /// [LayoutErrorType::ErrorCalculatingNextLayoutPos] error is returned.
  ///
  /// This updates the `layout_cursor_pos` of the current [Layout].
  ///
  /// Returns the [Position] where the next [Layout] can be added to the stack.
  fn calc_where_to_insert_new_layout_in_canvas(&mut self, allocated_size: Size) -> CommonResult<Position> {
    let current_layout = self.current_layout()?;
    let layout_cursor_pos = current_layout.layout_cursor_pos;

    let layout_cursor_pos = unwrap_or_err! {
      layout_cursor_pos,
      LayoutErrorType::ErrorCalculatingNextLayoutPos
    };

    let new_pos: Position = layout_cursor_pos + allocated_size;

    // Adjust `new_pos` using Direction.
    let new_pos: Position = match current_layout.dir {
      Direction::Vertical => new_pos * (0, 1).into(),
      Direction::Horizontal => new_pos * (1, 0).into(),
    };

    // Update the layout_cursor_pos of the current layout.
    current_layout.layout_cursor_pos = new_pos.as_some();

    Ok(new_pos)
  }

  /// This updates the `content_cursor_pos` of the current [Layout].
  fn calc_where_to_insert_new_content_in_layout(&mut self, content_size: Size) -> CommonResult<()> {
    let current_layout = self.current_layout()?;

    let pos = unwrap_option_or_compute_if_none! {
      current_layout.content_cursor_pos,
      || (0, 0).into()
    };
    current_layout.content_cursor_pos = Some(pos + content_size);

    Ok(())
  }

  /// Get the last layout on the stack (if none found then return Err).
  fn current_layout(&mut self) -> CommonResult<&mut Layout> {
    // Expect layout_stack not to be empty!
    if self.layout_stack.is_empty() {
      LayoutError::new_err(LayoutErrorType::LayoutStackShouldNotBeEmpty)?
    }
    Ok(self.layout_stack.last_mut().unwrap())
  }
}
