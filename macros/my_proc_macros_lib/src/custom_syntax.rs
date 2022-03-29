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

use quote::quote;
use syn::{parse::{Parse, ParseStream},
          parse_macro_input,
          Expr,
          Ident,
          Result,
          Token,
          Type,
          Visibility};

/// fn_macro_custom_syntax! {
///   $MANAGER_IDENT for $THING_TYPE
/// }
/// Eg: `fn_macro_custom_syntax! { ThingManager for Vec<T> }`
///
/// For reference, here's an example from syn:
/// - [lazy-static](https://github.com/dtolnay/syn/blob/master/examples/lazy-static/lazy-static/src/lib.rs)
pub fn fn_proc_macro_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  quote! {
    pub fn foo () -> i32 {
      42
    }
  }
  .into()
}

struct ManagerOfThingInfo {
  manager_ident: Ident,
  thing_type: Type,
}

impl Parse for ManagerOfThingInfo {
  fn parse(input: ParseStream) -> Result<Self> {
    let manager_ident: Ident = input.parse()?;
    input.parse::<Token![for]>()?;
    let thing_type: Type = input.parse()?;
    Ok(ManagerOfThingInfo {
      manager_ident,
      thing_type,
    })
  }
}
