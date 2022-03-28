use core::panic;

use syn::Ident;

pub trait MetaExt {
  fn is_meta_name_value(&self) -> bool;
  fn get_meta_name_value_str(&self) -> String;
  fn get_meta_name_value_ident(&self) -> Ident;
}

/// Can be either a 👉 [syn::Meta::NameValue], [syn::Meta::List], or [syn::Meta::Path].
impl MetaExt for syn::Meta {
  fn is_meta_name_value(&self) -> bool {
    match self {
      syn::Meta::Path(_) => false,
      syn::Meta::List(_) => false,
      syn::Meta::NameValue(_) => true,
    }
  }

  fn get_meta_name_value_str(&self) -> String {
    match self {
      syn::Meta::Path(_) => panic!("Path found"),
      syn::Meta::List(_) => panic!("List found"),
      syn::Meta::NameValue(meta_name_value) => {
        let lit_str = match &meta_name_value.lit {
          syn::Lit::Str(lit_str) => lit_str.value(),
          _ => panic!("Expected a string literal"),
        };
        lit_str
      }
    }
  }

  /// ```no_run
  /// Path {
  ///   leading_colon: None,
  ///   segments: [
  ///       PathSegment {
  ///           ident: Ident {
  ///               ident: "key",
  ///               span: #0 bytes(510..513),
  ///           },
  ///           arguments: None,
  ///       },
  ///   ],
  /// }
  /// ```
  fn get_meta_name_value_ident(&self) -> Ident {
    match self {
      syn::Meta::Path(_) => panic!("Path found"),
      syn::Meta::List(_) => panic!("List found"),
      syn::Meta::NameValue(meta_name_value) => {
        if let Some(ident) = meta_name_value.path.get_ident() {
          ident.clone()
        } else {
          panic!("Expected an ident")
        }
      }
    }
  }
}
