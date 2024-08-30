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

/// We can use lifetimes and slices to work with data without modifying it. This pattern
/// shows up a lot when working with parsers (eg: `nom`) and general string manipulation.
///
/// Real world examples:
/// - <https://github.com/r3bl-org/r3bl-open-core/tree/main/core/src/tui_core/graphemes>
/// - <https://github.com/r3bl-org/r3bl-open-core/blob/main/core/src/tui_core/graphemes/access.rs#L173>
#[rustfmt::skip]
#[test]
fn ex_4_input_slices() {
    // 'fn {
        let data = String::from("foo bar baz");
        let middle_word: & /*'fn*/ str = middle_word(&data);
        assert_eq!(middle_word, "bar");
    // }
}

fn middle_word<'input>(input: &'input str) -> &'input str {
    let iter = input.split_whitespace();

    let (_, middle_word_index) = {
        let iter_clone = iter.clone();
        let word_count = iter_clone.count();
        let middle_word_index = word_count / 2;
        (word_count, middle_word_index)
    };

    let (middle_word_len, len_until_middle_word) = {
        let mut middle_word_len = 0;
        let len_until_middle_word = iter
            .enumerate()
            // Go as far as the middle word.
            .take_while(|(index, _)| *index <= middle_word_index)
            .map(|(index, word)| {
                // At middle word.
                if index == middle_word_index {
                    middle_word_len = word.len();
                    0
                }
                // Before middle word.
                else {
                    word.len()
                }
            })
            .sum::<usize>();

        (middle_word_len, len_until_middle_word)
    };

    let (start_index, end_index) = {
        let start_index = len_until_middle_word + 1;
        let end_index = len_until_middle_word + middle_word_len + 1;
        (start_index, end_index)
    };

    &/*'input*/input[start_index..end_index]
}
