pub trait IdentFromString {
  fn from_string(
    &self,
    string: &str,
  ) -> Self;
}

impl IdentFromString for proc_macro2::Ident {
  /// Generates a new identifier using the given string template as the name and the span
  /// from the `self` [Ident]. The template string can contain `{}` placeholders for the
  /// `self` [Ident] name.
  fn from_string(
    &self,
    name_with_template_placeholder: &str,
  ) -> Self {
    let name = str::replace(
      &name_with_template_placeholder,
      "{}",
      &self.to_string(),
    );
    proc_macro2::Ident::new(&name, self.span())
  }
}
