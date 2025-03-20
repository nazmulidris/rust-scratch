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

 mod impl_ops_point {
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

 mod impl_ops_size {
     use super::*;

     impl Add<Width> for Height {
         type Output = Size;

         fn add(self, rhs: Width) -> Self::Output {
             let h_val = self;
             let w_val = rhs;
             Size {
                 width: w_val,
                 height: h_val,
             }
         }
     }

     impl Add<Height> for Width {
         type Output = Size;

         fn add(self, rhs: Height) -> Self::Output {
             let w_val = self;
             let h_val = rhs;
             Size {
                 width: w_val,
                 height: h_val,
             }
         }
     }
 }

 #[cfg(test)]
 mod tests {
     use super::*;

     #[test]
     fn test_point() {
         let x_val = X(0);
         let y_val = Y(0);

         let p_val_1 = x_val + y_val;
         let p_val_2 = y_val + x_val;

         assert_eq!(p_val_1, p_val_2);

         let p_val_3 = point(x_val + y_val);
         let p_val_4 = point(y_val + x_val);

         assert_eq!(p_val_3, p_val_4);
     }

     #[test]
     fn test_size() {
         let w_val = Width(10);
         let h_val = Height(10);

         let s_val_1 = w_val + h_val;
         let s_val_2 = h_val + w_val;

         assert_eq!(s_val_1, s_val_2);

         let s_val_3 = size(w_val + h_val);
         let s_val_4 = size(h_val + w_val);

         assert_eq!(s_val_3, s_val_4);
     }
 }
