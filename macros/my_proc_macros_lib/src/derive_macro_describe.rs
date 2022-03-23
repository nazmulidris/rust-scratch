use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{
  parse_macro_input, DataEnum, DataUnion, DeriveInput,
  Data::{Struct, Enum, Union},
  Fields::{Named, Unnamed, Unit},
  DataStruct, FieldsNamed, FieldsUnnamed,
};

pub fn macro_impl(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, data, .. } = parse_macro_input!(input);

  let description = match data {
    Struct(my_struct) => gen_description_str_for_struct(my_struct),
    Enum(my_enum) => gen_description_str_for_enum(my_enum),
    Union(my_union) => gen_description_str_for_union(my_union),
  };

  quote! {
    impl #ident {
      fn describe(&self) -> String {
        let mut string = String::from(stringify!(#ident));
        string.push_str(" is ");
        string.push_str(#description);
        string
      }
    }
  }
  .into() // Convert from proc_macro2::TokenStream to TokenStream.
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
  let my_named_field_idents = fields.named.iter().map(|it| &it.ident);
  format!(
    "a struct with these named fields: {}",
    quote! {#(#my_named_field_idents), *}
  )
}

fn handle_unnamed_fields(fields: FieldsUnnamed) -> String {
  let my_unnamed_fields_count = fields.unnamed.iter().count();
  format!("a struct with {} unnamed fields", my_unnamed_fields_count)
}

fn handle_unit() -> String {
  format!("a unit struct")
}

fn gen_description_str_for_enum(my_enum: DataEnum) -> String {
  let my_variant_idents = my_enum.variants.iter().map(|it| &it.ident);
  format!(
    "an enum with these variants: {}",
    quote! {#(#my_variant_idents),*}
  )
}
