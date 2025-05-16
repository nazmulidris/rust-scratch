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

//! This module demonstrates the use of `smallvec` crate. And easier to
//! use version of them: `InlineVec`.
//!
//! Run the following command to add the dependencies:
//! ```shell
//! cargo add smallvec r3bl_tui
//! ```
//!
//! - Show how to use smallvec -> InlineVec
//! - Show how to use smallstr -> InlineString
//! - Use the join_ macros from r3bl_tui

#[cfg(test)]
mod inline_vec_ex_tests {
    use r3bl_tui::{Index, InlineVec, Length, fg_lizard_green, inline_vec, len};

    #[serial_test::serial]
    #[test]
    fn test_new_inline_vec() {
        // Using with default capacity. Use `[]` accessor.
        {
            let mut inline_vec = InlineVec::new();
            let length: Length = len(5); // 5
            let max_index: Index = length.convert_to_index(); // 4
            for i in 0..=max_index.as_usize() {
                inline_vec.push(i); // 0, 1, 2, 3, 4
            }
            assert_eq!(inline_vec[Index::from(0).as_usize()], 0);
            assert_eq!(inline_vec[max_index.as_usize()], 4);
            // assert_eq!(inline_vec[max_index.as_usize() + 1], 0); // OOB error!
            assert_eq!(inline_vec.get(max_index.as_usize() + 1), None);
            fg_lizard_green(format!("InlineVec: {:?}", inline_vec)).println();
            assert_eq!(inline_vec.capacity(), 8);
            assert_eq!(inline_vec.len(), 5);
        }

        // Using with macro. Use `get()` accessor.
        {
            let length: Length = len(5); // 5
            let max_index: Index = length.convert_to_index(); // 4
            let inline_vec = inline_vec!(0, 1, 2, 3, 4);
            assert_eq!(inline_vec.get(Index::from(0).as_usize()), Some(&0));
            assert_eq!(inline_vec.get(max_index.as_usize()), Some(&4));
            assert_eq!(inline_vec.get(max_index.as_usize() + 1), None);
            fg_lizard_green(format!("InlineVec: {:?}", inline_vec)).println();
            assert_eq!(inline_vec.capacity(), 8);
            assert_eq!(inline_vec.len(), 5);
        }

        // Using with capacity (even though it is pre-allocated). Use `get()` accessor.
        {
            let mut inline_vec = InlineVec::with_capacity(5);
            let length: Length = len(5); // 5
            let max_index: Index = length.convert_to_index(); // 4
            for i in 0..=max_index.as_usize() {
                inline_vec.push(i); // 0, 1, 2, 3, 4
            }
            assert_eq!(inline_vec.get(Index::from(0).as_usize()), Some(&0));
            assert_eq!(inline_vec.get(max_index.as_usize()), Some(&4));
            assert_eq!(inline_vec.get(max_index.as_usize() + 1), None);
            fg_lizard_green(format!("InlineVec: {:?}", inline_vec)).println();
            assert_eq!(inline_vec.capacity(), 8);
            assert_eq!(inline_vec.len(), 5);
        }
    }

    #[serial_test::serial]
    #[test]
    fn test_mut_inline_vec() {
        let mut inline_vec = InlineVec::new();

        let length: Length = len(5); // 5
        let max_index: Index = length.convert_to_index(); // 4
        for i in 0..=max_index.as_usize() {
            inline_vec.push(i); // 0, 1, 2, 3, 4
        }

        inline_vec[max_index.as_usize()] = 100;

        assert_eq!(inline_vec[0], 0);
        assert_eq!(inline_vec[max_index.as_usize()], 100);

        fg_lizard_green(format!("InlineVec: {:?}", inline_vec)).println();

        // Remove the first element, and shift the rest.
        inline_vec.remove(0);
        assert_eq!(inline_vec.len(), 4);
        assert_eq!(inline_vec.capacity(), 8);
        assert_eq!(inline_vec[0], 1);
        assert_eq!(inline_vec[3], 100);
        fg_lizard_green(format!("InlineVec: {:?}", inline_vec)).println();
    }

    #[serial_test::serial]
    #[test]
    #[should_panic]
    fn test_inline_vec_oob() {
        let mut inline_vec = InlineVec::new();

        assert_eq!(inline_vec.capacity(), 8);
        assert_eq!(inline_vec.len(), 0);

        let length: Length = len(5); // 5
        let max_index: Index = length.convert_to_index(); // 4
        for i in 0..=max_index.as_usize() {
            inline_vec.push(i); // 0, 1, 2, 3, 4
        }

        assert_eq!(inline_vec.capacity(), 8);
        assert_eq!(inline_vec.len(), 5);

        // This should panic because we are trying to access an index that is out of
        // bounds.
        inline_vec[max_index.as_usize() + 1] = 100;
    }
}

#[cfg(test)]
mod smallvec_ex_tests {
    use smallvec::{SmallVec, smallvec};

    // Type alias to reduce typing.
    type MySmallVec = SmallVec<[u8; 4]>;

    #[serial_test::serial]
    #[test]
    fn test_new_smallvec() {
        // With new.
        {
            let mut acc = MySmallVec::new();
            for i in 0..=2 {
                acc.push(i); // 0, 1, 2
            }
            assert_eq!(acc.len(), 3);
            assert_eq!(acc.capacity(), 4);
            assert_eq!(acc.get(0), Some(&0));
            assert_eq!(acc.get(1), Some(&1));
            assert_eq!(acc.get(2), Some(&2));
            assert_eq!(acc.get(3), None);
        }

        // With macro.
        {
            let acc: MySmallVec = smallvec![0, 1, 2];
            assert_eq!(acc.len(), 3);
            assert_eq!(acc.capacity(), 4);
            assert_eq!(acc[0], 0);
            assert_eq!(acc[1], 1);
            assert_eq!(acc[2], 2);
        }
    }

    #[serial_test::serial]
    #[test]
    fn test_mut_smallvec() {
        let mut acc = MySmallVec::new();
        for i in 0..=2 {
            acc.push(i); // 0, 1, 2
        }
        assert_eq!(acc.len(), 3);
        assert_eq!(acc.capacity(), 4);

        acc[2] = 100;

        assert_eq!(acc[0], 0);
        assert_eq!(acc[1], 1);
        assert_eq!(acc[2], 100);

        // Remove the first element, and shift the rest.
        acc.remove(0);
        assert_eq!(acc.len(), 2);
        assert_eq!(acc.capacity(), 4);
        assert_eq!(acc[0], 1);
        assert_eq!(acc[1], 100);
    }
}
