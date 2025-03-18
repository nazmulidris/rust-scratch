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

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

pub type Number = u16;

#[derive(Copy, Clone, PartialEq)]
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

    /// Can implement `Debug` for `X` here. But can't implement `Debug` for `Number` due
    /// to orphan rule.
    impl Debug for X {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "X({})", self.0)
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
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

    /// Can implement `Debug` for `Y` here. But can't implement `Debug` for `Number` due
    /// to orphan rule.
    impl Debug for Y {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Y({})", self.0)
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
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

    /// Can implement `Debug` for `Width` here. But can't implement `Debug` for `Number` due
    /// to orphan rule.
    impl Debug for Width {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Width({})", self.0)
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
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

    /// Can implement `Debug` for `Height` here. But can't implement `Debug` for `Number` due
    /// to orphan rule.
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
        // Direct access.
        {
            let x = X(0);
            assert_eq!(x.0, 0);
        }

        // Deref.
        {
            let x = X(0);
            let it = *x;
            assert_eq!(it, 0);
        }

        // DerefMut.
        {
            let mut x = X(0);
            *x = 1;
            assert_eq!(*x, 1);
        }

        // Debug.
        {
            let x = X(0);
            assert_eq!(format!("{:?}", x), "X(0)");
        }
    }

    #[test]
    fn test_y() {
        // Direct access.
        {
            let y = Y(0);
            assert_eq!(y.0, 0);
        }

        // Deref.
        {
            let y = Y(0);
            let it = *y;
            assert_eq!(it, 0);
        }

        // DerefMut.
        {
            let mut y = Y(0);
            *y = 1;
            assert_eq!(*y, 1);
        }

        // Debug.
        {
            let y = Y(0);
            assert_eq!(format!("{:?}", y), "Y(0)");
        }
    }

    #[test]
    fn test_width() {
        // Direct access.
        {
            let width = Width(0);
            assert_eq!(width.0, 0);
        }

        // Deref.
        {
            let width = Width(0);
            let it = *width;
            assert_eq!(it, 0);
        }

        // DerefMut.
        {
            let mut width = Width(0);
            *width = 1;
            assert_eq!(*width, 1);
        }

        // Debug.
        {
            let width = Width(0);
            assert_eq!(format!("{:?}", width), "Width(0)");
        }
    }

    #[test]
    fn test_height() {
        // Direct access.
        {
            let height = Height(0);
            assert_eq!(height.0, 0);
        }

        // Deref.
        {
            let height = Height(0);
            let it = *height;
            assert_eq!(it, 0);
        }

        // DerefMut.
        {
            let mut height = Height(0);
            *height = 1;
            assert_eq!(*height, 1);
        }

        // Debug.
        {
            let height = Height(0);
            assert_eq!(format!("{:?}", height), "Height(0)");
        }
    }
}
