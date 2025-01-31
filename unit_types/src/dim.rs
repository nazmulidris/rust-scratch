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

//! [Dim] is a struct that holds the `width` and `height` of a text buffer.
//! [ColWidthCount] (aka [Width]) and [RowHeightCount] (aka [Height]) are the types of the
//! `width` and `height` respectively. This ensures that it isn't possible to use a
//! `width` when you intended to use a `height` and vice versa. Also [Size] is an alias
//! for [Dim].
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
//! - This API uses the `impl Into<struct>` pattern and [Add] `+` operator overloading to
//!   allow for easy construction of [Dim] by [ColWidthCount] with [RowHeightCount] in any
//!   order.
//! - You can use the [crate::dim()] to create a [Dim] struct. This function can take a
//!   sequence of [Add]ed [Width] and [Height] in any order, or tuples of them in any
//!   order.
//! - Just using the [Add] `+` operator ([Height] and [Width] can be in any order):
//!     - You can use [Add] to convert: [Width] + [Height], into: a [Dim].
//!
//! # Examples
//!
//! ```rust
//! use r3bl_core::ch;
//! use unit_types::{Dim, ColWidthCount, RowHeightCount, width, height, Width, Height, dim};
//!
//! // Note the order of the arguments don't matter below.
//! let size: Dim = dim( width(1) + height(2) );
//! assert_eq!(size.width, ch(1).into());
//! assert_eq!(*size.height, ch(2));
//!
//! // Note the order of the arguments don't matter below.
//! let size_2: Dim = ( height(2), width(1) ).into();
//! assert_eq!(*size_2.width, ch(1));
//! assert_eq!(*size_2.height, ch(2));
//!
//! // Note the order of the arguments don't matter below.
//! let size_3 = Dim::new(
//!     ( height(2), width(1) )
//! );
//! assert!(matches!(size_3.width, ColWidthCount(_)));
//! assert!(matches!(size_3.height, RowHeightCount(_)));
//! assert!(size_2 == size_3);
//!
//! let size_sum = size_2 + size_3;
//! assert_eq!(size_sum.width, ch(2).into());
//! assert_eq!(*size_sum.height, ch(4));
//!
//! let size_diff = size_2 - size_3;
//! assert_eq!(size_diff.width, ch(0).into());
//! assert_eq!(*size_diff.height, ch(0));
//! ```

use crate::{ColWidthCount, RowHeightCount};
use r3bl_core::ChUnit;
use std::fmt::Debug;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;
use std::ops::SubAssign;

// Type aliases for better code readability.
pub type Size = Dim;
pub type Width = ColWidthCount;
pub type Height = RowHeightCount;

#[derive(Copy, Clone, PartialEq, PartialOrd, Default, Ord, Eq, Hash)]
pub struct Dim {
    pub width: Width,
    pub height: Height,
}

pub fn dim(arg: impl Into<Dim>) -> Dim {
    arg.into()
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Ord, Eq, Hash)]
pub enum SufficientSize {
    IsLargeEnough,
    IsTooSmall,
}

// TODO: [ ] impl constructor, debug, ops for Dim (equivalent to r3bl_core::Size)

mod constructor {
    use super::*;

    impl Dim {
        pub fn new(arg: impl Into<Dim>) -> Self {
            arg.into()
        }
    }

    impl From<(ColWidthCount, RowHeightCount)> for Dim {
        fn from((width, height): (ColWidthCount, RowHeightCount)) -> Self {
            Dim { width, height }
        }
    }

    impl From<(RowHeightCount, ColWidthCount)> for Dim {
        fn from((height, width): (RowHeightCount, ColWidthCount)) -> Self {
            Dim { width, height }
        }
    }

    impl Add<Height> for Width {
        type Output = Dim;

        fn add(self, rhs: Height) -> Self::Output {
            Dim {
                width: self,
                height: rhs,
            }
        }
    }

    impl Add<Width> for Height {
        type Output = Dim;
        fn add(self, rhs: Width) -> Self::Output {
            Dim {
                width: rhs,
                height: self,
            }
        }
    }
}

mod api {
    use super::*;

    impl Dim {
        pub fn fits_min_size(&self, min_size: impl Into<Dim>) -> SufficientSize {
            let size: Dim = min_size.into();
            let min_width = size.width;
            let min_height = size.height;

            if self.width < min_width || self.height < min_height {
                SufficientSize::IsLargeEnough
            } else {
                SufficientSize::IsTooSmall
            }
        }
    }
}

mod debug {
    use super::*;

    impl Debug for Dim {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[w: {w:?}, h: {h:?}]", w = *self.width, h = *self.height)
        }
    }
}

