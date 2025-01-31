/*
 *   Copyright (c) 2025 Nazmul Idris
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

//! [Pos] is a struct that holds the `row` and `col` indices of a character in a text
//! buffer. [RowIndex] (aka [Row]) and [ColIndex] (aka [Col]) are the types of the `row`
//! and `col` indices respectively. This ensures that it isn't possible to use a `col`
//! when you intended to use a `row` and vice versa. Also [ScrOfs] is an alias for [Pos],
//! since a scroll offset is just a position after all.
//!
//! Here is a visual representation of how position and sizing works for the layout
//! engine.
//!
//! ```text
//!     0   4    9    1    2    2
//!                   4    0    5
//!    ┌────┴────┴────┴────┴────┴── col
//!  0 ┤     ╭─────────────╮
//!  1 ┤     │ origin pos: │
//!  2 ┤     │ [5, 0]      │
//!  3 ┤     │ size:       │
//!  4 ┤     │ [16, 5]     │
//!  5 ┤     ╰─────────────╯
//!    │
//!   row
//! ```
//!
//! # The many ways to create one
//!
//! This API uses the `impl Into<struct>` pattern and [Add] `+` operator overloading to
//! allow for easy conversion between [r3bl_core::ChUnit] and [RowIndex]/[ColIndex].
//! - You can use [crate::pos()] function and pass it a [RowIndex] and [ColIndex] tuple,
//!   or pass a sequence of them with the [Add] `+` operator.
//! - Just using the [Add] `+` operator:
//!     - You can use [Add] to convert: [RowIndex] + [ColIndex], into: a [Pos].
//!     - You can use [Add] to convert: [ColIndex] + [RowIndex], into: a [Pos].
//!
//! # Examples
//!
//! ```rust
//! use r3bl_core::ch;
//! use unit_types::{Pos, RowIndex, ColIndex, row, col, pos, ScrOfs};
//!
//! // So many different ways to create a Pos.
//! let pos_1: Pos = pos(row(2) + col(3));
//! let pos_1: Pos = (row(2) + col(3)).into();
//! let pos_1: Pos = (row(2), col(3)).into();
//! let pos_1: Pos = (col(3), row(2)).into();
//!
//! // So many different ways to create a ScrOfs.
//! let scr_ofs_1: ScrOfs = pos(row(2) + col(3));
//! let scr_ofs_1: ScrOfs = (row(2) + col(3)).into();
//! let scr_ofs_1: ScrOfs = (row(2), col(3)).into();
//! let scr_ofs_1: ScrOfs = (col(3), row(2)).into();
//!
//! assert!(matches!(pos_1.row, RowIndex(_)));
//! assert!(matches!(pos_1.col, ColIndex(_)));
//! assert_eq!(*pos_1.row, ch(2));
//! assert_eq!(*pos_1.col, ch(3));
//!
//! let pos_a = pos(row(4) + col(10));
//! let pos_b = pos(row(2) + col(6));
//!
//! let pos_sum = pos_a + pos_b;
//! assert_eq!(*pos_sum.row, ch(6));
//! assert_eq!(*pos_sum.col, ch(16));
//!
//! let pos_diff = pos_a - pos_b;
//! assert_eq!(*pos_diff.row, ch(2));
//! assert_eq!(*pos_diff.col, ch(4));
//! ```

use crate::{ColIndex, ColWidthCount, Dim, RowHeightCount, RowIndex};
use std::ops::{AddAssign, SubAssign};
use std::{
    fmt::{Debug, Formatter, Result},
    ops::{Add, Mul, Sub},
};

/// Type aliases for better code readability.
pub type ScrOfs = Pos;
pub type Row = RowIndex;
pub type Col = ColIndex;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct Pos {
    /// Row index, 0 based.
    pub row: Row,
    /// Column index, 0 based.
    pub col: Col,
}

pub fn pos(arg: impl Into<Pos>) -> Pos {
    arg.into()
}

pub fn scr_ofs(arg: impl Into<Pos>) -> ScrOfs {
    arg.into()
}

mod constructor {
    use super::*;

    impl Pos {
        pub fn new(arg: impl Into<Pos>) -> Self {
            arg.into()
        }
    }

    impl From<(RowIndex, ColIndex)> for Pos {
        fn from((row, col): (RowIndex, ColIndex)) -> Self {
            Pos { row, col }
        }
    }

    impl From<(ColIndex, RowIndex)> for Pos {
        fn from((col, row): (ColIndex, RowIndex)) -> Self {
            Pos { row, col }
        }
    }

    impl Add<Col> for Row {
        type Output = Pos;

        fn add(self, rhs: Col) -> Self::Output {
            Pos {
                row: self,
                col: rhs,
            }
        }
    }

    impl Add<Row> for Col {
        type Output = Pos;

        fn add(self, rhs: Row) -> Self::Output {
            Pos {
                row: rhs,
                col: self,
            }
        }
    }
}

mod ops {
    use super::*;

    // Dim is equivalent to (ColWidthCount, RowHeightCount).
    impl Mul<Dim> for Pos {
        type Output = Pos;

        fn mul(self, rhs: Dim) -> Self::Output {
            Pos {
                row: self.row * rhs.height,
                col: self.col * rhs.width,
            }
        }
    }

    // (ColWidthCount, RowHeightCount) or (RowHeightCount, ColWidthCount) is equivalent to Dim.
    impl Mul<(ColWidthCount, RowHeightCount)> for Pos {
        type Output = Pos;

        fn mul(self, rhs: (ColWidthCount, RowHeightCount)) -> Self::Output {
            Pos {
                col: (self.col * rhs.0),
                row: (self.row * rhs.1).into(),
            }
        }
    }

    // (ColWidthCount, RowHeightCount) or (RowHeightCount, ColWidthCount) is equivalent to Dim.
    impl Mul<(RowHeightCount, ColWidthCount)> for Pos {
        type Output = Pos;

        fn mul(self, rhs: (RowHeightCount, ColWidthCount)) -> Self::Output {
            Pos {
                row: (*self.row * *rhs.0).into(),
                col: (*self.col * *rhs.1).into(),
            }
        }
    }

    impl Add<Dim> for Pos {
        type Output = Pos;

        fn add(self, rhs: Dim) -> Self::Output {
            Pos {
                row: self.row + rhs.height,
                col: self.col + rhs.width,
            }
        }
    }

    impl Sub<Dim> for Pos {
        type Output = Pos;

        fn sub(self, rhs: Dim) -> Self::Output {
            Pos {
                row: self.row - rhs.height,
                col: self.col - rhs.width,
            }
        }
    }

    impl AddAssign<Dim> for Pos {
        fn add_assign(&mut self, rhs: Dim) {
            *self = *self + rhs;
        }
    }

    impl SubAssign<Dim> for Pos {
        fn sub_assign(&mut self, rhs: Dim) {
            *self = *self - rhs;
        }
    }

    impl Add<Pos> for Pos {
        type Output = Pos;

        fn add(self, rhs: Pos) -> Self::Output {
            Pos {
                row: self.row + rhs.row,
                col: self.col + rhs.col,
            }
        }
    }

    impl Sub<Pos> for Pos {
        type Output = Pos;

        fn sub(self, rhs: Pos) -> Self::Output {
            Pos {
                row: self.row - rhs.row,
                col: self.col - rhs.col,
            }
        }
    }

    impl AddAssign<Pos> for Pos {
        fn add_assign(&mut self, rhs: Pos) {
            *self = *self + rhs;
        }
    }

    impl SubAssign<Pos> for Pos {
        fn sub_assign(&mut self, rhs: Pos) {
            *self = *self - rhs;
        }
    }

    impl Add<ColWidthCount> for Pos {
        type Output = Pos;

        fn add(self, rhs: ColWidthCount) -> Self::Output {
            Pos {
                row: self.row,
                col: self.col + rhs,
            }
        }
    }

    impl AddAssign<ColWidthCount> for Pos {
        fn add_assign(&mut self, rhs: ColWidthCount) {
            *self = *self + rhs;
        }
    }

    impl Sub<ColWidthCount> for Pos {
        type Output = Pos;

        fn sub(self, rhs: ColWidthCount) -> Self::Output {
            Pos {
                row: self.row,
                col: self.col - rhs,
            }
        }
    }

    impl SubAssign<ColWidthCount> for Pos {
        fn sub_assign(&mut self, rhs: ColWidthCount) {
            *self = *self - rhs;
        }
    }

    impl Add<RowHeightCount> for Pos {
        type Output = Pos;

        fn add(self, rhs: RowHeightCount) -> Self::Output {
            Pos {
                row: self.row + rhs,
                col: self.col,
            }
        }
    }

    impl Sub<RowHeightCount> for Pos {
        type Output = Pos;

        fn sub(self, rhs: RowHeightCount) -> Self::Output {
            Pos {
                row: self.row - rhs,
                col: self.col,
            }
        }
    }

    impl AddAssign<RowHeightCount> for Pos {
        fn add_assign(&mut self, rhs: RowHeightCount) {
            *self = *self + rhs;
        }
    }

    impl SubAssign<RowHeightCount> for Pos {
        fn sub_assign(&mut self, rhs: RowHeightCount) {
            *self = *self - rhs;
        }
    }
}

mod api {
    use super::*;

    // Reset API.
    impl Pos {
        /// Reset col and row index to `0`.
        pub fn reset(&mut self) {
            self.col.reset();
            self.row.reset();
        }
        /// Reset row index to `0`.
        pub fn reset_row(&mut self) {
            self.row.reset();
        }
        /// Reset col index to `0`.
        pub fn reset_col(&mut self) {
            self.col.reset();
        }
    }

    // Row index API.
    impl Pos {
        /// Set row index to `value`.
        pub fn set_row(&mut self, value: impl Into<RowIndex>) {
            self.row = value.into();
        }

        /// Increment row index by `value`.
        pub fn add_row(&mut self, value: impl Into<RowHeightCount>) {
            let value: RowHeightCount = value.into();
            self.row = self.row + value;
        }

        /// Increment row index by `value`, while making sure it will never exceed
        /// `max_row`.
        pub fn add_row_with_bounds(
            &mut self,
            value: impl Into<RowHeightCount>,
            max_row: impl Into<RowHeightCount>,
        ) {
            let value: RowHeightCount = value.into();
            let max: RowHeightCount = max_row.into();
            *self.row = std::cmp::min(*self.row + *value, *max);
        }

        /// Decrement row index by `value`.
        pub fn sub_row(&mut self, value: impl Into<RowHeightCount>) {
            let value: RowHeightCount = value.into();
            self.row = self.row - value;
        }
    }

    // Col index API.
    impl Pos {
        /// Set col index to `value`.
        pub fn set_col(&mut self, value: impl Into<ColIndex>) {
            self.col = value.into();
        }

        /// Increment col index by `value`.
        pub fn add_col(&mut self, value: impl Into<ColWidthCount>) {
            let value: ColWidthCount = value.into();
            self.col = self.col + value;
        }

        /// Increment col index by `value`, while making sure it will never exceed
        /// `max_col`.
        pub fn add_col_with_bounds(
            &mut self,
            value: impl Into<ColWidthCount>,
            max_col: impl Into<ColWidthCount>,
        ) {
            let value: ColWidthCount = value.into();
            let max: ColWidthCount = max_col.into();
            *self.col = std::cmp::min(*self.col + *value, *max);
        }

        /// Clip col index to `max_col` if it exceeds it.
        pub fn clip_col_to_bounds(&mut self, max_col: impl Into<ColWidthCount>) {
            let max: ColWidthCount = max_col.into();
            *self.col = std::cmp::min(*self.col, *max);
        }

        /// Decrement col index by `value`.
        pub fn sub_col(&mut self, value: impl Into<ColWidthCount>) {
            let value: ColWidthCount = value.into();
            self.col = self.col - value;
        }
    }
}

mod debug {
    use super::*;

    impl Debug for Pos {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "[c: {a:?}, r: {b:?}]", a = *self.col, b = *self.row)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RowHeightCount, col, height, row, width};
    use r3bl_core::ch;
    use std::fmt::Write as _;

    #[test]
    fn test_api() {
        // Constructor.
        {
            let pos_0 = row(1) + col(2);
            assert_eq!(*pos_0.row, ch(1));
            assert_eq!(*pos_0.col, ch(2));

            let pos_1 = pos(row(1) + col(2));
            assert_eq!(*pos_1.row, ch(1));
            assert_eq!(*pos_1.col, ch(2));

            let pos_2 = pos(col(2) + row(1));
            assert_eq!(*pos_2.row, ch(1));
            assert_eq!(*pos_2.col, ch(2));

            let scr_ofs_1 = scr_ofs(row(1) + col(2));
            assert_eq!(*scr_ofs_1.row, ch(1));
            assert_eq!(*scr_ofs_1.col, ch(2));
        }

        // Methods.
        {
            let row_index = RowIndex::new(ch(1));
            let col_index = ColIndex::new(ch(2));
            let width = ColWidthCount::new(ch(3));

            let mut pos: Pos = (col_index, row_index).into();
            assert_eq!(*pos.row, ch(1));
            assert_eq!(*pos.col, ch(2));

            pos.reset();
            assert_eq!(*pos.row, ch(0));
            assert_eq!(*pos.col, ch(0));

            *pos.row = ch(1);
            *pos.col = ch(2);
            assert_eq!(*pos.row, ch(1));
            assert_eq!(*pos.col, ch(2));

            pos.reset_col();
            assert_eq!(*pos.col, ch(0));

            pos.set_col(col_index);
            assert_eq!(*pos.col, ch(2));

            pos.add_col(width);
            assert_eq!(*pos.col, ch(5));

            pos.add_col_with_bounds(width, width);
            assert_eq!(*pos.col, ch(3));

            pos.clip_col_to_bounds(width - ColWidthCount::new(1));
            assert_eq!(*pos.col, ch(2));

            pos.sub_col(ColWidthCount::new(1));
            assert_eq!(*pos.col, ch(1));

            pos.sub_col(ColWidthCount::new(10));
            assert_eq!(*pos.col, ch(0));

            pos.reset_row();
            assert_eq!(*pos.row, ch(0));

            pos.set_row(row_index);
            assert_eq!(*pos.row, ch(1));

            pos.add_row(RowHeightCount::new(ch(3)));
            assert_eq!(*pos.row, ch(4));

            pos.add_row_with_bounds(RowHeightCount::new(ch(10)), RowHeightCount::new(ch(5)));
            assert_eq!(*pos.row, ch(5));

            pos.sub_row(RowHeightCount::new(ch(2)));
            assert_eq!(*pos.row, ch(3));

            pos.sub_row(RowHeightCount::new(ch(10)));
            assert_eq!(*pos.row, ch(0));
        }

        // Debug Pos.
        {
            let pos = Pos::new((ColIndex::new(ch(2)), RowIndex::new(ch(1))));
            let mut acc = String::new();
            let _ = write!(acc, "{:?}", pos);
            assert_eq!(acc, "[c: 2, r: 1]");
        }

        // Mul (ColWidthCount, RowHeightCount) or (RowHeightCount, ColWidthCount).
        {
            let pos = Pos::new((row(1), col(2)));

            let pos_1 = pos * (RowHeightCount::new(ch(2)), ColWidthCount::new(ch(2)));
            assert_eq!(*pos_1.row, ch(2));
            assert_eq!(*pos_1.col, ch(4));

            let pos_2 = pos * (ColWidthCount::new(ch(2)), RowHeightCount::new(ch(2)));
            assert_eq!(*pos_2.row, ch(2));
            assert_eq!(*pos_2.col, ch(4));
        }

        // Add, Sub Dim.
        {
            let pos = Pos::new((row(1), col(2)));
            let dim: Dim = (ColWidthCount::new(ch(2)), RowHeightCount::new(ch(2))).into();

            let pos_1 = pos + dim;
            assert_eq!(*pos_1.row, ch(3));
            assert_eq!(*pos_1.col, ch(4));

            let pos_2 = pos_1 - dim;
            assert_eq!(*pos_2.row, ch(1));
            assert_eq!(*pos_2.col, ch(2));
        }

        // AddAssign, SubAssign Dim.
        {
            let mut pos = Pos::new((RowIndex::new(ch(1)), ColIndex::new(ch(2))));
            pos += Dim::new((ColWidthCount::new(ch(2)), RowHeightCount::new(ch(2))));
            assert_eq!(*pos.row, ch(3));
            assert_eq!(*pos.col, ch(4));

            pos -= Dim::new((ColWidthCount::new(ch(2)), RowHeightCount::new(ch(2))));
            assert_eq!(*pos.row, ch(1));
            assert_eq!(*pos.col, ch(2));
        }

        // Add, Sub Pos.
        {
            let pos = Pos::new((row(2), col(2)));
            let pos_1 = pos - Pos::new((row(1), col(1)));
            assert_eq!(*pos_1.row, ch(1));
            assert_eq!(*pos_1.col, ch(1));

            let pos_2 = pos + Pos::new((row(1), col(1)));
            assert_eq!(*pos_2.row, ch(3));
            assert_eq!(*pos_2.col, ch(3));
        }

        // AddAssign, SubAssign Pos.
        {
            let mut pos_1 = Pos::new((row(1), col(2)));
            pos_1 += Pos::new((row(3), col(4)));
            assert_eq!(*pos_1.row, ch(4));
            assert_eq!(*pos_1.col, ch(6));

            let mut pos_2 = Pos::new((row(5), col(7)));
            pos_2 -= Pos::new((row(2), col(3)));
            assert_eq!(*pos_2.row, ch(3));
            assert_eq!(*pos_2.col, ch(4));
        }

        // Add, Sub ColWidthCount.
        {
            let pos = Pos::new((ColIndex::new(ch(5)), RowIndex::new(ch(7))));

            let pos_1 = pos + ColWidthCount::new(ch(2));
            assert_eq!(*pos_1.col, ch(7));
            assert_eq!(*pos_1.row, ch(7));

            let pos_2 = pos - ColWidthCount::new(ch(2));
            assert_eq!(*pos_2.col, ch(3));
            assert_eq!(*pos_2.row, ch(7));
        }

        // AddAssign, SubAssign ColWidthCount.
        {
            let mut pos_1 = Pos::new((row(5), col(7)));
            pos_1 += ColWidthCount::new(ch(2));
            assert_eq!(*pos_1.row, ch(5));

            let mut pos_2 = Pos::new((row(5), col(7)));
            pos_2 -= ColWidthCount::new(ch(2));
            assert_eq!(*pos_2.row, ch(5));
        }

        // Add, Sub RowWidthCount.
        {
            let pos = Pos::new((RowIndex::new(ch(5)), ColIndex::new(ch(7))));
            let pos_1 = pos + RowHeightCount::new(ch(2));
            assert_eq!(*pos_1.row, ch(7));

            let pos_2 = pos - RowHeightCount::new(ch(2));
            assert_eq!(*pos_2.row, ch(3));
        }

        // AddAssign, SubAssign RowWidthCount.
        {
            let mut pos_1 = Pos::new((RowIndex::new(ch(5)), ColIndex::new(ch(7))));
            pos_1 += RowHeightCount::new(ch(2));
            assert_eq!(*pos_1.row, ch(7));

            let mut pos_2 = Pos::new((RowIndex::new(ch(5)), ColIndex::new(ch(7))));
            pos_2 -= RowHeightCount::new(ch(2));
            assert_eq!(*pos_2.row, ch(3));
        }
    }

    #[test]
    fn test_pos_new() {
        // Order matters.
        let pos = Pos::new((row(1), col(2)));
        assert_eq!(pos.row, ch(1).into());
        assert_eq!(pos.col, ch(2).into());
        assert_eq!(*pos.row, ch(1));
        assert_eq!(*pos.col, ch(2));

        let pos_2 = Pos {
            row: ch(1).into(),
            col: ch(2).into(),
        };
        assert_eq!(pos, pos_2);
    }

    #[test]
    fn test_pos_from() {
        // Order does not matter.
        let pos_1: Pos = (RowIndex::new(1), ColIndex::new(2)).into();
        let pos_2: Pos = (ColIndex::new(2), RowIndex::new(1)).into();

        assert_eq!(pos_1, pos_2);
    }

    #[test]
    fn test_pos_add() {
        // Order matters!
        let pos1 = Pos::new((row(1), col(2)));
        let pos2 = Pos::new((row(3), col(4)));
        let result = pos1 + pos2;
        assert_eq!(result, Pos::new((row(4), col(6))));
    }

    #[test]
    fn test_pos_sub() {
        let pos1 = Pos::new((row(5), col(7)));
        let pos2 = Pos::new((row(2), col(3)));
        let result = pos1 - pos2;
        assert_eq!(result, Pos::new((row(3), col(4))));
    }

    #[test]
    fn test_add_box_size_to_pos() {
        let pos = row(1) + col(2);
        let dim = width(2) + height(2);
        let result = pos + dim;
        assert_eq!(result, row(3) + col(4));
    }

    #[test]
    fn test_mul_box_pos_to_pair() {
        // [30, 10] * [1, 0] = [30, 0]
        {
            let pos = col(30) + row(10);
            let pair_cancel_row = (width(1), height(0));
            let new_pos = pos * pair_cancel_row;
            assert_eq!(new_pos, col(30) + row(0));

            let dim_cancel_row = width(1) + height(0);
            let new_pos = pos * dim_cancel_row;
            assert_eq!(new_pos, col(30) + row(0));
        }

        // [30, 10] * [0, 1] = [0, 10]
        {
            let pos = col(30) + row(10);
            let pair_cancel_col = (width(0), height(1));
            let new_pos = pos * pair_cancel_col;
            assert_eq!(new_pos, col(0) + row(10));

            let dim_cancel_col = width(0) + height(1);
            let new_pos = pos * dim_cancel_col;
            assert_eq!(new_pos, col(0) + row(10));
        }
    }
}
