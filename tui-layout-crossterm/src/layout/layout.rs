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
#[derive(Copy, Clone, Debug)]
pub enum Direction {
  Horiz,
  Vert,
}

impl Default for Direction {
  fn default() -> Direction {
    Direction::Horiz
  }
}

/// A box is a rectangle with a position and size. The direction of the box determines how
/// it's contained elements are positioned.
#[derive(Copy, Clone, Debug, Default)]
pub struct Layout {
  pub dir: Direction,
  pub pos: Option<Position>,
  pub content_size: Option<Size>,
  pub bounds_size: Option<Size>,
  pub width_hint_pc: Option<PerCent>, // TODO: use this to calc box size during layout
  pub height_hint_pc: Option<PerCent>, // TODO: use this to calc box size during layout
}

impl Layout {
  /// Explicitly set the position & size of our box.
  pub fn new_root(
    dir: Direction,
    pos: Position,
    size: Size,
  ) -> Self {
    Self {
      dir,
      pos: Some(pos),
      bounds_size: Some(size),
      content_size: None,
      width_hint_pc: None,
      height_hint_pc: None,
    }
  }

  /// Actual position and size for our box will be calculated based on provided hints.
  pub fn new(
    dir: Direction,
    width_pc: PerCent,
    height_pc: PerCent,
  ) -> Self {
    Self {
      dir,
      pos: None,
      bounds_size: None,
      content_size: None,
      width_hint_pc: Some(width_pc),
      height_hint_pc: Some(height_pc),
    }
  }
}
