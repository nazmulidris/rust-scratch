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

//! Here is a visual representation of how position and sizing works for the layout
//! engine.
//!
//! ```text
//!     0   4    9    1    2    2
//!                   4    0    5
//!    ┌────┴────┴────┴────┴────┴──→ x
//!  0 ┤     ╭─────────────╮
//!  1 ┤     │ origin pos: │
//!  2 ┤     │ [5, 0]      │
//!  3 ┤     │ size:       │
//!  4 ┤     │ [16, 5]     │
//!  5 ┤     ╰─────────────╯
//!    ↓
//!    y
//! ```

use std::ops::{Add, Mul};

/// Maps to whatever base units `crossterm` uses.
type BoxUnit = u16;

/// Position, defined as [x, y].
#[derive(Copy, Clone, Debug, Default)]
pub struct BoxPosition {
  pub x: BoxUnit,
  pub y: BoxUnit,
}

impl BoxPosition {
  pub fn new(
    x: BoxUnit,
    y: BoxUnit,
  ) -> BoxPosition {
    BoxPosition { x, y }
  }
}

/// Size, defined as [height, width].
#[derive(Copy, Clone, Debug, Default)]
pub struct BoxSize {
  pub width: BoxUnit,  // number of cols (y).
  pub height: BoxUnit, // number of rows (x).
}

impl BoxSize {
  pub fn new(
    width: BoxUnit,
    height: BoxUnit,
  ) -> BoxSize {
    BoxSize { height, width }
  }
}

/// Pair, defined as [left, right].
#[derive(Copy, Clone, Debug, Default)]
pub struct Pair {
  pub first: BoxUnit,
  pub second: BoxUnit,
}

impl Pair {
  pub fn new(
    first: BoxUnit,
    second: BoxUnit,
  ) -> Pair {
    Pair { first, second }
  }
}

/// Add: BoxPosition + BoxSize = BoxPosition.
/// https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
impl Add<BoxSize> for BoxPosition {
  type Output = BoxPosition;
  fn add(
    self,
    rhs: BoxSize,
  ) -> BoxPosition {
    BoxPosition::new(
      self.x + rhs.width,
      self.y + rhs.height,
    )
  }
}

/// Mul: BoxPosition * Pair = BoxPosition.
/// https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
impl Mul<Pair> for BoxPosition {
  type Output = BoxPosition;
  fn mul(
    self,
    rhs: Pair,
  ) -> BoxPosition {
    BoxPosition::new(
      self.x * rhs.first,
      self.y * rhs.second,
    )
  }
}
