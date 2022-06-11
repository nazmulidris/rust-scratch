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
use crossterm::terminal::*;
use r3bl_rs_utils::*;
use std::{
  fmt::{self, Debug},
  ops::SubAssign,
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
/// Size, defined as [height, width].
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Size {
  pub width: UnitType,  // number of cols (y).
  pub height: UnitType, // number of rows (x).
}

impl SubAssign<UnitType> for Size {
  fn sub_assign(
    &mut self,
    other: UnitType,
  ) {
    self.width -= other;
    self.height -= other;
  }
}

impl From<Pair> for Size {
  fn from(pair: Pair) -> Self {
    Self {
      width: pair.first,
      height: pair.second,
    }
  }
}

impl From<(UnitType, UnitType)> for Size {
  /// 1. First (pair.0) is width or cols.
  /// 2. Second (pair.1) is height or rows.
  fn from(pair: (UnitType, UnitType)) -> Self {
    Self {
      width: pair.0,
      height: pair.1,
    }
  }
}

impl From<(usize, usize)> for Size {
  fn from(pair: (usize, usize)) -> Self {
    Self {
      width: pair.0.try_into().unwrap_or(pair.0 as UnitType),
      height: pair.1.try_into().unwrap_or(pair.1 as UnitType),
    }
  }
}

impl From<(i32, i32)> for Size {
  fn from(pair: (i32, i32)) -> Self {
    Self {
      width: pair.0.try_into().unwrap_or(pair.0 as UnitType),
      height: pair.1.try_into().unwrap_or(pair.1 as UnitType),
    }
  }
}

impl Size {
  /// Create a new [Size] from [crossterm::terminal::size()].
  pub fn try_to_get_from_crossterm_terminal() -> CommonResult<Self> {
    let size: Self = size()?.into();
    Ok(size)
  }

  /// Return an `Option` with `self`.
  pub fn as_some(&self) -> Option<Self> {
    Some(*self)
  }
}

impl Debug for Size {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>,
  ) -> fmt::Result {
    write!(f, "[width:{}, height:{}]", self.width, self.height)
  }
}
