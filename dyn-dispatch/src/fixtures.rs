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

pub use std::error::Error;
pub use std::fmt;
pub use std::fmt::{Display, Formatter};
pub use std::sync::Arc;

pub mod random_boolean {
    use rand::Rng;

    pub fn random_boolean() -> bool {
        let mut rng = rand::thread_rng();
        rng.gen()
    }
}
pub use random_boolean::*;

pub mod error_one {
    use super::*;

    #[derive(Debug)]
    pub struct ErrorOne;

    impl Display for ErrorOne {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "ErrorOne")
        }
    }

    impl Error for ErrorOne {}
}
pub use error_one::*;

pub mod error_two {
    use super::*;

    #[derive(Debug)]
    pub struct ErrorTwo;

    impl Display for ErrorTwo {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "ErrorTwo")
        }
    }

    impl Error for ErrorTwo {}
}
pub use error_two::*;
