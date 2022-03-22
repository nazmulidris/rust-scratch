//! Add lib crate for macros: <https://dev.to/dandyvica/rust-procedural-macros-step-by-step-tutorial-36n8>
//! `lib.rs` restriction: <https://users.rust-lang.org/t/how-to-import-procedural-macros-that-is-not-in-lib-rs/58323/9>

extern crate proc_macro;
use proc_macro::TokenStream;

mod debug_token_stream_fn_like_macro;
use debug_token_stream_fn_like_macro::simple_function_macro_make_a_fn_impl;

// TODO: add derive builder macro
// mod derive_builder;
// use derive_builder::Builder;

#[proc_macro]
pub fn simple_function_macro_make_a_fn(input: TokenStream) -> TokenStream {
  simple_function_macro_make_a_fn_impl(input)
}
