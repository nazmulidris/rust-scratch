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

//! This module demonstrates the use of `smallstr` crate. And easier to
//! use version of them: `InlineString`.
//!
//! Run the following command to add the dependencies:
//! ```shell
//! cargo add smallstr r3bl_tui
//! ```
//!
//! Show how to use smallstr -> InlineString

#[cfg(test)]
mod inline_string_ex_tests {
    use r3bl_tui::{InlineString, fg_lizard_green, fg_soft_pink, inline_string};
    use smallstr::SmallString;

    #[serial_test::serial]
    #[test]
    fn test_new_inline_string() {
        // Constructor.
        {
            let mut acc = InlineString::new();
            use std::fmt::Write as _;
            _ = write!(acc, "Hello, world!").unwrap();
            assert_eq!(acc, "Hello, world!");
        }

        // Macro.
        {
            let mut acc = inline_string!("Hello,");
            use std::fmt::Write as _;
            _ = write!(acc, " world!").unwrap();
            assert_eq!(acc, "Hello, world!");
        }
    }

    /// Demonstrates the use of `inline_string!` macro to create an
    /// `InlineString` and then format it using the `Display` trait.
    /// Without allocating a new [String] (on the heap).
    #[serial_test::serial]
    #[test]
    fn test_new_inline_string_display_impl() {
        struct DemoStruct {
            id: u8,
            name: InlineString,
        }

        impl std::fmt::Display for DemoStruct {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "id: {}, name: {}", self.id, self.name)
            }
        }

        let demo = DemoStruct {
            id: 1,
            name: inline_string!("Hello, world!"),
        };
        let to_inline_string = inline_string!("{}", demo);
        assert_eq!(to_inline_string, "id: 1, name: Hello, world!");
        fg_lizard_green(to_inline_string).println();
    }

    #[serial_test::serial]
    #[test]
    fn test_new_smallstr() {
        let mut acc: SmallString<[u8; 8]> = SmallString::new();
        assert_eq!(acc.capacity(), 8);
        assert_eq!(acc.len(), 0);
        fg_lizard_green(format!("is spilled: {}", acc.spilled())).println();

        use std::fmt::Write as _;
        _ = write!(acc, "Hello, world!").unwrap();
        assert_eq!(acc, "Hello, world!");

        assert_eq!(acc.len(), 13);
        assert_eq!(acc.spilled(), true);
        fg_soft_pink(format!("is spilled: {}", acc.spilled())).println();
    }
}
