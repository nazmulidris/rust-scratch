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
  ops::SubAssign,
};

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

impl Size {
  /// Convert given `Unit` tuple value to `Size` struct.
  pub fn from_pair(pair: Pair) -> Self {
    Self {
      width: pair.first,
      height: pair.second,
    }
  }

  pub fn from_usize(
    width: usize,
    height: usize,
  ) -> Self {
    Self {
      width: width as UnitType,
      height: height as UnitType,
    }
  }

  /// Wrap given values as `Size`.
  pub fn new(
    width: UnitType,
    height: UnitType,
  ) -> Self {
    Self { height, width }
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
