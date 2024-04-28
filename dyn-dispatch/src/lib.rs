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
use std::error::Error;
use std::fmt::Display;

// ErrorOne.
mod error_one {
    use super::*;

    #[derive(Debug)]
    pub struct ErrorOne;

    impl Display for ErrorOne {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "ErrorOne")
        }
    }

    impl Error for ErrorOne {}
}
use error_one::ErrorOne;

// ErrorTwo.
mod error_two {
    use super::*;

    #[derive(Debug)]
    pub struct ErrorTwo;

    impl Display for ErrorTwo {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "ErrorTwo")
        }
    }

    impl Error for ErrorTwo {}
}
use error_two::ErrorTwo;

// Random boolean generator.
pub fn random_bool() -> bool {
    rand::random()
}

// Static dispatch.
mod static_dispatch {
    use super::*;

    mod receives {
        use super::*;

        pub fn accept_error<E: Error>(error: E) {
            println!("Handling ErrorOne Debug: {:?}", error);
            println!("Handling ErrorOne Display: {}", error);
        }

        pub fn accept_error_with_syntactic_sugar(error: impl Error) {
            println!("Handling ErrorOne Debug: {:?}", error);
            println!("Handling ErrorOne Display: {}", error);
        }
    }

    mod returns {
        use super::*;

        pub fn return_error_one() -> ErrorOne {
            ErrorOne
        }

        pub fn return_error_two() -> ErrorTwo {
            ErrorTwo
        }

        // DOES NOT WORK! Need dynamic dispatch.
        // pub fn return_single_error() -> impl Error {
        //     if random_bool() {
        //         ErrorOne
        //     } else {
        //         ErrorTwo
        //     }
        // }

        pub fn return_single_error() -> impl Error {
            return ErrorOne;
        }
    }
}

// Dynamic dispatch.
mod dynamic_dispatch {
    use super::*;

    mod receives {
        use super::*;

        pub fn recieve_error_by_ref(error: &dyn Error) {
            println!("Handling Error Debug: {:?}", error);
            println!("Handling Error Display: {}", error);
        }

        pub fn example_1() {
            let error_one = ErrorOne;
            recieve_error_by_ref(&error_one);
            let error_two = ErrorTwo;
            recieve_error_by_ref(&error_two);
        }

        pub fn receive_error_by_box(error: Box<dyn Error>) {
            println!("Handling Error Debug: {:?}", error);
            println!("Handling Error Display: {}", error);
        }

        pub fn example_2() {
            let error_one = ErrorOne;
            let it = Box::new(error_one);
            receive_error_by_box(it);
            let error_two = ErrorTwo;
            receive_error_by_box(Box::new(error_two));
        }

        pub fn receive_slice_of_errors(arg: &[&dyn Error]) {
            for error in arg {
                println!("Handling Error Debug: {:?}", error);
                println!("Handling Error Display: {}", error);
            }
        }
    }

    mod returns {
        use super::*;

        pub fn return_one_of_two_errors() -> Box<dyn Error> {
            if random_bool() {
                Box::new(ErrorOne)
            } else {
                Box::new(ErrorTwo)
            }
        }

        pub fn return_one_of_two_errors_with_arc() -> std::sync::Arc<dyn Error> {
            if random_bool() {
                std::sync::Arc::new(ErrorOne)
            } else {
                std::sync::Arc::new(ErrorTwo)
            }
        }

        pub fn return_slice_of_errors() -> Vec<&'static dyn Error> {
            let mut errors: Vec<&dyn Error> = vec![];
            if random_bool() {
                errors.push(&(ErrorOne));
            } else {
                errors.push(&(ErrorTwo));
            }
            errors
        }

        pub fn mut_vec_containing_different_types_of_errors(mut_vec: &mut Vec<&dyn Error>) {
            mut_vec.push(&ErrorOne);
            mut_vec.push(&ErrorTwo);
        }
    }
}
