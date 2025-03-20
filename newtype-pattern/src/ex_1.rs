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

 #[derive(Debug, Clone, Copy, PartialEq)]
 struct Point {
     pub x: usize,
     pub y: usize,
 }

 #[derive(Debug, Clone, Copy, PartialEq)]
 struct Size {
     pub width: usize,
     pub height: usize,
 }

 impl From<(/* x */ usize, /* y */ usize)> for Point {
     fn from(tuple: (usize, usize)) -> Self {
         Point {
             x: tuple.0,
             y: tuple.1,
         }
     }
 }

 impl From<(/* width */ usize, /* height */ usize)> for Size {
     fn from(tuple: (usize, usize)) -> Self {
         Size {
             width: tuple.0,
             height: tuple.1,
         }
     }
 }

 #[cfg(test)]
 mod tests {
     use super::*;

     #[allow(unused_variables)]
     #[test]
     fn test_simple() {
         let x = 0;
         let y = 0;
         let origin = Point { x, y };

         let width = 10;
         let height = 10;
         let size = Size { width, height };
     }

     #[test]
     #[should_panic]
     fn test_fails_off_by_one() {
         let size = Size {
             width: 100,
             height: 200,
         };

         // Should be Point { x : 99, y : 199 }
         let end_point = Point {
             x: size.width,
             y: size.height,
         };

         assert_eq!(end_point.x + 1, size.width);
         assert_eq!(end_point.y + 1, size.height);
     }

     #[test]
     #[should_panic]
     fn test_positional_arg_constr_problems() {
         let point_1: Point = (/* x */ 10, /* y */ 20).into();
         let size_1: Size = (/* width */ 50, /* height */ 100).into();

         // Mix up the x and y values. Fail.
         let point_2: Point = (point_1.y, point_1.x).into();
         assert_eq!(point_1, point_2);

         // Mix up the width and height values. Fail.
         let size_2: Size = (size_1.height, size_1.width).into();
         assert_eq!(size_1, size_2);

         // Mix up all the values. Fail.
         let point_3: Point = (size_1.height, point_1.x).into();
         assert_eq!(point_1, point_3);
     }
 }
