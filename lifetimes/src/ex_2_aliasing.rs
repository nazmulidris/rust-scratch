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

/// # There are two kinds of reference
///
/// 1. Shared reference: `&`
/// 2. Mutable reference: `&mut`
///
/// # Here are the rules of references
///
/// 1. A reference cannot outlive its referent.
/// 2. A **mutable reference** cannot be aliased.
///
/// # Aliasing
///
/// 1. Variables and pointers alias if they refer to overlapping regions of memory.
///
/// The definition of "alias" that Rust will use likely involves some notion of
/// **liveness** and **mutation**: we don't actually care if aliasing occurs if there
/// aren't any actual writes to memory happening.
///
/// # Run the test
///
/// ```bash
/// cargo test -- -q --show-output ex_2
/// ```
///
/// # More info
///
/// - <https://doc.rust-lang.org/nomicon/references.html>
/// - <https://doc.rust-lang.org/nomicon/aliasing.html>
#[test]
fn ex_2_aliasing() {
    /// `input_ref` and `output_ref` can't overlap or alias, and thus can't clobber each
    /// other.
    fn compute(input_ref: &usize, output_ref: &mut usize) {
        if *input_ref > 10 {
            *output_ref = 1;
        }
        if *input_ref > 5 {
            *output_ref *= 2;
        }
    }

    // This is safe to do because `input` and `output` don't overlap.
    {
        let input = 10usize;
        let mut output = 1usize;

        let input_address = &input as *const usize;
        let output_address = &output as *const usize;

        compute(&input, &mut output);

        assert_eq!(output, 2);
        assert_ne!(input_address, output_address);
    }

    // Try and clobber `input` with `output`.
    // - Rust won't allow `input` and `output` to overlap aka alias.
    // - Rust won't allow the `&mut output` to be aliased!
    {
        let mut output = 1usize;
        // compute(&output, &mut output); /* 🧨 won't compile! */
    }
}
