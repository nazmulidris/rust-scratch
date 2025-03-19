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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point {
    pub x: X,
    pub y: Y,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Size {
    pub width: Width,
    pub height: Height,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point() {
        let p = Point { x: X(10), y: Y(20) };
        assert_eq!(p.x.0, 10);
        assert_eq!(p.y.0, 20);
        println!("{:?}", p);
    }

    #[test]
    fn test_size() {
        let s = Size {
            width: Width(30),
            height: Height(40),
        };
        assert_eq!(s.width.0, 30);
        assert_eq!(s.height.0, 40);
        println!("{:?}", s);
    }
}
