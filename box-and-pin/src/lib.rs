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

//! # Why do we need both Box and Pin?
//!
//! [std::pin::Pin] is used all the time in async code, especially for cancel safety. And
//! [Box] is used everywhere for trait pointers.
//!
//! This repo, article, and video is a deep dive of just these 2 small concepts using the
//! repo below (w/ richly formatted test output) as a guide.
//! - There are 3 examples in the repo that can be used to showcase this topic!
//! - And you have to use formatted output to get clarity on what's going on!
//!
//! # Run the tests to see the output
//!
//! To run these tests, in watch mode, use the following command:
//! ```sh
//! cargo watch -x "test --lib -- --show-output"
//! ```
//!
//! To restrict the tests to just a specific test, eg `move_a_box`, use the following
//! command, in watch mode:
//! ```sh
//! cargo watch -x "test --lib -- --show-output move_a_box"
//! ```
//!
//! To run a test just once, run:
//! ```sh
//! cargo test --lib -- --show-output
//! ```
//!
//! # Formatting pointers
//!
//! To format pointers in Rust, use the formatting trait
//! [`{:p}`](https://doc.rust-lang.org/std/fmt/#formatting-traits). You can format a
//! pointer by using two approaches:
//! 1. Get the address of the pointer using [`std::ptr::addr_of!`] and then format it
//!    using `{:p}`. Eg: `let x = 1; println!("{:p}", std::ptr::addr_of!(x));`
//! 2. Get a reference to the pointer using `&` and then format it using `{:p}`. Eg: `let
//!    x = 1; println!("{:p}", &x);`
//!
//! # What is a smart pointer?
//!
//! Smart pointers in Rust are data structures that act like pointers but also have
//! additional metadata and capabilities. They provide a level of abstraction over raw
//! pointers, offering features like ownership management, reference counting, and
//! more. Smart pointers often manage ownership of the data they point to, ensuring
//! proper deallocation when no longer needed.
//!
//! # Memory allocation, stack and heap
//!
//! - [Great visualization](https://courses.grainger.illinois.edu/cs225/fa2022/resources/stack-heap/)

#[cfg(test)]
mod box_and_pin {
    /// Given a pointer `$p`, it prints:
    /// 1. it's address,
    /// 2. and size of the thing it points to (in bytes).
    macro_rules! print_ptr_addr_size {
        ($p: expr) => {
            format!("{:p}┆{}b", $p, std::mem::size_of_val($p))
        };
    }

    /// Given a pinned pointer `$p`, it prints:
    /// 1. it's address,
    /// 2. and size of the thing it points to (in bytes).
    macro_rules! print_pin_addr_size {
        ($p: expr) => {
            format!("{:p}┆{}b", $p, std::mem::size_of_val(&(*$p)))
        };
    }

    fn assert_three_equal<T: PartialEq + std::fmt::Debug>(a: &T, b: &T, c: &T) {
        assert_eq!(a, b, "a and b are not equal");
        assert_eq!(a, c, "a and c are not equal");
    }

    use crossterm::style::Stylize;
    use serial_test::serial;

    /// More info on working with raw pointers:
    /// <https://gemini.google.com/app/3f99cdc756923c0f>
    #[test]
    #[serial]
    fn print_ptr_addr_size_macro() {
        println!(
            "\n{}\n",
            "fn print_ptr_addr_size_macro()"
                .dark_grey()
                .on_black()
                .underlined()
        );

        let x = 100u8;
        let x_addr_str = print_ptr_addr_size!(&x);
        println!("1. addr[&x]: {}", x_addr_str.clone().cyan().on_black());
        assert_eq!(x_addr_str, format!("{:p}┆1b", &x));

        // Raw pointer: https://gemini.google.com/app/3f99cdc756923c0f
        let x_addr = std::ptr::addr_of!(x);
        let x_size_of = std::mem::size_of_val(&x);
        let x_addr_str_2 = format!("{:p}┆{}b", x_addr, x_size_of);

        println!("2. addr[&x]: {}", x_addr_str_2.clone().cyan().on_black());
        assert_eq!(x_addr_str, x_addr_str_2);
    }

