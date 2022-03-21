//! Add lib crate for macros: <https://dev.to/dandyvica/rust-procedural-macros-step-by-step-tutorial-36n8>
//! `lib.rs` restriction: <https://users.rust-lang.org/t/how-to-import-procedural-macros-that-is-not-in-lib-rs/58323/9>

extern crate proc_macro;
use proc_macro::TokenStream;

mod misc;
use misc::make_answer_macro;

#[proc_macro]
pub fn make_answer(_: TokenStream) -> TokenStream {
  make_answer_macro()
}
