use my_proc_macros_lib::simple_function_macro_make_a_fn;

#[test]
fn test_proc_macro() {
  simple_function_macro_make_a_fn!();
  assert_eq!(foo(), 42);
}