    /// # What is a [Box]?
    ///
    /// [Box] is a smart pointer to heap memory.
    /// - The thing pointed to by the [Box] is allocated on the heap,
    /// - and the pointer is stored on the stack.
    ///
    /// # How Box moves
    ///
    /// When you move a [Box], you're essentially transferring the ownership of the
    /// heap-allocated data to a new variable. The actual data remains in the same memory
    /// location, but the [Box] itself (which is a pointer to that data) is moved.
    ///
    /// 1. The thing on the heap (`u8`) does not move between `b_1` and `b_2`.
    /// 2. The pointer (aka [Box]) on the stack moves and `&b_1` != `&b_2`.
    ///
    /// You can do the opposite of this by swapping the contents of two boxes, as shown in
    /// [swap_box_contents()].
    ///
    /// <https://courses.grainger.illinois.edu/cs225/fa2022/resources/stack-heap/>
    #[test]
    #[serial]
    fn move_a_box() {
        println!(
            "\n{}\n",
            "fn move_a_box()".dark_grey().on_black().underlined()
        );

        // 🦄 Create a boxed value (on the heap).
        let b_1 = Box::new(255u8);
        let b_1_addr = print_ptr_addr_size!(b_1.as_ref());
        let b_1_ref_addr = print_ptr_addr_size!(&b_1);

        println!(
            "1. *b_1 is value on heap: {}, {}",
            format!("{:?}", *b_1).magenta().underlined(),
            "b_1.as_ref() points to heap".black().on_dark_grey()
        );

        /* &*b_1 = b_1 = b_1.as_ref() */
        println!(
            "2. addr[{}]: {} ┇ addr[{}]: {}",
            "b_1.as_ref()".cyan(),
            print_ptr_addr_size!(b_1.as_ref()).green().underlined(),
            "&b_1".cyan(),
            print_ptr_addr_size!(&b_1).green().italic(),
        );

        // 🦄 Move the boxed value (change ownership & drop).
        let b_2 = b_1;
        let b_2_addr = print_ptr_addr_size!(b_2.as_ref());
        let b_2_ref_addr = print_ptr_addr_size!(&b_2);
        // println!("{b_1:p}"); // ⛔ error: use of moved value: `b_1`

        println!(
            "{}, {}",
            "3. b_2 = b_1, ie, move b_1 to b_2".yellow().underlined(),
            "b_1 is dropped".to_string().red().italic()
        );

        println!(
            "4. *b_2 is value on heap: {}, {}",
            format!("{:?}", *b_2).magenta().underlined(),
            "b_2.as_ref() points to heap".black().on_dark_grey()
        );

        /* &*b_2 = b_2 = b_2.as_ref() */
        println!(
            "5. addr[{}]: {} ┇ addr[{}]: {}",
            "b_2.as_ref()".cyan(),
            print_ptr_addr_size!(b_2.as_ref()).blue().underlined(),
            "&b_2".cyan(),
            print_ptr_addr_size!(&b_2).blue().italic(),
        );

        // The heap memory allocation does not change (does not move). Pointee does not move.
        assert_eq!(b_1_addr, b_2_addr);

        // The stack memory allocation does change (does move). Boxes aka pointers have move.
        assert_ne!(b_1_ref_addr, b_2_ref_addr);

        // When b_2 is dropped, the heap allocation is deallocated. This is why Box is a smart pointer.
    }

    /// Swapping the contents of two boxes. This is the "opposite" of moving a box
    /// as shown in [move_a_box()].
    /// - A box is a pointer to heap-allocated data.
    /// - The heap-allocated data is swapped, but the pointers to them are not.
    #[test]
    #[serial]
    fn swap_box_contents() {
        println!(
            "\n{}\n",
            "fn swap_box_contents()".dark_grey().on_black().underlined()
        );

        // 🦄 Create two boxed values (on the heap).
        let mut b_1 = Box::new(100u8);
        let mut b_2 = Box::new(200u8);
        let og_b_1_addr = print_ptr_addr_size!(b_1.as_ref());
        let og_b_2_addr = print_ptr_addr_size!(b_2.as_ref());

        assert_eq!(*b_1, 100);
        assert_eq!(*b_2, 200);
        println!(
            "addr[{}]: {} | addr[{}]: {}",
            "b_1.as_ref()".cyan(),
            format!("{:p}", b_1.as_ref()).magenta().underlined(),
            "b_2.as_ref()".cyan(),
            format!("{:p}", b_2.as_ref()).green(),
        );
        println!(
            "addr[{}]: {} | addr[{}]: {}",
            "&b_1".yellow(),
            format!("{:p}", &b_1).blue(),
            "&b_2".yellow(),
            format!("{:p}", &b_2).blue().on_black(),
        );

        // 🦄 Swap the boxed values. Heap contents change, but not the boxes (pointers).
        println!("{}", "Swapping b_1 and b_2".white().on_dark_yellow());
        std::mem::swap(&mut b_1, &mut b_2);

        let swapped_b_1_addr = print_ptr_addr_size!(b_1.as_ref());
        let swapped_b_2_addr = print_ptr_addr_size!(b_2.as_ref());

        assert_eq!(*b_1, 200);
        assert_eq!(*b_2, 100);
        println!(
            "addr[{}]: {} | addr[{}]: {}",
            "b_1.as_ref()".cyan(),
            format!("{:p}", b_1.as_ref()).green(),
            "b_2.as_ref()".cyan(),
            format!("{:p}", b_2.as_ref()).magenta().underlined(),
        );
        println!(
            "addr[{}]: {} | addr[{}]: {}",
            "&b_1".yellow(),
            format!("{:p}", &b_1).blue(),
            "&b_2".yellow(),
            format!("{:p}", &b_2).blue().on_black(),
        );

        assert_eq!(og_b_1_addr, swapped_b_2_addr);
        assert_eq!(og_b_2_addr, swapped_b_1_addr);
    }

