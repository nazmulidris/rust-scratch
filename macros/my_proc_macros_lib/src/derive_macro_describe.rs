use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DataEnum, DataUnion, DeriveInput, FieldsNamed, FieldsUnnamed};

pub fn macro_impl(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, data, .. } = parse_macro_input!(input);

  let description = match data {
    syn::Data::Struct(my_struct) => match my_struct.fields {
      syn::Fields::Named(FieldsNamed { named, .. }) => {
        let ident_array = named.iter().map(|field| &field.ident);
        format!(
          "a struct with these named fields: {}",
          quote! {#(#ident_array), *}
        )
      }
      syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
        let num_fields = unnamed.iter().count();
        format!("a struct with {} unnamed fields", num_fields)
      }
      syn::Fields::Unit => format!("a unit struct"),
    },
    syn::Data::Enum(DataEnum { variants, .. }) => {
      let vs = variants.iter().map(|v| &v.ident);
      format!("an enum with these variants: {}", quote! {#(#vs),*})
    }
    syn::Data::Union(DataUnion {
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
      fn describe() {
      println!("{} is {}.", stringify!(#ident), #description);
      }
  }
  };

  output.into()
}
