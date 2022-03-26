use quote::quote;
use syn::{parse_macro_input, Data::Struct, DataStruct, DeriveInput, Fields::Named};

use super::utils::IdentFromString;

/*
TODO:
- [x] Refactor helper functions into `utils.rs`
- [ ] Create a `<STRUCT>Builder` struct w/ all the fields
- [ ] Create a `<STRUCT>Builder` impl block w/ following methods
  - [ ] new()
  - [ ] fn corresponding to each field
  - [ ] build()
 */

/// Example #1: <https://github.com/dtolnay/syn/blob/master/examples/heapsize/heapsize_derive/src/lib.rs>
/// Example #2: <https://github.com/jonhoo/proc-macro-workshop/blob/master/builder/src/lib.rs>
pub fn derive_proc_macro_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let DeriveInput {
    ident: struct_name_ident,
    data,
    generics,
    ..
  } = parse_macro_input!(input);

  let builder_name_ident = struct_name_ident.from_string("{}Builder");

  let where_clause = &generics.where_clause;

  match data {
    Struct(ref data_struct) => {
      let gen_fns_ts = parse_named_fields_into_fns_ts(data_struct);
      quote! {
        impl #generics #struct_name_ident #generics #where_clause {
          #gen_fns_ts
        }
      }
      .into()
    }
    _ => quote! {}.into(),
  }
}

/// Returns [proc_macro2::TokenStream] (not [proc_macro::TokenStream]).
fn parse_named_fields_into_fns_ts(data_struct: &DataStruct) -> proc_macro2::TokenStream {
  match data_struct.fields {
    Named(ref fields) => {
      // Create iterator over named fields, holding generated function token streams.
      let fn_ts_iter = fields
        .named
        .iter()
        .map(|named_field| {
          let field_ident = named_field.ident.as_ref().unwrap();
          let fn_name_ident = field_ident.from_string("set_{}");
          let arg_ty = &named_field.ty;
          quote! {
            pub fn #fn_name_ident(mut self, value: #arg_ty) -> Self {
              self.#field_ident = value;
              self
            }
          }
        });

      // Unwrap iterator into a [proc_macro2::TokenStream].
      quote! {
        #(#fn_ts_iter)*
      }
    }
    _ => quote! {},
  }
}