mod ops {
    use super::*;

    impl Sub<Dim> for Dim {
        type Output = Dim;

        fn sub(self, rhs: Dim) -> Self::Output {
            Dim {
                width: self.width - rhs.width,
                height: self.height - rhs.height,
            }
        }
    }

    impl Add<Dim> for Dim {
        type Output = Dim;

        fn add(self, rhs: Dim) -> Self::Output {
            Dim {
                width: self.width + rhs.width,
                height: self.height + rhs.height,
            }
        }
    }

    impl SubAssign<ChUnit> for Dim {
        fn sub_assign(&mut self, other: ChUnit) {
            *self.width -= other;
            *self.height -= other;
        }
    }

    impl AddAssign<ChUnit> for Dim {
        fn add_assign(&mut self, other: ChUnit) {
            *self.width += other;
            *self.height += other;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{height, width};

    use super::*;
    use r3bl_core::ch;

    #[test]
    fn test_dim() {
        let size = dim(width(5) + height(10));
        assert_eq!(size.width, ColWidthCount(ch(5)));
        assert_eq!(*size.width, ch(5));
        assert_eq!(size.height, RowHeightCount(ch(10)));
        assert_eq!(*size.height, ch(10));
        let size_2 = dim(height(10) + width(5));

        assert!(matches!(size_2.width, ColWidthCount(_)));
        assert!(matches!(size_2.height, RowHeightCount(_)));
    }

    #[test]
    fn test_size_new() {
        // Order does not matter.
        let size = Dim::new((ColWidthCount::new(5), RowHeightCount::new(10)));
        assert_eq!(size.width, ColWidthCount(ch(5)));
        assert_eq!(*size.width, 5.into());
        assert_eq!(size.height, RowHeightCount(10.into()));
        assert_eq!(*size.height, ch(10));

        // Order does not matter.
        let size_2 = Dim::new((width(5), height(10)));
        assert!(matches!(size_2.width, ColWidthCount(_)));
        assert!(matches!(size_2.height, RowHeightCount(_)));
    }

    #[test]
    fn test_size_from() {
        // Order does not matter!
        let size: Dim = (ColWidthCount(ch(5)), RowHeightCount(ch(10))).into();
        let size_2: Dim = (RowHeightCount(ch(10)), ColWidthCount(ch(5))).into();

        assert_eq!(size.width, ColWidthCount(ch(5)));
        assert_eq!(*size.width, ch(5));
        assert_eq!(size.height, RowHeightCount(ch(10)));
        assert_eq!(*size.height, ch(10));

        assert_eq!(size, size_2);
    }

    #[test]
    fn test_size_add() {
        let size1 = Dim::new((ColWidthCount(5.into()), RowHeightCount(10.into())));
        let size2 = Dim::new((ColWidthCount::from(ch(3)), RowHeightCount::from(ch(4))));
        let result = size1 + size2;
        assert_eq!(result.width, ColWidthCount(8.into()));
        assert_eq!(*result.width, ch(8));
        assert_eq!(result.height, RowHeightCount(14.into()));
        assert_eq!(*result.height, ch(14));
    }

    #[test]
    fn test_size_sub() {
        let size1 = Dim::new((ColWidthCount(5.into()), RowHeightCount(10.into())));
        let size2 = Dim::new((ColWidthCount(3.into()), RowHeightCount(4.into())));
        let result = size1 - size2;
        assert_eq!(result.width, ColWidthCount(ch(2)));
        assert_eq!(result.height, RowHeightCount(ch(6)));
    }

    #[test]
    fn test_fits_min_size() {
        let size = Dim::new((width(5), height(10)));
        assert_eq!(
            size.fits_min_size(Dim::new((width(3), height(4)))),
            SufficientSize::IsTooSmall
        );
        assert_eq!(
            size.fits_min_size(Dim::new((width(100), height(100)))),
            SufficientSize::IsLargeEnough
        );
    }

    #[test]
    fn test_debug_fmt() {
        let size = Dim::new((width(5), height(10)));
        assert_eq!(format!("{:?}", size), "[w: 5, h: 10]");
    }

    #[test]
    fn test_size_sub_assign() {
        let mut size = Dim::new((width(5), height(10)));
        size -= ch(3);
        assert_eq!(size.width, ColWidthCount(ch(2)));
        assert_eq!(size.height, RowHeightCount(ch(7)));
    }

    #[test]
    fn test_size_add_assign() {
        let mut size = Dim::new((width(5), height(10)));
        size += ch(3);
        assert_eq!(size.width, ColWidthCount(ch(8)));
        assert_eq!(size.height, RowHeightCount(ch(13)));
    }
}
