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

 #[allow(dead_code)]
 mod too_many_fns {
     use super::*;

     pub fn x_alt(arg_x: usize) -> X {
         X(arg_x as Number)
     }

     pub fn x_alt_2(arg_x: u16) -> X {
         X(arg_x)
     }

     pub fn x_alt_3(arg_x: u32) -> X {
         X(arg_x as Number)
     }

     pub fn x_alt_4(arg_x: &str) -> X {
         X(arg_x.parse::<u16>().unwrap_or_default())
     }
 }

 // Constructor fn for `X`.

 #[allow(dead_code)]
 pub fn x(arg_x: impl Into<X>) -> X {
     arg_x.into()
 }

 mod into_impl_x {
     use super::*;

     impl From<usize> for X {
         fn from(arg_x: usize) -> Self {
             X(arg_x as Number)
         }
     }

     impl From<u16> for X {
         fn from(arg_x: u16) -> Self {
             X(arg_x)
         }
     }

     impl From<u32> for X {
         fn from(arg_x: u32) -> Self {
             X(arg_x as Number)
         }
     }

     impl From<&str> for X {
         fn from(arg_x: &str) -> Self {
             X(arg_x.parse::<u16>().unwrap_or_default())
         }
     }

     impl From<i32> for X {
         fn from(arg_x: i32) -> Self {
             X(arg_x as Number)
         }
     }
 }

 // Constructor fn for `Y`.

 #[allow(dead_code)]
 pub fn y(arg_y: impl Into<Y>) -> Y {
     arg_y.into()
 }

 mod into_impl_y {
     use super::*;

     impl From<usize> for Y {
         fn from(arg_y: usize) -> Self {
             Y(arg_y as Number)
         }
     }

     impl From<u16> for Y {
         fn from(arg_y: u16) -> Self {
             Y(arg_y)
         }
     }

     impl From<u32> for Y {
         fn from(arg_y: u32) -> Self {
             Y(arg_y as Number)
         }
     }

     impl From<&str> for Y {
         fn from(arg_y: &str) -> Self {
             Y(arg_y.parse::<u16>().unwrap_or_default())
         }
     }

     impl From<i32> for Y {
         fn from(arg_y: i32) -> Self {
             Y(arg_y as Number)
         }
     }
 }

 // Constructor fn for `Width`.

 #[allow(dead_code)]
 pub fn width(arg_width: impl Into<Width>) -> Width {
     arg_width.into()
 }

 mod into_impl_width {
     use super::*;

     impl From<usize> for Width {
         fn from(arg_width: usize) -> Self {
             Width(arg_width as Number)
         }
     }

     impl From<u16> for Width {
         fn from(arg_width: u16) -> Self {
             Width(arg_width)
         }
     }

     impl From<u32> for Width {
         fn from(arg_width: u32) -> Self {
             Width(arg_width as Number)
         }
     }

     impl From<&str> for Width {
         fn from(arg_width: &str) -> Self {
             Width(arg_width.parse::<u16>().unwrap_or_default())
         }
     }

     impl From<i32> for Width {
         fn from(arg_width: i32) -> Self {
             Width(arg_width as Number)
         }
     }
 }

 // Constructor fn for `Height`.

 #[allow(dead_code)]
 pub fn height(arg_height: impl Into<Height>) -> Height {
     arg_height.into()
 }

 mod into_impl_height {
     use super::*;

     impl From<usize> for Height {
         fn from(arg_height: usize) -> Self {
             Height(arg_height as Number)
         }
     }

     impl From<u16> for Height {
         fn from(arg_height: u16) -> Self {
             Height(arg_height)
         }
     }

     impl From<u32> for Height {
         fn from(arg_height: u32) -> Self {
             Height(arg_height as Number)
         }
     }

     impl From<&str> for Height {
         fn from(arg_height: &str) -> Self {
             Height(arg_height.parse::<u16>().unwrap_or_default())
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
     fn test_x() {
         {
             let x_val_1 = x(0usize);
             let x_val_2 = x(0u16);
             let x_val_3 = x(0u32);
             let x_val_4 = x("0");
             assert_eq!(x_val_1, x_val_2);
             assert_eq!(x_val_1, x_val_3);
             assert_eq!(x_val_1, x_val_4);
         }

         {
             let x_val_1: X = 0usize.into();
             let x_val_2: X = 0u16.into();
             let x_val_3: X = 0u32.into();
             let x_val_4: X = "0".into();
             assert_eq!(x_val_1, x_val_2);
             assert_eq!(x_val_1, x_val_3);
             assert_eq!(x_val_1, x_val_4);
         }
     }

     #[test]
     fn test_y() {
         {
             let y_val_1 = y(0usize);
             let y_val_2 = y(0u16);
             let y_val_3 = y(0u32);
             let y_val_4 = y("0");
             assert_eq!(y_val_1, y_val_2);
             assert_eq!(y_val_1, y_val_3);
             assert_eq!(y_val_1, y_val_4);
         }

         {
             let y_val_1: Y = 0usize.into();
             let y_val_2: Y = 0u16.into();
             let y_val_3: Y = 0u32.into();
             let y_val_4: Y = "0".into();
             assert_eq!(y_val_1, y_val_2);
             assert_eq!(y_val_1, y_val_3);
             assert_eq!(y_val_1, y_val_4);
         }
     }

     #[test]
     fn test_width() {
         {
             let width_val_1 = width(0usize);
             let width_val_2 = width(0u16);
             let width_val_3 = width(0u32);
             let width_val_4 = width("0");
             assert_eq!(width_val_1, width_val_2);
             assert_eq!(width_val_1, width_val_3);
             assert_eq!(width_val_1, width_val_4);
         }

         {
             let width_val_1: Width = 0usize.into();
             let width_val_2: Width = 0u16.into();
             let width_val_3: Width = 0u32.into();
             let width_val_4: Width = "0".into();
             assert_eq!(width_val_1, width_val_2);
             assert_eq!(width_val_1, width_val_3);
             assert_eq!(width_val_1, width_val_4);
         }
     }

     #[test]
     fn test_height() {
         {
             let height_val_1 = height(0usize);
             let height_val_2 = height(0u16);
             let height_val_3 = height(0u32);
             let height_val_4 = height("0");
             assert_eq!(height_val_1, height_val_2);
             assert_eq!(height_val_1, height_val_3);
             assert_eq!(height_val_1, height_val_4);
         }

         {
             let height_val_1: Height = 0usize.into();
             let height_val_2: Height = 0u16.into();
             let height_val_3: Height = 0u32.into();
             let height_val_4: Height = "0".into();
             assert_eq!(height_val_1, height_val_2);
             assert_eq!(height_val_1, height_val_3);
             assert_eq!(height_val_1, height_val_4);
         }
     }
 }
