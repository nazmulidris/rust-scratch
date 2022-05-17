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
pub type Unit = u16;

/// Position, defined as [x, y].
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Position {
  pub x: Unit,
  pub y: Unit,
}

impl Position {
  pub fn new(
    x: Unit,
    y: Unit,
  ) -> Self {
    Self { x, y }
  }

  pub fn as_some(&self) -> Option<Self> {
    Some(*self)
  }
}

impl Debug for Position {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>,
  ) -> fmt::Result {
    write!(
      f,
      "Position [x:{}, y:{}]",
      self.x, self.y
    )
  }
}

/// Size, defined as [height, width].
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Size {
  pub width: Unit,  // number of cols (y).
  pub height: Unit, // number of rows (x).
}

impl Size {
  pub fn new(
    width: Unit,
    height: Unit,
  ) -> Self {
    Self { height, width }
  }

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
      "Size [width:{}, height:{}]",
      self.width, self.height
    )
  }
}

/// Pair, defined as [left, right].
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Pair {
  pub first: Unit,
  pub second: Unit,
}

impl Pair {
  pub fn new(
    first: Unit,
    second: Unit,
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
pub struct PerCent {
  pub value: u8,
}

impl fmt::Display for PerCent {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>,
  ) -> fmt::Result {
    write!(f, "{}%", self.value)
  }
}

impl Debug for PerCent {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>,
  ) -> fmt::Result {
    write!(f, "PerCent value:{}%", self.value)
  }
}

impl PerCent {
  pub fn parse_tuple(tuple: (i32, i32)) -> ResultCommon<(PerCent, PerCent)> {
    let (first, second) = tuple;

    let first = PerCent::from(first);
    let second = PerCent::from(second);

    if first.is_none() || second.is_none() {
      let err_msg = format!(
        "Invalid percentage values in tuple: {:?}",
        tuple
      );
      return LayoutError::new_err_with_msg(
        LayoutErrorType::InvalidLayoutSizePercentage,
        err_msg,
      );
    }

    return Ok((first.unwrap(), second.unwrap()));
  }

  pub fn parse(item: i32) -> ResultCommon<PerCent> {
    let value = unwrap_option_or_run_fn_returning_err!(PerCent::from(item), || {
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

  pub fn from(item: i32) -> Option<PerCent> {
    if item < 0 || item > 100 {
      return None;
    }
    return Some(PerCent { value: item as u8 });
  }

  pub fn as_some(&self) -> Option<PerCent> {
    Some(*self)
  }
}

/// Return the calculated percentage of the given value.
pub fn calc_percentage(
  percentage: PerCent,
  value: Unit,
) -> Unit {
  type Int = Unit;
  let percentage_int = percentage.value;
  let percentage_f32 = f32::from(percentage_int) / 100.0;
  let result_f32 = percentage_f32 * f32::from(value);
  let result_int = unsafe { result_f32.to_int_unchecked::<Int>() };
  result_int
}

/// Size, defined as [height, width].
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct RequestedSize {
  pub width: PerCent,
  pub height: PerCent,
}

impl RequestedSize {
  /// Try and parse the two given numbers as percentages. Returns error if the parsing
  /// fails.
  pub fn from(
    width_percent: i32,
    height_percent: i32,
  ) -> ResultCommon<RequestedSize> {
    let size_tuple = (width_percent, height_percent);
    let (width_pc, height_pc) = PerCent::parse_tuple(size_tuple)?;
    Ok(Self::new(width_pc, height_pc))
  }

  pub fn new(
    width: PerCent,
    height: PerCent,
  ) -> Self {
    Self { height, width }
  }

  pub fn as_some(&self) -> Option<Self> {
    Some(*self)
  }
}

impl Debug for RequestedSize {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>,
  ) -> fmt::Result {
    write!(
      f,
      "RequestedSize [width:{}%, height:{}%]",
      self.width, self.height
    )
  }
}
