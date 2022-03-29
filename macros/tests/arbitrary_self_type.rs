/*
 *   Copyright (c) 2022 Nazmul Idris
 *   All rights reserved.

 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at

 *   http://www.apache.org/licenses/LICENSE-2.0

 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

#![feature(arbitrary_self_types)]

//! Rust now supports taking more than just `self` and `&self` as the first argument to a
//! method. You can also use `P = &'lt S | &'lt mut S | Box<S> | Rc<S> | Arc<S> | Pin<P>`.
//! Here are the [official
//! docs](https://doc.rust-lang.org/reference/items/associated-items.html#methods) on
//! this. Note that `Mutex` and `RwLock` are not supported.
//!
//! To use this feature you have to do the following for each crate:
//!
//! 1. Enable nightly features by adding a `rust-toolchain.toml` file beside your
//!    `Cargo.toml` file. And add these lines to it:
//!    ```toml
//!    [toolchain]
//!    channel = "nightly"
//!    ```
//! 2. Add `#![feature(arbitrary_self_types)]` to your crate's `Cargo.toml` file.
//!
//! # Arbitrary Self type (WIP)
//! https://github.com/rust-lang/rust/issues/44874
//! https://stackoverflow.com/questions/25462935/what-types-are-valid-for-the-self-parameter-of-a-method
//! https://stackoverflow.com/questions/27454761/what-is-a-crate-attribute-and-where-do-i-add-it
//!
//! # Using different toolchain in project (nightly)
//! https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file
//! https://stackoverflow.com/questions/66681150/how-to-tell-cargo-to-use-nightly

use std::sync::Arc;

#[test]
fn test_arbitrary_self_type() {
  struct Example {
    x: i32,
    y: i32,
  }

  impl Example {
    fn new(
      x: i32,
      y: i32,
    ) -> Self {
      Self { x, y, }
    }

    fn sum(self: &Self,) -> i32 { self.x + self.y }
    fn by_arc_sum(self: Arc<Self,>,) -> i32 { self.x + self.y }
  }

  // Arc<Self>
  {
    let example = Example::new(1, 2,);
    assert_eq!(example.sum(), 3,);

    let example_arc = Arc::new(example,);
    assert_eq!(example_arc.by_arc_sum(), 3);
  }
}