    /// By combining [Box] and [Pin], you can have heap-allocated data that doesn't move.
    ///
    /// 1. [Box] provides heap allocation and ownership. This is necessary when the size
    ///    of the data is unknown at compile time, such as with trait pointers.
    /// 2. [Pin] provides stability by preventing the data it points to from moving in
    ///    memory. This is crucial for certain types of data structures, like
    ///    [`std::future::Future`]s in any kind of `async` code. Especially for cancel
    ///    safety in the `tokio::select!` macro.
    #[test]
    #[serial]
    fn box_and_pin_dynamic_duo() {
        println!(
            "\n{}\n",
            "fn box_and_pin_dynamic_duo()"
                .dark_grey()
                .on_black()
                .underlined()
        );

        // 🦄 Create a boxed value (on the heap).
        let b_1 = Box::new(100u8);
        let b_1_addr = print_ptr_addr_size!(b_1.as_ref());

        println!(
            "1. addr[{}]: {}",
            "b_1.as_ref()".cyan(),
            b_1_addr.clone().black().underlined().on_green(),
        );
        println!(
            "2. addr[{}]: {}",
            "&b_1".cyan(),
            print_ptr_addr_size!(&b_1).magenta().italic(),
        );

        // 🦄 Pin the boxed value.
        // The pinned value has to be dereferenced, to get a reference the box within.
        //
        // There are many ways to do this:
        // - Box::pin(..), or
        // - std::pin::pin!(..)
        // The semantics of this are different. They add another Box wrapper to the boxed
        // value, and then Pin that! You can use in this case:
        // `print_pin_addr_size!((*p_b_1).as_ref())`
        let p_b_1 = std::boxed::Box::into_pin(b_1);
        let p_b_1_addr = print_pin_addr_size!(p_b_1);

        println!(
            "3. {} value on heap: {}",
            "*p_b_1".cyan(),
            format!("{:?}", *p_b_1).dark_grey(),
        );
        println!(
            "4. addr[{}]: {}",
            "p_b_1.as_ref()".cyan(),
            p_b_1_addr.to_string().black().underlined().on_green(),
        );
        println!(
            "5. addr[{}]: {}",
            "&p_b_1".cyan(),
            print_ptr_addr_size!(&p_b_1).green().italic(),
        );

        // 🦄 Try to move the pinned boxed value.
        let b_2 = p_b_1;
        let b_2_addr = print_pin_addr_size!(b_2);
        println!(
            "{}",
            "6. Assign b_2 = p_b_1, moving p_b_1, dropping it"
                .white()
                .on_dark_yellow()
        );
        // println!("{}", p_b_1); // ⛔ error: use of moved value: `p_b_1`

        println!(
            "7. addr[{}]: {}",
            "b_2.as_ref()".cyan(),
            b_2_addr.to_string().black().underlined().on_green(),
        );
        println!(
            "8. Despite the move {} and {} have the same address",
            "b_2".cyan(),
            "p_b_1".cyan()
        );

        // 🦄 Ownership changes, but it does not move!
        assert_three_equal(&b_1_addr, &p_b_1_addr, &b_2_addr);
    }
}
