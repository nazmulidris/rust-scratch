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
  let args = parse_macro_input!(args as ArgsHoldingVariableNames);
  let item = parse_macro_input!(item as ItemFn);
  quote! {}.into()
}

/// Parses a list of variable names separated by commas.
///
///     a, b, c
///
/// This is how the compiler passes in arguments to our attribute -- it is
/// everything inside the delimiters after the attribute name.
///
///     #[attrib_macro_logger(a, b, c)]
///                           ^^^^^^^
struct ArgsHoldingVariableNames {
  vars: Set<Ident>,
}

impl Parse for ArgsHoldingVariableNames {
  fn parse(args: ParseStream) -> Result<Self> {
    let vars = Punctuated::<Ident, Token![,]>::parse_terminated(args)?;
    Ok(ArgsHoldingVariableNames {
      vars: vars.into_iter().collect(),
    })
  }
}
