use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn derive_proc_macro_impl(input: TokenStream,) -> TokenStream {
  let DeriveInput {
    ident,
    data,
    generics,
    ..
  } = parse_macro_input!(input);

  quote! { /* todo */ }.into()
}
