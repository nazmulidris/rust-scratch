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

/// Rust requires any _references_ to freeze:
/// - the referent and its owners.
///
/// While a _reference_ is **in scope**, Rust will not allow you to:
/// - change the referent and its owners.
///
/// [More info](https://doc.rust-lang.org/nomicon/ownership.html)
#[test]
fn ex_1_references() {
    #[allow(dead_code)]
    fn try_to_use_after_free(arg: usize) -> &'static str {
        let s = format!("{} is a number", arg);
        // return &s; /* 🧨 won't compile! */
        unreachable!()
    }

    fn try_to_modify_referent() {
        let mut data = vec![1, 2, 3]; /* referent */
        let ref_to_first_item = &data[0]; /* reference */
        // data.push(4); /* 🧨 won't compile */
        println!("first_item: {}", ref_to_first_item); /* reference still in scope */
        // drop(ref_to_first_item);
    }
}
