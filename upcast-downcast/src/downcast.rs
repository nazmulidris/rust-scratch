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

use std::any::Any;

trait MyAny: Any {}

impl dyn MyAny {
    fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }
}

impl MyAny for i32 {}
impl MyAny for String {}

#[test]
fn test_downcast() {
    let x: Box<dyn MyAny> = Box::new(10_i32);
    let y: Box<dyn MyAny> = Box::new("hello".to_string());

    if let Some(i) = x.downcast_ref::<i32>() {
        println!("x is an i32: {}", i);
    }

    if let Some(s) = y.downcast_ref::<String>() {
        println!("y is a String: {}", s);
    }

    if x.downcast_ref::<String>().is_none() {
        println!("x is not a String");
    }
}
