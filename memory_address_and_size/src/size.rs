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

// TODO: use std::mem::size_of
// TODO: use size-of crate

//! Different ways of getting the size of a variable, in Rust:
//! - std::mem::size_of_val
//! - size-of crate

#[test]
fn get_size_of_using_std() {
    use super::test_utils::fixtures::*;

    // [Scalar type] This works for scalar types like `usize`, `u8`, `i32`, etc. And
    // compound types that are statically sized like: arrays, tuples, and (sized) structs.
    // This is just 1 byte for u8.
    let val: u8 = 42;
    let size_of_scalar = std::mem::size_of_val(&val);
    println!("Size of scalar {}: {}", val, format_size(size_of_scalar));

    // [Vec type] For a `Vec`, size_of_val will return the size of the `Vec` structure
    // itself, which includes: 1. the pointer to the heap-allocated data, 2. the length,
    // and 3. the capacity. It does not not include the size of the elements stored in the
    // vector. No matter how many items are in the vector, the size of the vector itself
    // is always 24 bytes, which is 3 x 64 bits (my usize).
    let val: Vec<u8> = vec![1; 100];
    let size_of_stack_alloc_vec_no_backing_store = std::mem::size_of_val(&val);
    println!(
        "Size of vec {} items: {}",
        val.len(),
        format_size(size_of_stack_alloc_vec_no_backing_store)
    );

    // [Structure type] For a sized structure, size_of_val will return the size of the
    // structure itself, that is allocated on the stack. In this case the size of the
    // structure is 4 bytes.
    #[allow(dead_code)]
    struct MyStruct {
        a: u8,
        b: u8,
        c: u8,
        e: u8,
    }
    let size_of_struct = std::mem::size_of_val(&MyStruct {
        a: 42,
        b: 43,
        c: 44,
        e: 45,
    });
    println!(
        "Size of struct {:?}: {}",
        size_of_struct,
        format_size(size_of_struct)
    );
}

#[test]
fn get_size_of_using_size_of_crate() {
    use super::test_utils::fixtures::*;
    use size_of::SizeOf as _;

    // Scalar type.
    let val: u8 = 42;
    let size_of_scalar = val.size_of().total_bytes();
    println!("Size of scalar {}: {}", val, format_size(size_of_scalar));

    // Vec type.
    let val: Vec<u8> = vec![1; 100];
    let size_of_heap_alloc_with_backing_store_and_stack_alloc_vec_struct =
        val.size_of().total_bytes();
    println!(
        "Size of Vec<u8>; {} items: {}",
        val.len(),
        format_size(size_of_heap_alloc_with_backing_store_and_stack_alloc_vec_struct)
    );

    // Struct type.
    #[allow(dead_code)]
    #[derive(size_of::SizeOf, Clone, Copy)]
    struct MyStruct {
        a: u8,
        b: u8,
        c: u8,
        e: u8,
        f: MyStruct2,
    }
    #[allow(dead_code)]
    #[derive(size_of::SizeOf, Clone, Copy)]
    struct MyStruct2 {
        a: u8,
        b: u8,
        c: u8,
        e: u8,
    }
    let vec = vec![
        MyStruct {
            a: 42,
            b: 43,
            c: 44,
            e: 45,
            f: MyStruct2 {
                a: 42,
                b: 43,
                c: 44,
                e: 45,
            },
        };
        100
    ];
    let size_of_struct = vec.size_of().total_bytes();
    println!(
        "Size of Vec<MyStruct>; {} items: {}",
        vec.len(),
        format_size(size_of_struct)
    );
}
