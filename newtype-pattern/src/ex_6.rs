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

use super::ex_2::*;

impl From<Width> for X {
    fn from(width: Width) -> Self {
        X(/* width - 1 */ width.0.saturating_sub(1))
    }
}

impl From<Height> for Y {
    fn from(height: Height) -> Self {
        Y(/* height - 1 */ height.0.saturating_sub(1))
    }
}

impl From<X> for Width {
    fn from(x: X) -> Self {
        Width(/* x + 1 */ x.0 + 1)
    }
}

impl From<Y> for Height {
    fn from(y: Y) -> Self {
        Height(/* y + 1 */ y.0 + 1)
    }
}

#[cfg(test)]
mod test_conversions {
    use crate::{
        ex_2::{Height, Width, X, Y},
        ex_4::{height, width, x, y},
    };

    #[test]
    fn test_width_to_x() {
        let w_val = width(10);
        let x_val: X = w_val.into();
        assert_eq!(x_val, x(9));
    }

    #[test]
    fn test_height_to_y() {
        let h_val = height(10);
        let y_val: Y = h_val.into();
        assert_eq!(y_val, y(9));
    }

    #[test]
    fn test_x_to_width() {
        let x_val = x(10);
        let w_val: Width = x_val.into();
        assert_eq!(w_val, width(11));
    }

    #[test]
    fn test_y_to_height() {
        let y_val = y(10);
        let h_val: Height = y_val.into();
        assert_eq!(h_val, height(11));
    }
}

#[allow(dead_code)]
pub trait CheckInBounds<Index, Length>
where
    Self: Into<Index>,
{
    fn check_in_bounds(
        /* Index */ &self,
        /* Length */ arg_length: impl Into<Length>,
    ) -> CheckInBoundsResult;
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CheckInBoundsResult {
    InBounds,
    OutOfBounds,
}

impl CheckInBounds<X, Width> for X {
    fn check_in_bounds(
        /* Index */ &self,
        /* Length */ arg_length: impl Into<Width>,
    ) -> CheckInBoundsResult {
        let self_index = **self;
        let other_length = *arg_length.into();
        if self_index < other_length {
            CheckInBoundsResult::InBounds
        } else {
            CheckInBoundsResult::OutOfBounds
        }
    }
}

impl CheckInBounds<Y, Height> for Y {
    fn check_in_bounds(
        /* Index */ &self,
        /* Length */ arg_length: impl Into<Height>,
    ) -> CheckInBoundsResult {
        let self_index = **self;
        let other_length = *arg_length.into();
        if self_index < other_length {
            CheckInBoundsResult::InBounds
        } else {
            CheckInBoundsResult::OutOfBounds
        }
    }
}

#[cfg(test)]
mod test_check_in_bounds {
    use super::*;
    use crate::ex_4::{height, width, x, y};

    #[test]
    fn test_check_in_bounds_x_and_width() {
        let x_val = x(10);
        let w_val = width(20);
        assert_eq!(x_val.check_in_bounds(w_val), CheckInBoundsResult::InBounds);

        let x_val = x(20);
        assert_eq!(
            x_val.check_in_bounds(w_val),
            CheckInBoundsResult::OutOfBounds
        );
    }

    #[test]
    fn test_check_in_bounds_y_and_height() {
        let y_val = y(10);
        let h_val = height(20);
        assert_eq!(y_val.check_in_bounds(h_val), CheckInBoundsResult::InBounds);

        let y_val = y(20);
        assert_eq!(
            y_val.check_in_bounds(h_val),
            CheckInBoundsResult::OutOfBounds
        );
    }
}
