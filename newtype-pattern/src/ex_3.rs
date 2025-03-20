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

 #[derive(Debug, Clone, Copy, PartialEq)]
 pub struct Point {
     pub x: X,
     pub y: Y,
 }

 #[derive(Debug, Clone, Copy, PartialEq)]
 pub struct Size {
     pub width: Width,
     pub height: Height,
 }

 #[cfg(test)]
 mod tests {
     use super::*;

     #[test]
     fn test_point() {
         let x_val = X(0);
         let y_val = Y(0);
         let p_val = Point { y: y_val, x: x_val };
         assert_eq!(p_val.x, x_val);
         assert_eq!(p_val.y, y_val);
     }

     #[test]
     fn test_size() {
         let w_val = Width(10);
         let h_val = Height(10);
         let s_val = Size {
             width: w_val,
             height: h_val,
         };
         assert_eq!(s_val.width, w_val);
         assert_eq!(s_val.height, h_val);
     }

     // #[test]
     // fn does_not_compile() {
     //     let x_val = X(0);
     //     let y_val = Y(0);
     //     let p_val = Point { y: y_val, x: x_val };
     //     let w_val = Width(10);
     //     let h_val = Height(10);
     //     let s_val = Size {
     //         width: w_val,
     //         height: h_val,
     //     };
     //     let p_2_val = Point { x: y_val, y: x_val };
     //     let s_2_val = Size { width: h_val, height: w_val };
     // }
 }
