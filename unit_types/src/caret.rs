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

//! [Caret] is an enum that represents the position of the caret in the text buffer. It
//! can be in one of two states:
//! - [RawCaret]: A struct that represents the "raw" position is the `col_index` and
//!   `row_index` of the caret INSIDE the viewport, without making any adjustments for
//!   scrolling.
//! - [ScrAdjCaret]: A struct that represents the "scroll adjusted" position is the
//!   `col_index` and `row_index` of the caret OUTSIDE the viewport, after making
//!   adjustments for scrolling.
//!
//! The [Caret] enum is a wrapper around the [RawCaret] and [ScrAdjCaret] structs. It can
//! be converted to and from both types. You can either indirectly use both of these
//! structs using [Caret] or you can just use them directly using [RawCaret] or
//! [ScrAdjCaret].
//!
//! # The many ways to create one
//!
//! - This API uses the `impl Into<struct>` pattern and [Add] `+` operator overloading to
//!   allow for easy conversion between [RawCaret] and [ScrAdjCaret].
//! - You can use the [caret()], [raw_caret()], [scr_adj_caret()] functions to create a
//!   [Caret], [RawCaret], [ScrAdjCaret] struct respectively. These functions can take a
//!   sequence of [Add]ed [Pos] and [ScrOfs] as input, or tuples of them in any order.
//! - Just using using the [Add] `+` operator:
//!     - You can use [Add] to convert: [ScrOfs] + [RawCaret], into: a [ScrAdjCaret].
//!     - You can use [Add] to convert: [ScrAdjCaret] + [ScrOfs], into: a [RawCaret].
//!
//! # Examples
//!
//! ```rust
//! use r3bl_core::ch;
//! use unit_types::{
//!     Pos, ScrOfs, RawCaret, ScrAdjCaret, Caret,
//!     col, row, raw_caret, scr_ofs, pos, scr_adj_caret, caret
//! };
//!
//! let scroll_offset_1: ScrOfs = row(2) + col(3);
//!
//! //
//! // Directly using RawCaret and ScrAdjCaret.
//! //
//!
//! // Note the order of the arguments don't matter below.
//! let raw_caret_1: RawCaret = raw_caret(col(5) + row(5));
//! let scr_adj_caret_1: ScrAdjCaret = scr_adj_caret(col(7) + row(8));
//!
//! assert!(matches!(Caret::new(raw_caret_1), Caret::Raw(_)));
//! assert_eq!(pos(row(5) + col(5)), *raw_caret_1);
//! assert!(matches!(Caret::new(scr_adj_caret_1), Caret::ScrollAdjusted(_)));
//! assert_eq!(pos(row(8) + col(7)), *scr_adj_caret_1);
//!
//! //
//! // Using Caret (and not directly specifying RawCaret or ScrAdjCaret).
//! //
//!
//! // Convert ScrAdjCaret (and ScrollOffset) to RawCaret.
//! let caret_1: Caret = caret(scr_adj_caret_1 + scroll_offset_1);
//! let caret_2: Caret = caret(scroll_offset_1 + scr_adj_caret_1);
//! assert!(matches!(caret_1, Caret::Raw(_)));
//! assert!(matches!(caret_2, Caret::Raw(_)));
//! let expected_1 = pos(row(8) + col(7)) - scr_ofs(row(2) + col(3));
//! assert_eq!(expected_1, *caret_1);
//! assert_eq!(expected_1, *caret_2);
//!
//! // Convert RawCaret (and ScrollOffset) to ScrAdjCaret.
//! let caret_3: Caret = caret(raw_caret_1 + scroll_offset_1);
//! let caret_4: Caret = caret(scroll_offset_1 + raw_caret_1);
//! assert!(matches!(caret_3, Caret::ScrollAdjusted(_)));
//! assert!(matches!(caret_4, Caret::ScrollAdjusted(_)));
//! let expected_2 = pos(row(5) + col(5)) + scr_ofs(row(2) + col(3));
//! assert_eq!(expected_2, *caret_3);
//! assert_eq!(expected_2, *caret_4);
//! ```

use crate::{Pos, ScrOfs};
use std::ops::Add;
use std::ops::Deref;

pub fn caret(arg: impl Into<Caret>) -> Caret {
    arg.into()
}

pub fn raw_caret(arg: impl Into<RawCaret>) -> RawCaret {
    arg.into()
}

pub fn scr_adj_caret(arg: impl Into<ScrAdjCaret>) -> ScrAdjCaret {
    arg.into()
}

