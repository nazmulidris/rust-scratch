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
#![allow(unused_macros)]

use core::panic;

use quote::{quote, ToTokens};
use syn::{parse::{Parse, ParseBuffer, ParseStream},
          parse2,
          parse_macro_input,
          punctuated::Punctuated,
          token::Comma,
          Expr,
          GenericArgument,
          GenericParam,
          Generics,
          Ident,
          Result,
          Token,
          Type,
          Visibility,
          WhereClause};

use crate::utils::type_ext::TypeExt;

// TODO: 🎗️ move this to r3bl_rs_utils crate
macro_rules! debug {
  ($i:ident) => {
    println!(
      "{} {} = {}",
      r3bl_rs_utils::utils::style_error("▶"),
      r3bl_rs_utils::utils::style_prompt(stringify!($i)),
      r3bl_rs_utils::utils::style_dimmed(&format!("{:#?}", $i))
    );
  };
}

/// See [`ManagerOfThingInfo`] for more information on the syntax that this macro accepts.
///
/// For reference, here's an example from syn called
/// [`lazy-static`](https://github.com/dtolnay/syn/blob/master/examples/lazy-static/lazy-static/src/lib.rs)
pub fn fn_proc_macro_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  // dbg!(&input);

  let manager_of_thing_info = parse_macro_input!(input as ManagerOfThingInfo);
  // dbg!(&manager_of_thing_info);

  let ManagerOfThingInfo {
    manager_name_ident,
    manager_ty,
    thing_ty,
    manager_ty_generic_args,
    where_clause,
  } = manager_of_thing_info;

  // dbg!(&manager_name_ident);
  // dbg!(&manager_ty);
  // dbg!(&thing_ty);
  // dbg!(&manager_ty_generic_args);
  // dbg!(&where_clause);

  let doc_struct_str = format!(
    " Generated manager {}.",
    &manager_name_ident,
  );

  quote! {
    #[doc = #doc_struct_str]
    struct #manager_ty #where_clause {
      wrapped_thing: #thing_ty
    }
  }
  .into()
}

/// Example of syntax to parse:
/// ```no_run
/// fn_macro_custom_syntax! {
///   ThingManager<K, V>
///   where K: Send + Sync + 'static, V: Send + Sync + 'static
///   for std::collections::HashMap<K, V>
/// }
/// ```
#[derive(Debug)]
struct ManagerOfThingInfo {
  manager_name_ident: Ident,
  manager_ty: Type,
  manager_ty_generic_args: Option<Punctuated<GenericArgument, Comma>>,
  where_clause: Option<WhereClause>,
  thing_ty: Type,
}

/// [Parse docs](https://docs.rs/syn/latest/syn/parse/index.html)
impl Parse for ManagerOfThingInfo {
  fn parse(input: ParseStream) -> Result<Self> {
    // 👀 Manager Type, eg: `ThingManager<K,V>`.
    let manager_ty: Type = input.parse()?;
    let manager_ty_generic_args = match manager_ty.has_angle_bracketed_generic_args() {
      true => Some(
        manager_ty
          .get_angle_bracketed_generic_args_result()
          .unwrap(),
      ),
      false => None,
    };
    // debug!(manager_ty_has_generic_args);
    // dbg!(&manager_ty);

    // 👀 Optional where clause, eg: `where K: Send+Sync+'static, V: Send+Sync+'static`.
    let mut where_clause: Option<WhereClause> = None;
    if input.peek(Token![where]) {
      where_clause = Some(input.parse::<WhereClause>()?);
      // dbg!(&where_clause);
    } else {
      if manager_ty.has_angle_bracketed_generic_args() {
        let ident_vec = manager_ty
          .get_angle_bracketed_generic_args_idents_result()
          .unwrap();
        let my_ts = quote! {
          where #(#ident_vec: Send + Sync + 'static),*
        }
        .into();
        let my_where_clause: WhereClause = syn::parse(my_ts).unwrap();
        where_clause = Some(my_where_clause)
        // dbg!(&where_clause);
      }
    }

    // 👀 for keyword.
    input.parse::<Token![for]>()?;

    // 👀 Thing Type, eg: `std::collections::HashMap<K, V>`.
    let thing_ty: Type = input.parse()?;

    let manager_name_ident = if manager_ty.has_ident() {
      manager_ty.get_ident().unwrap()
    } else {
      panic!("Expected Type::Path::TypePath.segments to have an Ident")
    };

    Ok(ManagerOfThingInfo {
      manager_ty_generic_args,
      manager_name_ident,
      manager_ty,
      thing_ty,
      where_clause,
    })
  }
}
