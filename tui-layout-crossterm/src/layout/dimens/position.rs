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
use std::{
  fmt::{self, Debug},
  ops::{Add, AddAssign, Mul},
};

/// Here is a visual representation of how position and sizing works for the layout
/// engine.
///
/// ```text
///     0   4    9    1    2    2
///                   4    0    5
///    ┌────┴────┴────┴────┴────┴──→ x
///  0 ┤     ╭─────────────╮
///  1 ┤     │ origin pos: │
///  2 ┤     │ [5, 0]      │
///  3 ┤     │ size:       │
///  4 ┤     │ [16, 5]     │
///  5 ┤     ╰─────────────╯
///    ↓
///    y
/// ```
///
/// Position, defined as [x, y].
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Position {
  pub x: UnitType,
  pub y: UnitType,
}

impl AddAssign<UnitType> for Position {
  fn add_assign(
    &mut self,
    other: UnitType,
  ) {
    self.x += other;
    self.y += other;
  }
}

impl From<Pair> for Position {
  fn from(pair: Pair) -> Self {
    Self {
      x: pair.first,
      y: pair.second,
    }
  }
}

impl From<(UnitType, UnitType)> for Position {
  fn from(pair: (UnitType, UnitType)) -> Self {
    Self { x: pair.0, y: pair.1 }
  }
}

impl Position {
  /// Return an `Option` with `self`.
  pub fn as_some(&self) -> Option<Self> {
    Some(*self)
  }

  /// Add given `x` value to `self`.
  pub fn add_x(
    &mut self,
    value: usize,
  ) -> Self {
    let value: UnitType = value as UnitType;
    self.x = self.x + value;
    *self
  }

  /// Add given `y` value to `self`.
  pub fn add_y(
    &mut self,
    value: usize,
  ) -> Self {
    let value = value as UnitType;
    self.y = self.y + value;
    *self
  }
}

impl Debug for Position {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>,
  ) -> fmt::Result {
    write!(f, "[x:{}, y:{}]", self.x, self.y)
  }
}

/// Add: BoxPosition + BoxSize = BoxPosition.
/// https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
impl Add<Size> for Position {
  type Output = Position;
  fn add(
    self,
    other: Size,
  ) -> Self {
    Self {
      x: self.x + other.width,
      y: self.y + other.height,
    }
  }
}

/// Mul: BoxPosition * Pair = BoxPosition.
/// https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
impl Mul<Pair> for Position {
  type Output = Position;
  fn mul(
    self,
    other: Pair,
  ) -> Self {
    Self {
      x: self.x * other.first,
      y: self.y * other.second,
    }
  }
}
