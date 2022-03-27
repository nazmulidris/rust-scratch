pub trait DataExt {
  fn is_struct(&self) -> bool;
}

impl DataExt for syn::Data {
  fn is_struct(&self) -> bool {
    match self {
      syn::Data::Struct(_data_struct) => true,
      _ => false,
    }
  }
}
