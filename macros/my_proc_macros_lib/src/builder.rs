use quote::quote;
use syn::{parse_macro_input, DataStruct, DeriveInput};

use super::utils::{data_ext::DataExt,
                   ident_ext::IdentExt,
                   syn_parser_helpers::{transform_named_fields_into_ts,
                                        with_data_struct_make_ts}};

const BUILDER_DOC_URL: &str = "https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder";

/// Example #1: <https://github.com/dtolnay/syn/blob/master/examples/heapsize/heapsize_derive/src/lib.rs>
/// Example #2: <https://github.com/jonhoo/proc-macro-workshop/blob/master/builder/src/lib.rs>
pub fn derive_proc_macro_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let DeriveInput {
    ident: struct_name_ident,
    data,
    generics,
    ..
  }: DeriveInput = parse_macro_input!(input);

  if data.is_struct() {
    with_data_struct_make_ts(&data, &|data_struct| {
      let where_clause = &generics.where_clause;
      let builder_name_ident = struct_name_ident.from_string("{}Builder");
      let gen_fns_ts = transform_named_fields_into_fns_ts(data_struct);
      let gen_props_ts = transform_named_fields_to_props_ts(data_struct);
      let doc_struct = format!(
        "Implements the [builder pattern] for [`{}`].\n[builder pattern]: {}",
        &struct_name_ident, BUILDER_DOC_URL
      );
      quote! {
        #[doc = #doc_struct]
        struct #builder_name_ident #generics #where_clause {
          #gen_props_ts
        }

        impl #generics #struct_name_ident #generics #where_clause {
          #gen_fns_ts
        }
      }
    })
  } else {
    quote! {}
  }
  .into()
}

/// Returns [proc_macro2::TokenStream] (not [proc_macro::TokenStream]).
fn transform_named_fields_to_props_ts(
  data_struct: &DataStruct
) -> proc_macro2::TokenStream {
  transform_named_fields_into_ts(data_struct, &|named_field| {
    let field_ident = named_field.ident.as_ref().unwrap();
    let field_ty = &named_field.ty;
    quote! {
      pub #field_ident: #field_ty,
    }
  })
}

/// Returns [proc_macro2::TokenStream] (not [proc_macro::TokenStream]).
fn transform_named_fields_into_fns_ts(
  data_struct: &DataStruct
) -> proc_macro2::TokenStream {
  transform_named_fields_into_ts(data_struct, &|named_field| {
    let field_ident = named_field.ident.as_ref().unwrap();
    let fn_name_ident = field_ident.from_string("set_{}");
    let arg_ty = &named_field.ty;
    quote! {
      pub fn #fn_name_ident(mut self, value: #arg_ty) -> Self {
        self.#field_ident = value;
        self
      }
    }
  })
}
