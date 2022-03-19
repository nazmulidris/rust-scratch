/// Experiment - generate a simple struct w/ given field declarations.
macro_rules! make_struct {
  ($name: ident, $($field_name: ident: $ty: ty),*) => {
    #[derive(Debug)]
    struct $name { $($field_name: $ty),* }
  }
}

#[test]
fn test_make_struct() {
  make_struct!(Thing, field_1: i32, field_2: String);
  let instance = Thing {
    field_1: 12,
    field_2: "abc".to_string(),
  };
  assert_eq!(instance.field_1, 12);
  assert_eq!(instance.field_2, "abc");
}