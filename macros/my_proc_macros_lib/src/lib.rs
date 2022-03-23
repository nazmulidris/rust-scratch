//! Add lib crate for macros: <https://dev.to/dandyvica/rust-procedural-macros-step-by-step-tutorial-36n8>
//! `lib.rs` restriction: <https://users.rust-lang.org/t/how-to-import-procedural-macros-that-is-not-in-lib-rs/58323/9>

extern crate proc_macro;
use proc_macro::TokenStream;

mod fn_macro_ast_viz_debug;
mod derive_macro_describe;

#[proc_macro]
pub fn fn_macro_ast_viz_debug(input: TokenStream) -> TokenStream {
  fn_macro_ast_viz_debug::macro_impl(input)
}

#[proc_macro_derive(Describe)]
pub fn describe(input: TokenStream) -> TokenStream {
  derive_macro_describe::macro_impl(input)
}
