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

use proc_macro::TokenStream;
use quote::ToTokens;
use r3bl_rs_utils::{style_primary, style_prompt};
use syn::{parse_str, ItemFn};

/// https://docs.rs/syn/latest/syn/macro.parse_macro_input.html
pub fn fn_proc_macro_impl(_input: TokenStream) -> TokenStream {
  let output_token_stream_str = "fn foo() -> u32 { 42 }";
  let output = output_token_stream_str
    .parse()
    .unwrap();

  let ast_item_fn: ItemFn = parse_str::<ItemFn>(output_token_stream_str).unwrap();

  // viz_token_stream("input", &input);

  // viz_token_stream(
  //   &format!("{} {}", "output of ", output_token_stream_str),
  //   &output,
  // );

  viz_ast(ast_item_fn);

  output
}

/// https://docs.rs/syn/latest/syn/fn.parse_str.html
/// https://docs.rs/syn/latest/syn/struct.ItemFn.html
/// https://docs.rs/syn/latest/syn/struct.Attribute.html
/// https://docs.rs/syn/latest/syn/enum.Visibility.html
/// https://docs.rs/syn/latest/syn/struct.Signature.html
/// https://docs.rs/syn/latest/syn/struct.Block.html
/// https://docs.rs/syn/latest/syn/enum.Stmt.html
/// https://github.com/dtolnay/proc-macro-workshop#debugging-tips
fn viz_ast(ast: ItemFn) {
  // Simply dump the AST to the console.
  let ast_clone = ast.clone();
  eprintln!(
    "{} => {:#?}",
    style_primary("Debug::ast"),
    ast_clone
  );

  // Parse AST to dump some items to the console.
  let ItemFn {
    attrs,
    vis,
    sig,
    block,
  } = ast;

  eprintln!(
    "{} ast_item_fn {{ attrs.len:{}, vis:{}, sig:'{}' stmt: '{}' }}",
    style_primary("=>"),
    style_prompt(&attrs.len().to_string()),
    style_prompt(match vis {
      syn::Visibility::Public(_) => "public",
      syn::Visibility::Crate(_) => "crate",
      syn::Visibility::Restricted(_) => "restricted",
      syn::Visibility::Inherited => "inherited",
    }),
    style_prompt(&sig.ident.to_string()),
    style_prompt(&match block.stmts.first() {
      Some(stmt) => {
        let expr_str = stmt
          .to_token_stream()
          .to_string()
          .clone();
        expr_str
      }
      None => "empty".to_string(),
    }),
  );
}

// fn viz_token_stream(
//   msg: &str,
//   token_stream: &TokenStream,
// ) {
//   eprint_header(msg);
//   eprintln!("{:#?}", token_stream);
// }
