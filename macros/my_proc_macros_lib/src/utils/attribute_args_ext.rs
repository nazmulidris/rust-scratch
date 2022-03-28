use crate::utils::{meta_ext::MetaExt, nested_meta_ext::NestedMeta};

pub trait AttributeArgsExt {
  fn get_key_value_pair(&self) -> (String, String);
}

/// The args take a key value pair like `#[attrib_macro_logger(key = "value")]`, which
/// evaluates to:
/// ```no_run
/// &args = [
///     Meta(
///         NameValue(
///             MetaNameValue {
///                 path: Path {
///                     leading_colon: None,
///                     segments: [
///                         PathSegment {
///                             ident: Ident {
///                                 ident: "key",
///                                 span: #0 bytes(510..513),
///                             },
///                             arguments: None,
///                         },
///                     ],
///                 },
///                 eq_token: Eq,
///                 lit: Str(
///                     LitStr {
///                         token: "value",
///                     },
///                 ),
///             },
///         ),
///     ),
/// ]
/// ```
impl AttributeArgsExt for syn::AttributeArgs {
  fn get_key_value_pair(&self) -> (String, String) {
    for nested_meta in self.iter() {
      if nested_meta.is_meta() {
        let meta = nested_meta.get_meta();
        if meta.is_meta_name_value() {
          let key = meta
            .get_meta_name_value_ident()
            .to_string();
          let value = meta.get_meta_name_value_str();
          return (key, value);
        }
      }
    }
    panic!("Expected a key value pair");
  }
}
