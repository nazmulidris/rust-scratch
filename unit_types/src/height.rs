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

use r3bl_core::ChUnit;
use std::ops::DerefMut;
use std::ops::Sub;
use std::ops::{Add, Deref};

/// Height is row count, ie the number of rows that a UI component occupies. This is one
/// part of a [crate::Dim] (size), and is not the same as the [crate::RowIndex]
/// (position). You can simply use the [crate::height()] to create a new instance.
///
/// # Examples
///
/// ```rust
/// use unit_types::{RowHeightCount, height};
/// let height = height(5);
/// let height = RowHeightCount::new(5);
/// ```
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Ord, Eq, Hash)]
pub struct RowHeightCount(pub ChUnit);

pub fn height(height: impl Into<ChUnit>) -> RowHeightCount {
    RowHeightCount(height.into())
}

mod constructor {
    use super::*;

    impl RowHeightCount {
        pub fn new(arg: impl Into<ChUnit>) -> Self {
            RowHeightCount(arg.into())
        }
    }

    impl From<ChUnit> for RowHeightCount {
        fn from(ch_unit: ChUnit) -> Self {
            RowHeightCount(ch_unit)
        }
    }
}

mod ops {
    use super::*;

    impl Deref for RowHeightCount {
        type Target = ChUnit;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for RowHeightCount {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl Add<RowHeightCount> for RowHeightCount {
        type Output = RowHeightCount;

        fn add(self, rhs: RowHeightCount) -> Self::Output {
            RowHeightCount(self.0 + rhs.0)
        }
    }

    impl Sub<RowHeightCount> for RowHeightCount {
        type Output = RowHeightCount;

        fn sub(self, rhs: RowHeightCount) -> Self::Output {
            RowHeightCount(self.0 - rhs.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use r3bl_core::ch;

    use super::*;

    #[test]
    fn test_height_row_count_new() {
        let height = RowHeightCount::new(10);
        assert_eq!(height, RowHeightCount(10.into()));
        assert_eq!(*height, ch(10));
    }

    #[test]
    fn test_height_row_count_add() {
        let height1 = RowHeightCount(10.into());
        let height2 = RowHeightCount(4.into());
        let result = height1 + height2;
        assert_eq!(result, RowHeightCount(14.into()));
        assert_eq!(*result, ch(14));
    }

    #[test]
    fn test_height_row_count_sub() {
        let height1 = RowHeightCount(10.into());
        let height2 = RowHeightCount(4.into());
        let result = height1 - height2;
        assert_eq!(result, RowHeightCount(6.into()));
        assert_eq!(*result, ch(6));
    }

    #[test]
    fn test_deref_and_deref_mut() {
        let mut height = RowHeightCount(10.into());
        assert_eq!(*height, ch(10));
        *height = ch(20);
        assert_eq!(*height, ch(20));
    }
}
