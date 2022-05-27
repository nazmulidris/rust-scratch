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
use r3bl_rs_utils::{Builder, CommonResult};

/// Public API interface to create nested & responsive layout based UIs.
pub trait LayoutManager {
  /// Set the origin pos (x, y) & canvas size (width, height) of our box (container).
  fn start(
    &mut self,
    bounds_props: CanvasProps,
  ) -> CommonResult<()>;

  fn end(&mut self) -> CommonResult<()>;

  /// Add a new layout on the stack w/ the direction & (width, height) percentages.
  fn start_layout(
    &mut self,
    layout_props: LayoutProps,
  ) -> CommonResult<()>;

  fn end_layout(&mut self) -> CommonResult<()>;

  /// Painting operations.
  fn print(
    &mut self,
    text_vec: Vec<&str>,
  ) -> CommonResult<()>;
}

/// Internal (semi-private) methods that actually perform the layout and positioning.
pub(in crate::layout) trait PerformLayoutAndPositioning {
  fn calc_next_layout_cursor_pos(
    &mut self,
    allocated_size: Size,
  ) -> CommonResult<Position>;

  fn update_layout_cursor_pos(
    &mut self,
    new_pos: Position,
  ) -> CommonResult<()>;

  fn get_current_layout(&mut self) -> CommonResult<&mut Layout>;

  fn add_root_layout(
    &mut self,
    props: LayoutProps,
  ) -> CommonResult<()>;

  fn add_normal_layout(
    &mut self,
    props: LayoutProps,
  ) -> CommonResult<()>;
}

/// Properties that are needed to create a [Layout].
#[derive(Clone, Debug, Default, Builder)]
pub struct LayoutProps {
  pub id: String,
  pub dir: Direction,
  pub req_size: RequestedSizePercent,
  pub styles: Option<Vec<Style>>,
}

/// Properties that are needed to create a [Canvas].
#[derive(Clone, Debug, Default, Builder)]
pub struct CanvasProps {
  pub pos: Position,
  pub size: Size,
}
