/*
 *   Copyright (c) 2023 Nazmul Idris
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

//! In the TUI engine, the editor component uses a `Vec<String>` to store the lines of text. Each
//! line is represented as a single unicode string.

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    #[test]
    fn test_convert_vec_string_1() {
        let input_vec_string: Vec<String> = vec![
            "line_1".to_string(),
            "line_2".to_string(),
            "line_3".to_string(),
        ];

        let _input_vec_str: Vec<&str> = input_vec_string
            .iter()
            .map(|ref_string| ref_string.as_str())
            .collect();

        let input_vec_cow_string: Vec<Cow<String>> = input_vec_string
            .iter()
            .map(|ref_string| Cow::Owned(ref_string.to_owned() + "\n"))
            .collect();

        let input_vec_str: Vec<&str> = input_vec_cow_string
            .iter()
            .map(|s| s.as_ref().as_str())
            .collect();

        let input_vec_slice: &[&str] = input_vec_str.as_slice();

        let input_str = input_vec_slice.concat();

        println!("input_str: {input_str:?}");
    }

    /// https://www.cloudhadoop.com/rust-join-string-vector/
    #[test]
    fn test_convert_vec_string_2() {
        let input_vec_string: Vec<&str> = vec!["line_1", "line_2", "line_3"];
        let input_string: String = input_vec_string.join("\n");
        let input_str: &str = input_string.as_str();
        println!("input_str (debug):{input_str:?}");
        println!("input_str (display):\n{input_str}");
    }

    /// https://www.cloudhadoop.com/rust-join-string-vector/
    #[test]
    fn test_convert_vec_string_3() {
        let input_vec_string: Vec<String> = vec![
            "line_1".to_owned(),
            "line_2".to_owned(),
            "line_3".to_owned(),
        ];
        let input_string: String = input_vec_string.join("\n");
        let input_str: &str = input_string.as_str();
        println!("input_str (debug):{input_str:?}");
        println!("input_str (display):\n{input_str}");
    }
}
