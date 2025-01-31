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

use crate::RowHeightCount;
use r3bl_core::ChUnit;
use std::ops::{Add, Deref, DerefMut, Mul, Sub};

/// The vertical index in a grid of characters, starting at 0, which is the first row.
/// This is one part of a [crate::Pos] (position), and is not the same as
/// [crate::RowHeightCount], which is one part of a [crate::Dim] (size). You can simply
/// use the [crate::row()] to create a new instance.
///
/// # Examples
///
/// ```rust
/// use unit_types::{RowIndex, row};
/// let row = row(5);
/// let row = RowIndex::new(5);
/// ```
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Ord, Eq, Hash)]
pub struct RowIndex(pub ChUnit);

pub fn row(row: impl Into<ChUnit>) -> RowIndex {
    RowIndex(row.into())
}

mod constructor {
    use super::*;

    impl RowIndex {
        pub fn new(ch_unit: impl Into<ChUnit>) -> Self {
            RowIndex(ch_unit.into())
        }

        pub fn reset(&mut self) {
            self.0.reset();
        }
    }

    impl From<ChUnit> for RowIndex {
        fn from(ch_unit: ChUnit) -> Self {
            RowIndex(ch_unit)
        }
    }
}

mod ops {
    use super::*;

    impl Deref for RowIndex {
        type Target = ChUnit;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for RowIndex {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl Sub<RowIndex> for RowIndex {
        type Output = RowIndex;

        fn sub(self, rhs: RowIndex) -> Self::Output {
            RowIndex(self.0 - rhs.0)
        }
    }

    impl Add<RowIndex> for RowIndex {
        type Output = RowIndex;

        fn add(self, rhs: RowIndex) -> Self::Output {
            RowIndex(self.0 + rhs.0)
        }
    }

    impl Sub<RowHeightCount> for RowIndex {
        type Output = RowIndex;

        fn sub(self, rhs: RowHeightCount) -> Self::Output {
            RowIndex(self.0 - rhs.0)
        }
    }

    impl Add<RowHeightCount> for RowIndex {
        type Output = RowIndex;

        fn add(self, rhs: RowHeightCount) -> Self::Output {
            RowIndex(self.0 + rhs.0)
        }
    }

    impl Mul<RowHeightCount> for RowIndex {
        type Output = RowIndex;

        fn mul(self, rhs: RowHeightCount) -> Self::Output {
            RowIndex(self.0 * rhs.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use r3bl_core::ch;

    use super::*;

    #[test]
    fn test_row_index_add() {
        let row1 = RowIndex::from(ch(5));
        let row2 = RowIndex::new(3);
        let result = row1 + row2;
        assert_eq!(result, RowIndex(ch(8)));
        assert_eq!(*result, ch(8));
    }

    #[test]
    fn test_row_index_sub() {
        let row1 = RowIndex::from(ch(5));
        let row2 = RowIndex::new(3);
        let result = row1 - row2;
        assert_eq!(result, RowIndex::new(2));
        assert_eq!(*result, ch(2));
    }

    #[test]
    fn test_deref_and_deref_mut() {
        let mut row = RowIndex::new(5);
        assert_eq!(*row, ch(5));
        *row = ch(10);
        assert_eq!(*row, ch(10));
    }
}
