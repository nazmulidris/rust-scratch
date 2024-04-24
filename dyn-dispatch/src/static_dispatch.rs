/*
 *   Copyright (c) 2024 Nazmul Idris
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

use crate::fixtures::*;

mod receives {
    use super::*;

    /// Equivalent to [accept_error_2()].
    pub fn accept_error<T: Error>(arg: T) {
        let _ = arg.to_string();
    }

    /// Equivalent to [accept_error()]. The `impl` is just syntactic sugar.
    pub fn accept_error_2(arg: impl Error) {
        let _ = arg.to_string();
    }

    /// Static dispatch. Figures out at compile time.
    pub fn returns_single_error() -> impl Error {
        return ErrorOne {};
    }
}

mod returns {
    use super::*;

    /*
    /// This does not work. Since the caller can choose a different type than `ErrorOne`.
    pub fn returns_single_error_2<T: Error>() -> T {
        return ErrorOne {};
    }
    */

    /*
    /// This does not work. Since the compiler can't figure out which concrete type to return
    /// at compile time.
    pub fn returns_one_of_two_errors() -> impl Error {
        if random_boolean() {
            return ErrorOne {};
        } else {
            return ErrorTwo {};
        }
    }
    */
}
