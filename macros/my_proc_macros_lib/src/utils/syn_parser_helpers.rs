use quote::quote;
use syn::{Data::Struct, DataStruct, Fields::Named};

/// Returns [proc_macro2::TokenStream] (not [proc_macro::TokenStream]).
pub fn transform_named_fields_into_ts(
  data_struct: &DataStruct,
  transform_named_field_fn: &dyn Fn(&syn::Field) -> proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
  match data_struct.fields {
    Named(ref fields) => {
      // Create iterator over named fields, holding generated props token streams.
      let props_ts_iter = fields
        .named
        .iter()
        .map(|named_field| transform_named_field_fn(named_field));

      // Unwrap iterator into a [proc_macro2::TokenStream].
      quote! {
        #(#props_ts_iter)*
      }
    }
    _ => quote! {},
  }
}

/// If [syn::Data] contains [syn::DataStruct] then parse it, and generate a
/// [proc_macro2::TokenStream] and return it.
pub fn with_data_struct_make_ts(
  data: &syn::Data,
  data_struct_transform_fn: &dyn Fn(&syn::DataStruct) -> proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
  match data {
    Struct(ref data_struct) => data_struct_transform_fn(data_struct),
    _ => quote! {},
  }
}
