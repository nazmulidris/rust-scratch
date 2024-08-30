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

/// The mutual exclusion property of mutable references can be very limiting when working
/// with a composite structure. The borrow checker understand structs sufficiently to know
/// that it's possible to borrow disjoint fields of a struct simultaneously.
#[test]
fn ex_5_splitting_borrows_on_structs() {
    struct Data {
        a: usize,
        b: usize,
    }

    fn change_field_by_ref(field: &mut usize) {
        *field += 1;
    }

    let mut data = Data { a: 1, b: 2 };

    let a_ref = &mut data.a;
    let b_ref = &mut data.b;

    change_field_by_ref(a_ref);
    change_field_by_ref(b_ref);

    assert_eq!(data.a, 2);
    assert_eq!(data.b, 3);
}

/// This example shows a struct that only contains references. As long as the owned struct
/// and the references live for the same lifetime, it all works.
#[test]
fn ex_5_splitting_borrows_on_structs_2() {
    struct Data<'a> {
        field_usize: &'a mut usize,
        field_str: &'a str,
    }

    impl Data<'_> {
        fn new<'a>(str_param: &'a str, usize_param: &'a mut usize) -> Data<'a> {
            Data {
                field_usize: usize_param,
                field_str: str_param,
            }
        }

        fn change_field_usize(&mut self) {
            *self.field_usize += 1;
        }

        fn change_field_str(&mut self) {
            self.field_str = "new value";
        }
    }

    let str_arg = "old value";
    let usize_arg = &mut 1;
    let mut data = Data::new(str_arg, usize_arg);

    data.change_field_usize();
    data.change_field_str();

    assert_eq!(*data.field_usize, 2);
    assert_eq!(data.field_str, "new value");
}
