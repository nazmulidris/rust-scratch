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

//! The `Cow` type is a smart pointer that can be used to work with both owned and
//! borrowed data. It is useful when you want to avoid unnecessary allocations and
//! copying. You can also use it in functions where you might need to mutate the argument;
//! in which case the data will be lazily cloned when mutation or ownership is required.

#[test]
fn ex_6_cow() {
    use std::borrow::Cow;

    fn capitalize<'a>(input: Cow<'a, str>) -> Cow<'a, str> {
        if input.is_empty() {
            return input;
        }

        if input.chars().all(char::is_uppercase) {
            return input;
        }

        let mut cloned = String::with_capacity(input.len());
        cloned.push_str(&input[..1].to_uppercase());
        cloned.push_str(&input[1..]);
        Cow::Owned(cloned)
    }

    let borrowed_data = Cow::Borrowed("hello");
    let owned_data = Cow::Owned(String::from("world"));

    let capitalized_borrowed_data = capitalize(borrowed_data);
    let capitalized_owned_data = capitalize(owned_data);

    assert_eq!(capitalized_borrowed_data, "Hello");
    assert_eq!(capitalized_owned_data, "World");
}

#[test]
fn ex_6_cow_2() {
    use std::borrow::Cow;

    fn capitalize_mut<'a>(input: &mut Cow<'a, str>) {
        if input.is_empty() {
            return;
        }

        if input.chars().all(char::is_uppercase) {
            return;
        }

        let mut cloned = String::with_capacity(input.len());
        cloned.push_str(&input[..1].to_uppercase());
        cloned.push_str(&input[1..]);
        *input = Cow::Owned(cloned);
    }

    let mut borrowed_data = Cow::Borrowed("hello");
    let mut owned_data = Cow::Owned(String::from("world"));

    capitalize_mut(&mut borrowed_data);
    capitalize_mut(&mut owned_data);

    assert_eq!(borrowed_data, "Hello");
    assert_eq!(owned_data, "World");
}
