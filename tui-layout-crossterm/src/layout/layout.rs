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

use crate::dimens::*;

/// Direction of the layout of the box.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
  Horizontal,
  Vertical,
}

impl Default for Direction {
  fn default() -> Direction {
    Direction::Horizontal
  }
}

/// A box is a rectangle with a position and size. The direction of the box determines how
/// it's contained elements are positioned.
#[derive(Clone, Debug, Default)]
pub struct Layout {
  pub id: String,
  pub dir: Direction,
  pub origin_pos: Option<Position>,
  pub bounds_size: Option<Size>,
  pub req_size_pc: Option<RequestedSize>,
  pub layout_cursor_pos: Option<Position>,
  pub content_cursor_pos: Option<Position>,
}

impl Layout {
  /// Explicitly set the position & size of our box.
  pub fn make_root_layout(
    id: String,
    canvas_size: Size,
    origin_pos: Position,
    width_pc: PerCent,
    height_pc: PerCent,
    dir: Direction,
  ) -> Layout {
    let bounds_width = calc_percentage(width_pc, canvas_size.width);
    let bounds_height = calc_percentage(height_pc, canvas_size.height);
    Self {
      id,
      dir,
      origin_pos: origin_pos.as_some(),
      bounds_size: Size::new(bounds_width, bounds_height).as_some(),
      req_size_pc: RequestedSize::new(width_pc, height_pc).as_some(),
      layout_cursor_pos: origin_pos.as_some(),
      content_cursor_pos: None,
    }
  }

  /// Actual position and size for our box will be calculated based on provided hints.
  pub fn make_layout(
    id: String,
    dir: Direction,
    container_bounds: Size,
    origin_pos: Position,
    width_pc: PerCent,
    height_pc: PerCent,
  ) -> Self {
    let bounds_width = calc_percentage(width_pc, container_bounds.width);
    let bounds_height = calc_percentage(height_pc, container_bounds.height);
    Self {
      id,
      dir,
      origin_pos: origin_pos.as_some(),
      bounds_size: Size::new(bounds_width, bounds_height).as_some(),
      req_size_pc: RequestedSize::new(width_pc, height_pc).as_some(),
      layout_cursor_pos: None,
      content_cursor_pos: None,
    }
  }
}
