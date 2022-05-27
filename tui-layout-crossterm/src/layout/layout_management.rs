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
  fn canvas_start(
    &mut self,
    bounds_props: CanvasProps,
  ) -> CommonResult<()>;

  fn canvas_end(&mut self) -> CommonResult<()>;

  /// Add a new layout on the stack w/ the direction & (width, height) percentages.
  fn layout_start(
    &mut self,
    layout_props: LayoutProps,
  ) -> CommonResult<()>;

  fn layout_end(&mut self) -> CommonResult<()>;

  /// Painting operations.
  fn paint(
    &mut self,
    text_vec: Vec<&str>,
  ) -> CommonResult<()>;
}

/// Internal (semi-private) methods that actually perform the layout and positioning.
pub(in crate::layout) trait PerformPositioningAndSizing {
  /// Update `content_cursor_pos`.
  fn calc_where_to_insert_new_content_in_layout(
    &mut self,
    pos: Size,
  ) -> CommonResult<()>;

  /// Update `layout_cursor_pos`. This needs to be called before adding a new [Layout].
  fn calc_where_to_insert_new_layout_in_canvas(
    &mut self,
    allocated_size: Size,
  ) -> CommonResult<Position>;

  /// Get the [Layout] at the "top" of the `layout_stack`.
  fn current_layout(&mut self) -> CommonResult<&mut Layout>;

  /// Add the first [Layout] to the [Canvas].
  /// 1. This one is explicitly sized.
  /// 2. there can be only one.
  fn add_root_layout(
    &mut self,
    props: LayoutProps,
  ) -> CommonResult<()>;

  /// Add non-root [Layout].
  fn add_layout(
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
