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
use crate::{unwrap_or_return_with_err, LayoutErrorType};
use bounded_integer::bounded_integer;
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

bounded_integer! {
  /// Represents an integer value between 9 and 100 (inclusive).
  /// https://docs.rs/bounded-integer/latest/bounded_integer/index.html#
  pub struct PerCent { 0..101 }
}

impl PerCent {
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
  let percentage_int = Int::from(percentage);
  let percentage_f32 = f32::from(percentage_int) / 100.0;
  let result_f32 = percentage_f32 * f32::from(value);
  let result_int = unsafe { result_f32.to_int_unchecked::<Int>() };
  result_int
}

/// Try and convert the (width %: `u8`, height %: `u8`) into (`PerCent`, `PerCent`).
/// If this fails, return `None`.
pub fn convert_to_percent(sizes_pc: (u8, u8)) -> Option<(PerCent, PerCent)> {
  let width_pc: Option<PerCent> = PerCent::new(sizes_pc.0);
  let height_pc: Option<PerCent> = PerCent::new(sizes_pc.1);
  if width_pc.is_none() && height_pc.is_none() {
    return None;
  }
  let width_pc: PerCent = width_pc.unwrap();
  let height_pc: PerCent = height_pc.unwrap();
  Some((width_pc, height_pc))
}

/// Size, defined as [height, width].
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct RequestedSize {
  pub width: PerCent,
  pub height: PerCent,
}

impl RequestedSize {
  /// Try and parse the two given numbers as percentages. Throws error if the parsing
  /// fails.
  pub fn parse(
    width_percent: u8,
    height_percent: u8,
  ) -> ResultCommon<RequestedSize> {
    let size_tuple = (width_percent, height_percent);
    let (width_pc, height_pc) = unwrap_or_return_with_err! {
      convert_to_percent(size_tuple),
      LayoutErrorType::InvalidLayoutSizePercentage
    };
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
