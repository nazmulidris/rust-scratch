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
