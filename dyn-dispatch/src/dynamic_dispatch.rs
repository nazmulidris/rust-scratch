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

use super::fixtures::*;

mod returns {
    use super::*;

    pub fn returns_one_of_two_errors() -> Box<dyn Error> {
        if random_boolean() {
            return Box::new(ErrorOne {});
        } else {
            return Box::new(ErrorTwo {});
        }
    }

    pub fn returns_one_of_two_errors_2() -> Arc<dyn Error> {
        if random_boolean() {
            return Arc::new(ErrorOne {});
        } else {
            return Arc::new(ErrorTwo {});
        }
    }

    pub fn vec_containing_different_types_of_same_trait() -> Vec<&'static dyn Error> {
        // The key is the `&` before `dyn Error`.
        let mut vec: Vec<&dyn Error> = Vec::new();
        vec.push(&ErrorOne {});
        vec.push(&ErrorTwo {});
        return vec;
    }
}

mod receives {
    use super::*;

    pub fn receive_error(arg: Box<dyn Error>) {
        let _ = arg.to_string();
    }

    pub fn receive_slice_of_errors(arg: &[&dyn Error]) {
        for error in arg {
            let _ = error.to_string();
        }
    }
}
