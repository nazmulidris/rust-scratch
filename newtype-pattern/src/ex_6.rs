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

// Avoid off by 1 errors in conversions between length and index.

impl From<Width> for X {
    fn from(arg_width: Width) -> Self {
        // No underflow.
        X(arg_width.0.saturating_sub(1))
    }
}

impl From<Height> for Y {
    fn from(arg_height: Height) -> Self {
        // No underflow.
        Y(arg_height.0.saturating_sub(1))
    }
}

impl From<X> for Width {
    fn from(arg_x: X) -> Self {
        Width(arg_x.0 + 1)
    }
}
impl From<Y> for Height {
    fn from(arg_y: Y) -> Self {
        Height(arg_y.0 + 1)
    }
}

// Formalize the relationship between length and index comparisons in a trait.

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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CheckInBoundsResult {
    InBounds,
    OutOfBounds,
}

impl CheckInBounds<X, Width> for X {
    fn check_in_bounds(&self, arg_other: impl Into<Width>) -> CheckInBoundsResult {
        let other = arg_other.into();
        if self.0 < other.0 {
            CheckInBoundsResult::InBounds
        } else {
            CheckInBoundsResult::OutOfBounds
        }
    }
}

impl CheckInBounds<Y, Height> for Y {
    fn check_in_bounds(&self, arg_other: impl Into<Height>) -> CheckInBoundsResult {
        let other = arg_other.into();
        if self.0 < other.0 {
            CheckInBoundsResult::InBounds
        } else {
            CheckInBoundsResult::OutOfBounds
        }
    }
}

#[cfg(test)]
mod test_conversions {
    use super::*;
    use crate::ex_4::{height, width, x, y};

    #[test]
    fn test_width_to_x() {
        {
            let w_val = width(10);
            let x_val: X = w_val.into();
            assert_eq!(x_val, 9.into());
        }

        {
            let w_val = width(0);
            let x_val: X = w_val.into();
            assert_eq!(x_val, x(0));
        }
    }

    #[test]
    fn test_height_to_y() {
        {
            let h_val = height(20);
            let y_val: Y = h_val.into();
            assert_eq!(y_val, 19.into());
        }

        {
            let h_val = height(0);
            let y_val: Y = h_val.into();
            assert_eq!(y_val, y(0));
        }
    }

    #[test]
    fn test_x_to_width() {
        {
            let x_val = X(10);
            let w_val: Width = x_val.into();
            assert_eq!(w_val, 11.into());
        }

        {
            let x_val = X(0);
            let w_val: Width = x_val.into();
            assert_eq!(w_val, width(1));
        }
    }

    #[test]
    fn test_y_to_height() {
        {
            let y_val = Y(20);
            let h_val: Height = y_val.into();
            assert_eq!(h_val, 21.into());
        }

        {
            let y_val = Y(0);
            let h_val: Height = y_val.into();
            assert_eq!(h_val, height(1));
        }
    }
}

#[cfg(test)]
mod test_check_in_bounds {
    use super::*;

    #[test]
    fn test_check_in_bounds_x_and_width() {
        let x_val = X(10);
        let w_val = Width(20);
        assert_eq!(x_val.check_in_bounds(w_val), CheckInBoundsResult::InBounds);

        let x_val = X(20);
        assert_eq!(
            x_val.check_in_bounds(w_val),
            CheckInBoundsResult::OutOfBounds
        );
    }

    #[test]
    fn test_check_in_bounds_y_and_height() {
        let y_val = Y(10);
        let h_val = Height(20);
        assert_eq!(y_val.check_in_bounds(h_val), CheckInBoundsResult::InBounds);

        let y_val = Y(20);
        assert_eq!(
            y_val.check_in_bounds(h_val),
            CheckInBoundsResult::OutOfBounds
        );
    }
}
