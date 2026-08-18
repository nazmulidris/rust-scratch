//! # Elegant Constructor DSL Pattern in Rust
//!
//! This crate provides a clean, pedagogical reference implementation of the **Elegant
//! Constructor DSL Pattern** used throughout the `r3bl-open-core` codebase.
//!
//! ## Overview
//!
//! When designing constructors for data structures with multiple optional configurations,
//! Rust developers often face trade-offs:
//! - **`Option` arguments**: Verbose and noisy at call sites (`None, None, Some(...)`).
//! - **Positional arguments**: Hard to read and error-prone when types match (`(&str, &str)`).
//! - **Builder pattern**: Adds significant boilerplate and ceremony for small configurations.
//!
//! The **Elegant Constructor DSL Pattern** solves this by combining:
//! 1. **Lightweight newtype or unit struct tokens** for compile-time disambiguation.
//! 2. **`From` / `Into` conversions** allowing constructors to accept unit `()`, single tokens, or composed configs.
//! 3. **`Add` (`+`) and `AddAssign` (`+=`) operator overloading** allowing callers to compose options in any order.
//!
//! ## Submodules
//!
//! - [`two_tokens`]: Demonstrates the minimal 2-token scenario (like `EditorBufferConfig`),
//!   showing why `Config + Token` is not needed when only 2 tokens exist.
//! - [`three_tokens`]: Demonstrates the 3+ token scenario (like `TuiStyleAttribs`),
//!   showing why `Config + Token` and `Token + Config` are essential for left-associative chaining.

pub mod three_tokens;
pub mod two_tokens;

pub use three_tokens::*;
pub use two_tokens::*;
