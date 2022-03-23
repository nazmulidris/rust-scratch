use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{
  parse_macro_input, DataEnum, DataUnion, DeriveInput, FieldsNamed, FieldsUnnamed,
  Data::{Struct, Enum, Union},
  Fields::{Named, Unnamed, Unit},
};

pub fn macro_impl(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, data, .. } = parse_macro_input!(input);

  let description = match data {
    Struct(this_struct) => match this_struct.fields {
      Named(FieldsNamed { named, .. }) => {
        let ident_array = named.iter().map(|field| &field.ident);
        format!(
          "a struct with these named fields: {}",
          quote! {#(#ident_array), *}
        )
      }
      Unnamed(FieldsUnnamed { unnamed, .. }) => {
        let num_fields = unnamed.iter().count();
        format!("a struct with {} unnamed fields", num_fields)
      }
      Unit => format!("a unit struct"),
    },
    Enum(DataEnum { variants, .. }) => {
      let vs = variants.iter().map(|v| &v.ident);
      format!("an enum with these variants: {}", quote! {#(#vs),*})
    }
    Union(DataUnion {
      fields: FieldsNamed { named, .. },
      ..
    }) => {
      let ident_array = named.iter().map(|field| &field.ident);
      format!(
        "a union with these named fields: {}",
        quote! {#(#ident_array),*}
      )
    }
  };

  let output = quote! {
  impl #ident {
    fn describe(&self) -> String {
      let mut string = String::from(stringify!(#ident));
      string.push_str(" is ");
      string.push_str(#description);
      string
    }
  }
  };

  output.into()
}
