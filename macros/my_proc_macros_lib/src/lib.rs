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

//! Add lib crate for macros: <https://dev.to/dandyvica/rust-procedural-macros-step-by-step-tutorial-36n8>
//! `lib.rs` restriction: <https://users.rust-lang.org/t/how-to-import-procedural-macros-that-is-not-in-lib-rs/58323/9>

extern crate proc_macro;
use proc_macro::TokenStream;

mod ast_viz_debug;
mod describe;
mod builder;
mod utils;
mod logger;
mod custom_syntax;

#[proc_macro]
pub fn fn_macro_ast_viz_debug(input: TokenStream) -> TokenStream {
  ast_viz_debug::fn_proc_macro_impl(input)
}

#[proc_macro]
pub fn fn_macro_custom_syntax(input: TokenStream) -> TokenStream {
  custom_syntax::fn_proc_macro_impl(input)
}

#[proc_macro_derive(Describe)]
pub fn derive_macro_describe(input: TokenStream) -> TokenStream {
  describe::derive_proc_macro_impl(input)
}

#[proc_macro_derive(Builder)]
pub fn derive_macro_builder(input: TokenStream) -> TokenStream {
  builder::derive_proc_macro_impl(input)
}

#[proc_macro_attribute]
pub fn attrib_macro_logger_1(
  args: TokenStream,
  input: TokenStream,
) -> TokenStream {
  logger::attrib_proc_macro_impl_1(args, input)
}

#[proc_macro_attribute]
pub fn attrib_macro_logger_2(
  args: TokenStream,
  input: TokenStream,
) -> TokenStream {
  logger::attrib_proc_macro_impl_2(args, input)
}
