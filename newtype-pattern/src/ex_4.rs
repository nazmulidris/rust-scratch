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

use super::{ex_2::*, ex_3::*};

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

// TODO: Constructor functions for `Y`.

// TODO: Constructor functions for `Width`.

// TODO: Constructor functions for `Height`.
