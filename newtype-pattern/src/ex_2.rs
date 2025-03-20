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

 pub type Number = u16;

 use std::fmt::Debug;
 use std::ops::{Deref, DerefMut};

 #[derive(Copy, Clone, PartialEq, PartialOrd)]
 pub struct X(pub Number);

 mod x_ops {
     use super::*;

     impl Deref for X {
         type Target = Number;

         fn deref(&self) -> &Self::Target {
             &self.0
         }
     }

     impl DerefMut for X {
         fn deref_mut(&mut self) -> &mut Self::Target {
             &mut self.0
         }
     }

     impl Debug for X {
         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
             write!(f, "X({})", self.0)
         }
     }
 }

 #[derive(Clone, Copy, PartialEq, PartialOrd)]
 pub struct Y(pub Number);

 mod y_ops {
     use super::*;

     impl Deref for Y {
         type Target = Number;

         fn deref(&self) -> &Self::Target {
             &self.0
         }
     }

     impl DerefMut for Y {
         fn deref_mut(&mut self) -> &mut Self::Target {
             &mut self.0
         }
     }

     impl Debug for Y {
         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
             write!(f, "Y({})", self.0)
         }
     }
 }

 #[derive(Clone, Copy, PartialEq, PartialOrd)]
 pub struct Width(pub Number);

 mod width_ops {
     use super::*;

     impl Deref for Width {
         type Target = Number;

         fn deref(&self) -> &Self::Target {
             &self.0
         }
     }

     impl DerefMut for Width {
         fn deref_mut(&mut self) -> &mut Self::Target {
             &mut self.0
         }
     }

     impl Debug for Width {
         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
             write!(f, "Width({})", self.0)
         }
     }
 }

 #[derive(Clone, Copy, PartialEq, PartialOrd)]
 pub struct Height(pub Number);

 mod height_ops {
     use super::*;

     impl Deref for Height {
         type Target = Number;

         fn deref(&self) -> &Self::Target {
             &self.0
         }
     }

     impl DerefMut for Height {
         fn deref_mut(&mut self) -> &mut Self::Target {
             &mut self.0
         }
     }

     impl Debug for Height {
         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
             write!(f, "Height({})", self.0)
         }
     }
 }

 #[cfg(test)]
 mod tests {
     use super::*;

     #[test]
     fn test_x() {
         let mut x = X(10);

         // Deref.
         assert_eq!(*x, 10);

         assert_eq!(x.0, 10);
         assert_eq!(x, X(10));

         // DerefMut.
         *x = 100;

         assert_eq!(format!("{:?}", x), "X(100)");
     }

     #[test]
     fn test_y() {
         let mut y = Y(20);

         // Deref.
         assert_eq!(*y, 20);

         assert_eq!(y.0, 20);
         assert_eq!(y, Y(20));

         // DerefMut.
         *y = 200;

         assert_eq!(format!("{:?}", y), "Y(200)");
     }

     #[test]
     fn test_width() {
         let mut width = Width(30);

         // Deref.
         assert_eq!(*width, 30);

         assert_eq!(width.0, 30);
         assert_eq!(width, Width(30));

         // DerefMut.
         *width = 300;

         assert_eq!(format!("{:?}", width), "Width(300)");
     }

     #[test]
     fn test_height() {
         let mut height = Height(40);

         // Deref.
         assert_eq!(*height, 40);

         assert_eq!(height.0, 40);
         assert_eq!(height, Height(40));

         // DerefMut.
         *height = 400;

         assert_eq!(format!("{:?}", height), "Height(400)");
     }
 }
