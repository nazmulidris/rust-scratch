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

#[allow(dead_code)]

/// In case you want to change how the `x` and `y` fields are displayed on their own, you
/// can't do this: `impl std::fmt::Display for usize { .. }`
mod simplistic {
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct Point {
        /// 0 index.
        pub x: usize,
        /// 0 index.
        pub y: usize,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Size {
        /// Length of the rectangle.
        pub width: usize,
        /// Height of the rectangle.
        pub height: usize,
    }
}

mod fancy_constructor {
    use super::simplistic::*;

    impl From<(usize, usize)> for Point {
        fn from((x, y): (usize, usize)) -> Self {
            Point { x, y }
        }
    }

    impl From<(usize, usize)> for Size {
        fn from((width, height): (usize, usize)) -> Self {
            Size { width, height }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::simplistic::*;

    #[test]
    fn test_passes() {
        let x = 0;
        let y = 0;
        let origin = Point { x, y };

        let width = 100;
        let height = 200;
        let size = Size { width, height };

        println!("origin: {:?}", origin);
        println!("size: {:?}", size);
    }

    /// So easy to make off by 1 errors.
    #[test]
    fn test_fails() {
        let size = Size {
            width: 100,
            height: 200,
        };

        // Should be Point { x: 99, y: 199 }.
        let end = Point {
            x: size.width,
            y: size.height,
        };

        assert_eq!(end.x + 1, size.width);
        assert_eq!(end.y + 1, size.height);
    }

    /// Positional arguments are easy to mix up. Also, doesn't help when the type of every
    /// argument is a `usize`.
    #[test]
    fn test_positional_arg_constructor_problems() {
        let point_1: Point = (10, 20).into();
        let size_1: Size = (50, 100).into();

        // Mix up the x and y values. Fails.
        let point_2: Point = (point_1.y, point_1.x).into();
        assert_eq!(point_1, point_2);

        // Mix up the width and height values. Fails.
        let point_3: Point = (size_1.height, size_1.width).into();
        assert_eq!(point_3, (size_1.width, size_1.height).into());

        // Mix up all of the values. Fails.
        let point_4: Point = (size_1.height, point_1.x).into();
        assert_eq!(point_1, point_4);
    }
}
