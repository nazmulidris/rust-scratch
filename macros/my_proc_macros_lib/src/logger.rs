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

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use core::panic;
use std::{collections::HashSet as Set, path::Path};

use quote::quote;
use r3bl_rs_utils::utils::{print_header, style_primary, style_prompt};
use syn::{parse::{Parse, ParseStream, Result},
          parse_macro_input,
          punctuated::Punctuated,
          AttributeArgs,
          Ident,
          ItemFn,
          MetaNameValue,
          Token};

use crate::{utils,
            utils::{attribute_args_ext::AttributeArgsExt,
                    ident_ext::IdentExt,
                    meta_ext::MetaExt,
                    nested_meta_ext::NestedMeta}};

/// The args take a key value pair like `#[attrib_macro_logger(key = "value")]`.
pub fn attrib_proc_macro_impl_1(
  args: proc_macro::TokenStream,
  item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let args = parse_macro_input!(args as AttributeArgs);
  let item = parse_macro_input!(item as ItemFn);

  // Parse args (which contain key & value).
  let (key, value) = args.get_key_value_pair();
  println!(
    "key: {}, value: {}",
    style_prompt(&key),
    style_prompt(&value),
  );

  let fn_ident = item.sig.ident.from_string(&key);

  quote! {
    fn #fn_ident() -> &'static str {
      #value
    }
  }
  .into()
}

/// The args take a set of identifiers like `#[attrib_macro_logger(a, b, c)]`.
pub fn attrib_proc_macro_impl_2(
  args: proc_macro::TokenStream,
  item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let args = parse_macro_input!(args as ArgsHoldingIdents);
  let item = parse_macro_input!(item as ItemFn);

  let fn_name_ident = item.sig.ident;

  let args_to_string = args
    .idents
    .iter()
    .map(|ident| ident.to_string())
    .collect::<Vec<_>>()
    .join(", ");

  quote! {
    pub fn #fn_name_ident() -> &'static str { #args_to_string }
  }
  .into()
}

/// Parses a list of variable names separated by `+`.
///
///     a + b + c
///
/// This is how the compiler passes in arguments to our attribute -- it is
/// everything inside the delimiters after the attribute name.
///
///     #[attrib_macro_logger(a+ b+ c)]
///                           ^^^^^^^
struct ArgsHoldingIdents {
  idents: Set<Ident>,
}

impl Parse for ArgsHoldingIdents {
  fn parse(args: ParseStream) -> Result<Self> {
    let vars: Punctuated<Ident, Token![+]> = Punctuated::parse_terminated(args)?;
    Ok(ArgsHoldingIdents {
      idents: vars.into_iter().collect(),
    })
  }
}
