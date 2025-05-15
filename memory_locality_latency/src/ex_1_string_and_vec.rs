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

//! 1. Make sure to add the following to your `lib.rs` file:
//! ```rust
//! #![feature(vec_into_raw_parts)] // Needed for into_raw_parts() method on String and Vec.
//! ```
//!
//! 2. Make sure to add the following `rust-toolchain.toml` file to your project root directory:
//! ```toml
//! [toolchain]
//! channel = "nightly"
//! ```

#[cfg(test)]
mod string_and_vec_tests {
    use r3bl_tui::{fg_light_yellow_green, fg_lizard_green};

    #[serial_test::serial]
    #[test]
    /// Demonstrates the memory layout of String, which contains [ptr, len, capacity].
    fn mem_layout_string() {
        fg_lizard_green("\n=== String Memory Layout Example ===").println();

        // Create a String.
        // ASCII values for digits:
        // '0': 48 (0x30), '1': 49 (0x31), '2': 50 (0x32), '3': 51 (0x33), '4': 52 (0x34)
        // '5': 53 (0x35), '6': 54 (0x36), '7': 55 (0x37), '8': 56 (0x38), '9': 57 (0x39)
        let s = String::from("0123456789");

        // We can get these values safely.
        fg_light_yellow_green("\nSafely accessing String metadata:").println();
        println!("  ptr: {:p}", s.as_ptr());
        println!("  len: {}", s.len());
        println!("  cap: {}", s.capacity());

        // Unsafely transmute String to Vec of bytes.
        // This will show the Vec representation which includes the UTF-8 bytes
        // (identical to ASCII values for these digits).
        fg_light_yellow_green("\nUnsafely accessing String as Vec<u8> (hex dump):").println();
        println!("{:x?}", unsafe {
            std::mem::transmute::<String, Vec<u8>>(s)
        });

        // Note that transmuting a String to the following does not work:
        // let (ptr, len, cap): (*mut usize, usize, usize) = unsafe { std::mem::transmute(s) };
        // - `(*const u8, usize, usize)`
        // - `(*mut u8, usize, usize)`
        {
            fg_light_yellow_green("\nAccessing String with into_raw_parts():").println();
            let s = String::from("0123456789");
            let (ptr, len, cap) = s.into_raw_parts();
            println!("  ptr: {:p}", ptr);
            println!("  len: {}", len);
            println!("  cap: {}", cap);
        }
    }

    #[serial_test::serial]
    #[test]
    /// Demonstrates the memory layout of &str, which contains [ptr, len].
    fn mem_layout_str_slice() {
        fg_lizard_green("\n=== &str Memory Layout Example 1 ===").println();

        // Create a string slice
        let s = "Hello, world!";

        // &str is represented as [ptr, len].
        unsafe {
            // Transmute &str to raw parts.
            let raw_parts: (*const u8, usize) = std::mem::transmute(s);

            fg_light_yellow_green("\n&str memory layout:").println();
            println!("  ptr: {:p}", raw_parts.0);
            println!("  len: {}", raw_parts.1);

            // We can also get these values safely
            fg_light_yellow_green("\nSafely accessing &str metadata:").println();
            println!("  ptr: {:p}", s.as_ptr());
            println!("  len: {}", s.len());
        }
    }

    #[serial_test::serial]
    #[test]
    fn mem_layout_str_slice_2() {
        fg_lizard_green("\n=== &str Memory Layout Example 2 ===").println();

        // Demonstrate that &str is just a view into some data.
        let owned = String::from("Hello, world!");
        let slice = &owned[0..5]; // "Hello".

        // Safe approach to get the pointer and length for slice.
        let slice_ptr = slice.as_ptr();
        let slice_len = slice.len();

        // Safe approach to get the pointer and length for owned.
        let owned_ptr = owned.as_ptr();
        let owned_len = owned.len();
        let owned_capacity = owned.capacity();

        fg_light_yellow_green("\nComparing owned String and &str slice (safely):").println();
        println!("  String ptr: {:p}", owned_ptr);
        println!("  &str ptr:   {:p}", slice_ptr);
        println!(
            "  String points to same memory as slice: {}",
            slice_ptr == owned_ptr
        );
        println!("  String len: {}, slice len: {}", owned_len, slice_len);
        println!("  String cap: {}", owned_capacity);
    }
}
