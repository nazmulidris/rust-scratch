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

use crate::LayoutError;
use crate::LayoutErrorType;
use r3bl_rs_utils::unwrap_option_or_run_fn_returning_err;
use r3bl_rs_utils::ResultCommon;
use std::{
  fmt::{self, Debug},
  ops::{Add, Mul},
};

/// Maps to whatever base units `crossterm` uses.
pub type UnitType = u16;

/// Pair, defined as [left, right].
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Pair {
  pub first: UnitType,
  pub second: UnitType,
}

impl Pair {
  // Wrap given values as `Pair`.
  pub fn new(
    first: UnitType,
    second: UnitType,
  ) -> Self {
    Self { first, second }
  }
}

impl Debug for Pair {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>,
  ) -> fmt::Result {
    write!(
      f,
      "Pair [first:{}, second:{}]",
      self.first, self.second
    )
  }
}

/// Position, defined as [x, y].
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Position {
  pub x: UnitType,
  pub y: UnitType,
}

impl Position {
  /// Convert given `i32` tuple value to `Position` struct.
  pub fn from_pair(pair: Pair) -> Self {
    Self {
      x: pair.first,
      y: pair.second,
    }
  }

  /// Wrap given values as `Position`.
  pub fn new(
    x: UnitType,
    y: UnitType,
  ) -> Self {
    Self { x, y }
  }

  /// Return an `Option` with `self`.
  pub fn as_some(&self) -> Option<Self> {
    Some(*self)
  }

  /// Add given `x` value to `self`.
  pub fn add_x(
    &mut self,
    value: usize,
  ) {
    let value: UnitType = value as UnitType;
    self.x = self.x + value;
  }

  /// Add given `y` value to `self`.
  pub fn add_y(
    &mut self,
    value: usize,
  ) {
    let value = value as UnitType;
    self.y = self.y + value;
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

/// Size, defined as [height, width].
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Size {
  pub width: UnitType,  // number of cols (y).
  pub height: UnitType, // number of rows (x).
}

impl Size {
  /// Convert given `Unit` tuple value to `Size` struct.
  pub fn from_pair(pair: Pair) -> Self {
    Self {
      width: pair.first,
      height: pair.second,
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
    write!(
      f,
      "[width:{}, height:{}]",
      self.width, self.height
    )
  }
}

/// Add: BoxPosition + BoxSize = BoxPosition.
/// https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
impl Add<Size> for Position {
  type Output = Position;
  fn add(
    self,
    rhs: Size,
  ) -> Self {
    Self::new(
      self.x + rhs.width,
      self.y + rhs.height,
    )
  }
}

/// Mul: BoxPosition * Pair = BoxPosition.
/// https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
impl Mul<Pair> for Position {
  type Output = Position;
  fn mul(
    self,
    rhs: Pair,
  ) -> Self {
    Self::new(
      self.x * rhs.first,
      self.y * rhs.second,
    )
  }
}

/// Represents an integer value between 0 and 100 (inclusive).
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct Percent {
  pub value: u8,
}

impl fmt::Display for Percent {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>,
  ) -> fmt::Result {
    write!(f, "{}%", self.value)
  }
}

impl Debug for Percent {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>,
  ) -> fmt::Result {
    write!(f, "PerCent value:{}%", self.value)
  }
}

impl Percent {
  /// Try and convert given `Pair` into a `(Percent, Percent)`. Return
  /// `InvalidLayoutSizePercentage` error if given values are not between 0 and 100.
  pub fn parse_pair(pair: Pair) -> ResultCommon<(Percent, Percent)> {
    let first = Percent::from(pair.first.into());
    let second = Percent::from(pair.second.into());

    if first.is_none() || second.is_none() {
      let err_msg = format!(
        "Invalid percentage values in tuple: {:?}",
        pair
      );
      return LayoutError::new_err_with_msg(
        LayoutErrorType::InvalidLayoutSizePercentage,
        err_msg,
      );
    }

    return Ok((first.unwrap(), second.unwrap()));
  }

  /// Try and convert given `i32` value to `Percent`. Return `InvalidLayoutSizePercentage`
  /// error if given value is not between 0 and 100.
  pub fn parse(item: i32) -> ResultCommon<Percent> {
    let value = unwrap_option_or_run_fn_returning_err!(Percent::from(item), || {
      let err_msg = format!(
        "Invalid percentage value: {}",
        item
      );
      return LayoutError::new_err_with_msg(
        LayoutErrorType::InvalidLayoutSizePercentage,
        err_msg,
      );
    });
    return Ok(value);
  }

  /// Try and convert given `UnitType` value to `Percent`. Return `None` if given value is
  /// not between 0 and 100.
  pub fn from(item: i32) -> Option<Percent> {
    if item < 0 || item > 100 {
      return None;
    }
    return Some(Percent { value: item as u8 });
  }

  /// Wrap `self` in `Option`.
  pub fn as_some(&self) -> Option<Percent> {
    Some(*self)
  }
}

/// Return the calculated percentage of the given value.
pub fn calc_percentage(
  percentage: Percent,
  value: UnitType,
) -> UnitType {
  type Integer = UnitType;
  let percentage_int = percentage.value;
  let percentage_f32 = f32::from(percentage_int) / 100.0;
  let result_f32 = percentage_f32 * f32::from(value);
  let result_int = unsafe { result_f32.to_int_unchecked::<Integer>() };
  result_int
}

/// Size, defined as [height, width].
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct RequestedSizePercent {
  pub width: Percent,
  pub height: Percent,
}

impl RequestedSizePercent {
  /// Try and convert the two given numbers as percentages. Returns error if the parsing
  /// fails.
  pub fn parse_pair(pair: Pair) -> ResultCommon<RequestedSizePercent> {
    let (width_pc, height_pc) = Percent::parse_pair(pair)?;
    Ok(Self::new(width_pc, height_pc))
  }

  /// Wrap given values as `RequestedSizePercent`.
  pub fn new(
    width: Percent,
    height: Percent,
  ) -> Self {
    Self { height, width }
  }

  /// Wrap `self` in `Option`.
  pub fn as_some(&self) -> Option<Self> {
    Some(*self)
  }
}

impl Debug for RequestedSizePercent {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>,
  ) -> fmt::Result {
    write!(
      f,
      "[width:{}, height:{}]",
      self.width, self.height
    )
  }
}