/// The "raw" position is the `col_index` and `row_index` of the caret INSIDE the
/// viewport, without making any adjustments for scrolling.
/// - It does not take into account the amount of scrolling (vertical, horizontal) that is
///   currently active.
/// - When scrolling is "active", this position will be different from the "scroll
///   adjusted" position.
/// - This is the default `CaretKind`.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct RawCaret(pub Pos);

mod raw_caret_impl {
    use super::*;

    impl RawCaret {
        pub fn new(arg: impl Into<RawCaret>) -> Self {
            arg.into()
        }
    }

    impl Deref for RawCaret {
        type Target = Pos;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl From<(ScrAdjCaret, ScrOfs)> for RawCaret {
        fn from((scr_adj_caret, scr_ofs): (ScrAdjCaret, ScrOfs)) -> Self {
            let position = *scr_adj_caret - scr_ofs;
            RawCaret(position)
        }
    }

    impl From<(ScrOfs, ScrAdjCaret)> for RawCaret {
        fn from((scr_ofs, scr_adj_caret): (ScrOfs, ScrAdjCaret)) -> Self {
            (scr_adj_caret, scr_ofs).into()
        }
    }

    impl From<Pos> for RawCaret {
        fn from(position: Pos) -> Self {
            RawCaret(position)
        }
    }

    // ScrAdjCaret + ScrOfs = RawCaret
    impl Add<ScrAdjCaret> for ScrOfs {
        type Output = RawCaret;

        fn add(self, rhs: ScrAdjCaret) -> Self::Output {
            (rhs, self).into()
        }
    }

    // ScrOfs + ScrAdjCaret = RawCaret
    impl Add<ScrOfs> for ScrAdjCaret {
        type Output = RawCaret;

        fn add(self, rhs: ScrOfs) -> Self::Output {
            (self, rhs).into()
        }
    }
}

/// The "scroll adjusted" position is the `col_index` and `row_index` of the caret OUTSIDE
/// the viewport, after making adjustments for scrolling.
/// - It takes into account the amount of scrolling (vertical, horizontal) that is
///   currently active.
/// - When scrolling is "active", this position will be different from the "raw" position.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct ScrAdjCaret(pub Pos);

mod scroll_adjusted_caret_impl {
    use super::*;

    impl ScrAdjCaret {
        pub fn new(arg: impl Into<ScrAdjCaret>) -> Self {
            arg.into()
        }
    }

    impl Deref for ScrAdjCaret {
        type Target = Pos;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl From<(RawCaret, ScrOfs)> for ScrAdjCaret {
        fn from((raw_caret, scr_ofs): (RawCaret, ScrOfs)) -> Self {
            let position = *raw_caret + scr_ofs;
            ScrAdjCaret(position)
        }
    }

    impl From<(ScrOfs, RawCaret)> for ScrAdjCaret {
        fn from((scr_ofs, raw_caret): (ScrOfs, RawCaret)) -> Self {
            (raw_caret, scr_ofs).into()
        }
    }

    impl From<Pos> for ScrAdjCaret {
        fn from(position: Pos) -> Self {
            ScrAdjCaret(position)
        }
    }

    // RawCaret + ScrOfs = ScrAdjCaret
    impl Add<RawCaret> for ScrOfs {
        type Output = ScrAdjCaret;

        fn add(self, rhs: RawCaret) -> Self::Output {
            (rhs, self).into()
        }
    }

    // ScrOfs + RawCaret = ScrAdjCaret
    impl Add<ScrOfs> for RawCaret {
        type Output = ScrAdjCaret;

