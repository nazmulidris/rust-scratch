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

#[cfg(test)]
mod alignment_tests {
    use r3bl_tui::{fg_light_yellow_green, fg_lizard_green};
    use std::mem::{align_of, size_of};

    fn pretty_print<T: std::fmt::Debug>() {
        let type_name = std::any::type_name::<T>();
        let size = size_of::<T>();
        let align = align_of::<T>();

        fg_lizard_green(format!("\n{type_name}")).println();
        fg_light_yellow_green(format!("  size = {size}\n  alignment = {align}")).println();
    }

    #[serial_test::serial]
    #[test]
    fn test_1() {
        pretty_print::<u8>();
        pretty_print::<u16>();
        pretty_print::<u32>();
        pretty_print::<u64>();
        pretty_print::<usize>();
        pretty_print::<f64>();
    }

    #[serial_test::serial]
    #[test]
    fn test_2() {
        #[repr(C)]
        struct Demo {
            a: u8,  // 1 byte, alignment 1
            b: u32, // 4 bytes, alignment 4
            c: u16, // 2 bytes, alignment 2
        }

        let size = size_of::<Demo>();
        let align = align_of::<Demo>();

        fg_lizard_green(format!("\nSize of Demo: {size}")).println();
        fg_light_yellow_green(format!("Alignment of Demo: {align}")).println();
    }
}
