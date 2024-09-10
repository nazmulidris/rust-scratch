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

//! # Different ways of getting a memory address, for a variable on the stack, in Rust.
//!
//! ## *const T (Immutable Raw Pointer)
//!
//! In the code below `as *const usize` converts the reference to a raw pointer of type
//! `*const usize`.
//! - That is, `*const T` represents a raw pointer to an immutable value of type `T`.
//! - You cannot modify the value that the pointer points to through a `*const T`.
//!   - The `const` marks that it is not possible to modify (the value) `T` using this raw
//!     pointer,
//!   - It is similar to a reference `&T`, which also provides read-only access to the
//!     value.
//!
//! ## *mut T (Mutable Raw Pointer)
//!
//! In the code below `as *mut usize` converts the reference to a raw pointer of type
//! `*mut usize`.
//!  - I.e., `*mut T` represents a raw pointer to a mutable value of type `T`.
//! - You can modify the value that the pointer points to through a `*mut T`.
//!   - It is similar to a mutable reference `&mut T`, which allows for read and write
//!     access to the value.
//!
//! ## std::ptr::addr_of!
//!
//! In the code below, the `std::ptr::addr_of!` macro is used to get the address of a
//! variable, instead of the `as *const usize` and `as *mut usize` syntax.
//!
//! ## std::ptr::addr_of_mut!
//!
//! In the code below, the `std::ptr::addr_of_mut!` macro is used to get the mutable
//! address of a variable, instead of the `as *mut usize` syntax.
//!
//! ## println!("{:p}", &val);
//!
//! In the code below, the `println!("{:p}", &val);` syntax is used to print the address
//! of a variable.
//!
//! # Different ways of getting a memory address, for a variable on the heap, in Rust.
//!
//! ## Vec::as_ptr
//!
//! In the code below, the `Vec::as_ptr` method is used to get the raw pointer to the
//! heap-allocated memory. Other ways to do this are to convert `Vec<T>` to `Box<[T]>` and
//! get the raw pointer to the heap-allocated memory.
//!
//! # Box::as_ref
//!
//! | Syntax        | Type      | Description                                                                |
//! |---------------|-----------|----------------------------------------------------------------------------|
//! | `&my_box`     | `&Box<T>` | Use to get the addr of the `Box` itself, which is stack-allocated          |
//! | `Box::as_ref` | `&T`      | Use to get the addr of the value (inside the `Box`)) that's heap-allocated |
//!
//! Other ways to do this are to convert `Box<T>` to `&T` and get the raw pointer to the
//! heap-allocated memory using `Box::into_raw`.

#[test]
fn get_address_of_var_on_stack() {
    use super::test_utils::fixtures::*;

    let mut val: usize = 10;

    // Approach 1.1 - use `as *const T`.
    let ptr = &val as *const usize;
    println!("Address of val: {}", format_address(ptr));

    // Approach 1.2 - use `std::ptr::addr_of!`.
    let ptr_2 = std::ptr::addr_of!(val);
    println!("Address of val: {}", format_address(ptr_2));

    // Approach 2.1 - use `as *mut T`.
    let ptr_3 = &mut val as *mut usize;
    println!("Address of val: {}", format_address(ptr_3));

    // Approach 2.2 - use `std::ptr::addr_of_mut!`.
    let ptr_4 = std::ptr::addr_of_mut!(val);
    println!("Address of val: {}", format_address(ptr_4));

    // Approach 3 - use `println!("{:p}", &val)`.
    let ptr_5 = format!("{:p}", &val);
    println!("Address of val: {}", ptr_5.clone().bold().white());

    // Make assertions.
    assert_eq!(ptr, ptr_2);
    assert_eq!(ptr, ptr_3);
    assert_eq!(ptr, ptr_4);
    assert_eq!(format!("{:p}", ptr), ptr_5);
}

#[test]
fn get_address_of_box_on_heap() {
    use super::test_utils::fixtures::*;

    let my_box: Box<usize> = Box::new(100);

    // This is the address of the `Box` itself, ie `&Box<T>`.
    let box_ref = &my_box;
    let box_addr_stack = format_address(box_ref);
    println!(
        "[stack] Box address: {} , 🫱  val: {}",
        box_addr_stack, box_ref
    );

    // Get the address of the value pointed to by the `Box`, on the heap, ie `&T`
    let val_ref = my_box.as_ref();
    let val_addr_heap = format_address(val_ref);
    println!(
        "[heap]  Val address: {} , 🫱  val: {}",
        val_addr_heap, val_ref
    );

    assert_ne!(box_addr_stack, val_addr_heap);
}

#[test]
fn get_address_of_vec_on_heap() {
    use super::test_utils::fixtures::*;

    let val = vec![1, 2, 3, 4, 5];

    // Get the address of the variable on the stack.
    let val_ref = &val;
    let ptr = val_ref as *const Vec<i32>;
    println!("Address of val (stack): {}", format_address(ptr));

    // Approach 1.1 - Get the address of the heap-allocated backing store.
    let ptr_2 = val.as_ptr();
    println!("Address of vec (heap) : {}", format_address(ptr_2));

    // Approach 1.2 - Convert `Vec<T>` to `Box<[T]>` & get the raw pointer to the
    // heap-allocated memory.
    let boxed = val.into_boxed_slice();
    let ptr_3 = boxed.as_ptr();
    println!("Address of vec (heap) : {}", format_address(ptr_3));

    // Make assertions.
    assert_ne!(format_address(ptr), format_address(ptr_2));
    assert_eq!(format_address(ptr_2), format_address(ptr_3));
}
