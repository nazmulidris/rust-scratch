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

trait Iter {
    type Item<'a>
    where
        Self: 'a;

    fn g_get<'a>(&'a self) -> Option<Self::Item<'a>>;
}

pub struct MyContainer {
    value: String,
}

impl Iter for MyContainer {
    type Item<'a>
        = &'a str
    where
        Self: 'a;

    fn g_get<'a>(&'a self) -> Option<Self::Item<'a>> {
        Some(&self.value.as_ref())
    }
}

#[test]
fn test_g_get() {
    let instance = MyContainer {
        value: String::from("abcd"),
    };
    println!("{:?}", instance.g_get());
    assert_eq!(instance.g_get(), Some("abcd"));
}
