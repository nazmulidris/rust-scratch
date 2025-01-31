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

use crate::ColWidthCount;
use r3bl_core::ChUnit;
use std::ops::Mul;
use std::ops::{Add, Deref, DerefMut, Sub};

/// The horizontal index in a grid of characters, starting at 0, which is the first
/// column. This is one part of a [crate::Pos] (position), and is not the same as
/// [crate::ColWidthCount], which is one part of a [crate::Dim] (size). You can simply use
/// the [crate::col()] to create a new instance.
///
/// # Examples
///
/// ```rust
/// use unit_types::{ColIndex, col};
/// let col = col(5);
/// let col = ColIndex::new(5);
/// ```
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Ord, Eq, Hash)]
pub struct ColIndex(pub ChUnit);

pub fn col(col: impl Into<ChUnit>) -> ColIndex {
    ColIndex(col.into())
}

mod constructor {
    use super::*;

    impl ColIndex {
        pub fn new(ch_unit: impl Into<ChUnit>) -> Self {
            ColIndex(ch_unit.into())
        }

        pub fn reset(&mut self) {
            self.0.reset();
        }
    }

    impl From<ChUnit> for ColIndex {
        fn from(ch_unit: ChUnit) -> Self {
            ColIndex(ch_unit)
        }
    }
}

mod ops {
    use super::*;

    impl Deref for ColIndex {
        type Target = ChUnit;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for ColIndex {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl Sub<ColIndex> for ColIndex {
        type Output = ColIndex;

        fn sub(self, rhs: ColIndex) -> Self::Output {
            ColIndex(self.0 - rhs.0)
        }
    }

    impl Add<ColIndex> for ColIndex {
        type Output = ColIndex;

        fn add(self, rhs: ColIndex) -> Self::Output {
            ColIndex(self.0 + rhs.0)
        }
    }

    impl Sub<ColWidthCount> for ColIndex {
        type Output = ColIndex;

        fn sub(self, rhs: ColWidthCount) -> Self::Output {
            ColIndex(self.0 - rhs.0)
        }
    }

    impl Add<ColWidthCount> for ColIndex {
        type Output = ColIndex;

        fn add(self, rhs: ColWidthCount) -> Self::Output {
            ColIndex(self.0 + rhs.0)
        }
    }

    impl Mul<ColWidthCount> for ColIndex {
        type Output = ColIndex;

        fn mul(self, rhs: ColWidthCount) -> Self::Output {
            ColIndex(self.0 * rhs.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use r3bl_core::ch;

    use super::*;

    #[test]
    fn test_col_index_add() {
        let col1 = ColIndex::from(ch(5));
        let col2 = ColIndex::new(3);
        let result = col1 + col2;
        assert_eq!(result, ColIndex::new(8));
    }

    #[test]
    fn test_col_index_sub() {
        let col1 = ColIndex::from(ch(5));
        let col2 = ColIndex::new(3);
        let result = col1 - col2;
        assert_eq!(result, ColIndex::new(2));
    }

    #[test]
    fn test_deref_and_deref_mut() {
        let mut col = ColIndex::new(5);
        assert_eq!(*col, ch(5));
        *col = ch(10);
        assert_eq!(*col, ch(10));
    }
}
