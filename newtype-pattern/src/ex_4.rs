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

// Constructor functions for `X`.

/// Note each constructor function takes a different type of argument. We have to define a
/// new function for each new type of argument.
#[allow(dead_code)]
mod too_many_functions {
    use super::*;

    pub fn x_alt(arg_x: usize) -> X {
        X(arg_x as Number)
    }

    pub fn x_alt_2(arg_x: u8) -> X {
        X(arg_x as Number)
    }

    pub fn x_alt_3(arg_x: &str) -> X {
        X(arg_x.parse().unwrap_or_default())
    }
}

#[allow(dead_code)]
pub fn x(arg_x: impl Into<X>) -> X {
    arg_x.into()
}

mod impl_into_x {
    use super::*;

    impl From<usize> for X {
        fn from(arg_x: usize) -> Self {
            X(arg_x as Number)
        }
    }

    impl From<u8> for X {
        fn from(arg_x: u8) -> Self {
            X(arg_x as Number)
        }
    }

    impl From<&str> for X {
        fn from(arg_x: &str) -> Self {
            X(arg_x.parse().unwrap_or_default())
        }
    }

    impl From<i32> for X {
        fn from(arg_x: i32) -> Self {
            X(arg_x as Number)
        }
    }
}

// Constructor functions for `Y`.

#[allow(dead_code)]
pub fn y(arg_y: impl Into<Y>) -> Y {
    arg_y.into()
}

mod impl_into_y {
    use super::*;

    impl From<usize> for Y {
        fn from(arg_y: usize) -> Self {
            Y(arg_y as Number)
        }
    }

    impl From<u8> for Y {
        fn from(arg_y: u8) -> Self {
            Y(arg_y as Number)
        }
    }

    impl From<&str> for Y {
        fn from(arg_y: &str) -> Self {
            Y(arg_y.parse().unwrap_or_default())
        }
    }

    impl From<i32> for Y {
        fn from(arg_y: i32) -> Self {
            Y(arg_y as Number)
        }
    }
}

// Constructor functions for `Width`.

#[allow(dead_code)]
pub fn width(arg_width: impl Into<Width>) -> Width {
    arg_width.into()
}

mod impl_into_width {
    use super::*;

    impl From<usize> for Width {
        fn from(arg_width: usize) -> Self {
            Width(arg_width as Number)
        }
    }

    impl From<u8> for Width {
        fn from(arg_width: u8) -> Self {
            Width(arg_width as Number)
        }
    }

    impl From<&str> for Width {
        fn from(arg_width: &str) -> Self {
            Width(arg_width.parse().unwrap_or_default())
        }
    }

    impl From<i32> for Width {
        fn from(arg_width: i32) -> Self {
            Width(arg_width as Number)
        }
    }
}

// Constructor functions for `Height`.

#[allow(dead_code)]
pub fn height(arg_height: impl Into<Height>) -> Height {
    arg_height.into()
}

mod impl_into_height {
    use super::*;

    impl From<usize> for Height {
        fn from(arg_height: usize) -> Self {
            Height(arg_height as Number)
        }
    }

    impl From<u8> for Height {
        fn from(arg_height: u8) -> Self {
            Height(arg_height as Number)
        }
    }

    impl From<&str> for Height {
        fn from(arg_height: &str) -> Self {
            Height(arg_height.parse().unwrap_or_default())
        }
    }

    impl From<i32> for Height {
        fn from(arg_height: i32) -> Self {
            Height(arg_height as Number)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_x() {
        // usize -> X.
        {
            let arg_x: usize = 10;
            let x_val = x(arg_x);
            assert_eq!(x_val, X(10));
        }

        // u8 -> X.
        {
            let arg_x: u8 = 20;
            let x_val = x(arg_x);
            assert_eq!(x_val, X(20));
        }

        // &str -> X.
        {
            let arg_x: &str = "30";
            let x_val = x(arg_x);
            assert_eq!(x_val, X(30));
        }

        // i32 -> X.
        {
            let arg_x: i32 = 40;
            let x_val = x(arg_x);
            assert_eq!(x_val, X(40));
        }
    }

    #[test]
    fn test_into_y() {
        // usize -> Y.
        {
            let arg_y: usize = 10;
            let y_val = y(arg_y);
            assert_eq!(y_val, Y(10));
        }

        // u8 -> Y.
        {
            let arg_y: u8 = 20;
            let y_val = y(arg_y);
            assert_eq!(y_val, Y(20));
        }

        // &str -> Y.
        {
            let arg_y: &str = "30";
            let y_val = y(arg_y);
            assert_eq!(y_val, Y(30));
        }

        // i32 -> Y.
        {
            let arg_y: i32 = 40;
            let y_val = y(arg_y);
            assert_eq!(y_val, Y(40));
        }
    }

    #[test]
    fn test_into_width() {
        // usize -> Width.
        {
            let arg_width: usize = 10;
            let width_val = width(arg_width);
            assert_eq!(width_val, Width(10));
        }

        // u8 -> Width.
        {
            let arg_width: u8 = 20;
            let width_val = width(arg_width);
            assert_eq!(width_val, Width(20));
        }

        // &str -> Width.
        {
            let arg_width: &str = "30";
            let width_val = width(arg_width);
            assert_eq!(width_val, Width(30));
        }

        // i32 -> Width.
        {
            let arg_width: i32 = 40;
            let width_val = width(arg_width);
            assert_eq!(width_val, Width(40));
        }
    }

    #[test]
    fn test_into_height() {
        // usize -> Height.
        {
            let arg_height: usize = 10;
            let height_val = height(arg_height);
            assert_eq!(height_val, Height(10));
        }

        // u8 -> Height.
        {
            let arg_height: u8 = 20;
            let height_val = height(arg_height);
            assert_eq!(height_val, Height(20));
        }

        // &str -> Height.
        {
            let arg_height: &str = "30";
            let height_val = height(arg_height);
            assert_eq!(height_val, Height(30));
        }

        // i32 -> Height.
        {
            let arg_height: i32 = 40;
            let height_val = height(arg_height);
            assert_eq!(height_val, Height(40));
        }
    }
}