        fn add(self, rhs: ScrOfs) -> Self::Output {
            (self, rhs).into()
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Caret {
    Raw(RawCaret),
    ScrollAdjusted(ScrAdjCaret),
}

mod caret_impl {
    use super::*;

    impl Caret {
        pub fn new(arg: impl Into<Caret>) -> Self {
            arg.into()
        }
    }

    impl Deref for Caret {
        type Target = Pos;

        fn deref(&self) -> &Self::Target {
            match self {
                Caret::Raw(raw_caret) => raw_caret,
                Caret::ScrollAdjusted(scr_adj_caret) => scr_adj_caret,
            }
        }
    }

    impl Default for Caret {
        fn default() -> Self {
            Caret::Raw(RawCaret::default())
        }
    }

    impl From<ScrAdjCaret> for Caret {
        fn from(scr_adj_caret: ScrAdjCaret) -> Self {
            Caret::ScrollAdjusted(scr_adj_caret)
        }
    }

    impl From<RawCaret> for Caret {
        fn from(raw_caret: RawCaret) -> Self {
            Caret::Raw(raw_caret)
        }
    }

    impl From<(RawCaret, ScrOfs)> for Caret {
        fn from((raw_caret, scr_ofs): (RawCaret, ScrOfs)) -> Self {
            let scr_adj_caret: ScrAdjCaret = (raw_caret, scr_ofs).into();
            Caret::ScrollAdjusted(scr_adj_caret)
        }
    }

    impl From<(ScrOfs, RawCaret)> for Caret {
        fn from((scr_ofs, raw_caret): (ScrOfs, RawCaret)) -> Self {
            (raw_caret, scr_ofs).into()
        }
    }

    impl From<(ScrAdjCaret, ScrOfs)> for Caret {
        fn from((scr_adj_caret, scr_ofs): (ScrAdjCaret, ScrOfs)) -> Self {
            let raw_caret: RawCaret = (scr_adj_caret, scr_ofs).into();
            Caret::Raw(raw_caret)
        }
    }

    impl From<(ScrOfs, ScrAdjCaret)> for Caret {
        fn from((scr_ofs, scr_adj_caret): (ScrOfs, ScrAdjCaret)) -> Self {
            (scr_adj_caret, scr_ofs).into()
        }
    }

    impl From<Pos> for Caret {
        fn from(position: Pos) -> Self {
            let raw_caret: RawCaret = position.into();
            Caret::Raw(raw_caret)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{col, row, scr_ofs};
    use r3bl_core::ch;

    #[test]
    fn test_constructor_fns() {
        let pos_1 = row(5) + col(5);
        let pos_2 = col(2) + row(3);

        // raw_caret constructor fn.
        {
            let rc = raw_caret(pos_1);
            assert!(matches!(rc, RawCaret { .. }));
            assert_eq!(*rc, pos_1);
        }

        // scr_adj_caret constructor fn.
        {
            let sac = scr_adj_caret(pos_1);
            assert!(matches!(sac, ScrAdjCaret { .. }));
            assert_eq!(*sac, pos_1);
        }

        // Into RawCaret, from ...
        {
            let scr_ofs = scr_ofs(pos_2);
            let scr_adj_caret = scr_adj_caret(pos_1);

            let raw_caret_1 = caret(scr_ofs + scr_adj_caret);
            assert!(matches!(raw_caret_1, Caret::Raw(_)));
            assert_eq!(*raw_caret_1, *scr_adj_caret - scr_ofs);

            let raw_caret_2 = caret(scr_adj_caret + scr_ofs);
            assert!(matches!(raw_caret_2, Caret::Raw(_)));
            assert_eq!(*raw_caret_2, *scr_adj_caret - scr_ofs);
        }

        // Into ScrAdjCaret, from ...
        {
            let raw_caret = raw_caret(pos_1);
            let scr_ofs = scr_ofs(pos_1);

            let scr_adj_caret_1 = caret(raw_caret + scr_ofs);
            assert!(matches!(scr_adj_caret_1, Caret::ScrollAdjusted(_)));
            assert_eq!(*scr_adj_caret_1, *raw_caret + scr_ofs);

            let scr_adj_caret_2 = caret(scr_ofs + raw_caret);
            assert!(matches!(scr_adj_caret_2, Caret::ScrollAdjusted(_)));
            assert_eq!(*scr_adj_caret_2, *raw_caret + scr_ofs);
        }
    }

    #[test]
    fn test_default_caret_kind() {
        let default_caret = Caret::default();

        assert!(matches!(default_caret, Caret::Raw(_)));
        assert_eq!(default_caret, Caret::Raw(RawCaret::default()));
        assert_eq!(*default_caret, Pos::default());

        let caret: Caret = Pos::default().into();

        assert!(matches!(caret, Caret::Raw(_)));
        assert_eq!(caret, Caret::Raw(RawCaret::default()));
    }

    #[test]
    fn test_caret_new() {
        let raw_caret = RawCaret::new(Pos {
            col: ch(7).into(),
            row: ch(8).into(),
        });

        let scr_adj_caret = ScrAdjCaret::new(Pos {
            col: ch(7).into(),
            row: ch(8).into(),
        });

        {
            let caret = Caret::new(scr_adj_caret);
            assert!(matches!(caret, Caret::ScrollAdjusted(_)));
        }

        {
            let caret = Caret::new(raw_caret);
            assert!(matches!(caret, Caret::Raw(_)));
        }

        {
            let caret = Caret::new(Pos {
                col: ch(7).into(),
                row: ch(8).into(),
            });
            assert!(matches!(caret, Caret::Raw(_)));
        }

        {
            let caret = Caret::new((raw_caret, ScrOfs {
                col: ch(2).into(),
                row: ch(3).into(),
            }));
            assert!(matches!(caret, Caret::ScrollAdjusted(_)));
        }

        {
            let caret = Caret::new((scr_adj_caret, ScrOfs {
                col: ch(2).into(),
                row: ch(3).into(),
            }));
            assert!(matches!(caret, Caret::Raw(_)));
        }
    }

    #[test]
    fn test_caret_from() {
        let raw_caret = RawCaret::new(Pos {
            col: ch(7).into(),
            row: ch(8).into(),
        });

        let scr_adj_caret = ScrAdjCaret::new(Pos {
            col: ch(7).into(),
            row: ch(8).into(),
        });

        {
            let caret: Caret = scr_adj_caret.into();
            assert!(matches!(caret, Caret::ScrollAdjusted(_)));
        }

        {
            let caret: Caret = raw_caret.into();
            assert!(matches!(caret, Caret::Raw(_)));
        }

        {
            let caret: Caret = Pos {
                col: ch(7).into(),
                row: ch(8).into(),
            }
            .into();
            assert!(matches!(caret, Caret::Raw(_)));
        }

        {
            let caret: Caret = (raw_caret, ScrOfs {
                col: ch(2).into(),
                row: ch(3).into(),
            })
                .into();
            assert!(matches!(caret, Caret::ScrollAdjusted(_)));
        }

        {
            let caret: Caret = (scr_adj_caret, ScrOfs {
                col: ch(2).into(),
                row: ch(3).into(),
            })
                .into();
            assert!(matches!(caret, Caret::Raw(_)));
        }
    }

    #[test]
    fn test_raw_to_scroll_adjusted() {
        let position = Pos {
            col: ch(5).into(),
            row: ch(5).into(),
        };

        let scr_ofs = ScrOfs {
            col: ch(2).into(),
            row: ch(3).into(),
        };

        // Create RawCaret from Position.
        let raw_caret: RawCaret = position.into();

        assert_eq!(raw_caret.0, position);
        assert_eq!(*raw_caret, position);

        // Convert RawCaret (and ScrollOffset) to ScrAdjCaret.
        let scr_adj_caret: ScrAdjCaret = (raw_caret, scr_ofs).into();

        assert_eq!(scr_adj_caret.0, Pos {
            col: ch(7).into(),
            row: ch(8).into()
        });
        assert_eq!(*scr_adj_caret, Pos {
            col: ch(7).into(),
            row: ch(8).into()
        });

        // Convert RawCaret (and ScrollOffset) to Caret.
        let caret: Caret = (raw_caret, scr_ofs).into();

        assert!(matches!(caret, Caret::ScrollAdjusted(_)));
        assert!(!matches!(caret, Caret::Raw(_)));
        assert_eq!(*caret, *scr_adj_caret);
    }

    #[test]
    fn test_scroll_adjusted_to_raw() {
        let scr_adj_caret: ScrAdjCaret = Pos {
            col: ch(7).into(),
            row: ch(8).into(),
        }
        .into();

        let scr_ofs = ScrOfs {
            col: ch(2).into(),
            row: ch(3).into(),
        };

        let raw_caret: RawCaret = (scr_adj_caret, scr_ofs).into();

        assert_eq!(*raw_caret, Pos {
            col: ch(5).into(),
            row: ch(5).into(),
        });

        let back_to_scroll_adjusted_caret: ScrAdjCaret = (raw_caret, scr_ofs).into();

        assert_eq!(*back_to_scroll_adjusted_caret, *scr_adj_caret);
    }

    #[test]
    fn test_caret_conversion_to_scroll_adjusted() {
        let raw_caret: RawCaret = Pos {
            col: ch(5).into(),
            row: ch(5).into(),
        }
        .into();

        let caret: Caret = raw_caret.into();

        let Caret::Raw(raw_caret) = caret else {
            panic!("Expected RawCaret");
        };

        let scr_ofs = ScrOfs {
            col: ch(2).into(),
            row: ch(3).into(),
        };

        let scr_adj_caret: ScrAdjCaret = (raw_caret, scr_ofs).into();

        assert_eq!(*scr_adj_caret, Pos {
            col: ch(7).into(),
            row: ch(8).into(),
        });
    }

    #[test]
    fn test_caret_conversion_to_raw() {
        let scr_adj_caret: ScrAdjCaret = Pos {
            col: ch(7).into(),
            row: ch(8).into(),
        }
        .into();

        let caret: Caret = scr_adj_caret.into();

        let Caret::ScrollAdjusted(scr_adj_caret) = caret else {
            panic!("Expected ScrAdjCaret");
        };

        let scr_ofs = ScrOfs {
            col: ch(2).into(),
            row: ch(3).into(),
        };

        let raw_caret: RawCaret = (scr_adj_caret, scr_ofs).into();

        assert_eq!(*raw_caret, Pos {
            col: ch(5).into(),
            row: ch(5).into(),
        });
    }
}
