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

/// Width is column count, ie the number of columns that a UI component occupies. This is
/// one part of a [crate::Dim] (size), and is not the same as the [crate::ColIndex]
/// (position). You can simply use the [crate::width()] to create a new instance.
///
/// # Examples
///
/// ```rust
/// use unit_types::{ColWidthCount, width};
/// let width = width(5);
/// let width = ColWidthCount::new(5);
/// ```
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Ord, Eq, Hash)]
pub struct ColWidthCount(pub ChUnit);

pub fn width(width: impl Into<ChUnit>) -> ColWidthCount {
    ColWidthCount(width.into())
}

mod constructor {
    use super::*;

    impl ColWidthCount {
        pub fn new(arg: impl Into<ChUnit>) -> Self {
            ColWidthCount(arg.into())
        }
    }

    impl From<ChUnit> for ColWidthCount {
        fn from(ch_unit: ChUnit) -> Self {
            ColWidthCount(ch_unit)
        }
    }
}

mod ops {
    use super::*;

    impl Deref for ColWidthCount {
        type Target = ChUnit;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for ColWidthCount {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl Add<ColWidthCount> for ColWidthCount {
        type Output = ColWidthCount;

        fn add(self, rhs: ColWidthCount) -> Self::Output {
            ColWidthCount(self.0 + rhs.0)
        }
    }

    impl Sub<ColWidthCount> for ColWidthCount {
        type Output = ColWidthCount;

        fn sub(self, rhs: ColWidthCount) -> Self::Output {
            ColWidthCount(self.0 - rhs.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use r3bl_core::ch;

    use super::*;

    #[test]
    fn test_width_col_count_new() {
        let it = ColWidthCount::new(5);
        assert_eq!(it, width(5));
        assert_eq!(*it, ch(5));
    }

    #[test]
    fn test_width_col_count_add() {
        let width1 = ColWidthCount(5.into());
        let width2 = ColWidthCount(3.into());
        let result = width1 + width2;
        assert_eq!(result, ColWidthCount(8.into()));
        assert_eq!(*result, ch(8));
    }

    #[test]
    fn test_width_col_count_sub() {
        let width1 = ColWidthCount(5.into());
        let width2 = ColWidthCount(3.into());
        let result = width1 - width2;
        assert_eq!(result, ColWidthCount(2.into()));
        assert_eq!(*result, ch(2));
    }

    #[test]
    fn test_deref_and_deref_mut() {
        let mut width = ColWidthCount(5.into());
        assert_eq!(*width, ch(5));
        *width = ch(10);
        assert_eq!(*width, ch(10));
    }
}
