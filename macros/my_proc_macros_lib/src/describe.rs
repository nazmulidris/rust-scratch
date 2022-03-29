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

use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input,
          Data::{Enum, Struct, Union},
          DataEnum,
          DataStruct,
          DataUnion,
          DeriveInput,
          Fields::{Named, Unit, Unnamed},
          FieldsNamed,
          FieldsUnnamed};

pub fn derive_proc_macro_impl(input: TokenStream) -> TokenStream {
  let DeriveInput {
    ident: struct_name_ident,
    data,
    generics,
    ..
  } = parse_macro_input!(input as DeriveInput); // Same as: syn::parse(input).unwrap();

  let where_clause = &generics.where_clause;

  let description_str = match data {
    Struct(my_struct) => gen_description_str_for_struct(my_struct),
    Enum(my_enum) => gen_description_str_for_enum(my_enum),
    Union(my_union) => gen_description_str_for_union(my_union),
  };

  quote! {
    impl #generics #struct_name_ident #generics #where_clause {
      fn describe(&self) -> String {
        let mut string = String::from(stringify!(#struct_name_ident));
        string.push_str(" is ");
        string.push_str(#description_str);
        string
      }
    }
  }
  .into()
}

fn gen_description_str_for_union(my_union: DataUnion) -> String {
  handle_named_fields(my_union.fields)
}

fn gen_description_str_for_struct(my_struct: DataStruct) -> String {
  match my_struct.fields {
    Named(fields) => handle_named_fields(fields),
    Unnamed(fields) => handle_unnamed_fields(fields),
    Unit => handle_unit(),
  }
}

fn handle_named_fields(fields: FieldsNamed) -> String {
  let my_named_field_idents = fields
    .named
    .iter()
    .map(|it| &it.ident);
  format!(
    "a struct with these named fields: {}",
    quote! {#(#my_named_field_idents), *}
  )
}

fn handle_unnamed_fields(fields: FieldsUnnamed) -> String {
  let my_unnamed_fields_count = fields.unnamed.iter().count();
  format!("a struct with {} unnamed fields", my_unnamed_fields_count)
}

fn handle_unit() -> String { format!("a unit struct") }

fn gen_description_str_for_enum(my_enum: DataEnum) -> String {
  let my_variant_idents = my_enum
    .variants
    .iter()
    .map(|it| &it.ident);
  format!(
    "an enum with these variants: {}",
    quote! {#(#my_variant_idents),*}
  )
}
