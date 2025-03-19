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
use super::ex_3::*;
use std::ops::Add;

#[allow(dead_code)]
pub fn point(arg_point: impl Into<Point>) -> Point {
    arg_point.into()
}

/// Ordering of the `X` and `Y` types does not matter. Support for both ways are provided
/// making the API really difficult to use incorrectly.
mod impl_into_point {
    use super::*;

    impl Add<X> for Y {
        type Output = Point;

        fn add(self, rhs: X) -> Self::Output {
            let y_val = self;
            let x_val = rhs;
            Point { x: x_val, y: y_val }
        }
    }

    impl Add<Y> for X {
        type Output = Point;

        fn add(self, rhs: Y) -> Self::Output {
            let x_val = self;
            let y_val = rhs;
            Point { x: x_val, y: y_val }
        }
    }
}

#[allow(dead_code)]
pub fn size(arg_size: impl Into<Size>) -> Size {
    arg_size.into()
}

/// Ordering of the `Width` and `Height` types does not matter. Support for both ways are
/// provided making the API really difficult to use incorrectly.
mod impl_into_size {
    use super::*;

    impl Add<Width> for Height {
        type Output = Size;

        fn add(self, rhs: Width) -> Self::Output {
            let height_val = self;
            let width_val = rhs;
            Size {
                width: width_val,
                height: height_val,
            }
        }
    }

    impl Add<Height> for Width {
        type Output = Size;

        fn add(self, rhs: Height) -> Self::Output {
            let width_val = self;
            let height_val = rhs;
            Size {
                width: width_val,
                height: height_val,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point() {
        let p_1 = Point { x: X(10), y: Y(20) };
        let p_2 = p_1.x + p_1.y;
        let p_3 = point(p_1.y + p_1.x);

        assert_eq!(p_1.x.0, 10);
        assert_eq!(p_1.y.0, 20);
        println!("{:?}", p_1);

        assert_eq!(p_1, p_2);
        assert_eq!(p_1, p_3);
    }

    #[test]
    fn test_size() {
        let s_1 = Size {
            width: Width(30),
            height: Height(40),
        };
        let s_2 = s_1.width + s_1.height;
        let s_3 = size(s_1.height + s_1.width);

        assert_eq!(s_1.width.0, 30);
        assert_eq!(s_1.height.0, 40);
        println!("{:?}", s_1);

        assert_eq!(s_1, s_2);
        assert_eq!(s_1, s_3);
    }
}
