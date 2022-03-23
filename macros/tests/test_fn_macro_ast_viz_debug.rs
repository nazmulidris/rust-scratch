use my_proc_macros_lib::fn_macro_ast_viz_debug;

#[test]
fn test_proc_macro() {
  fn_macro_ast_viz_debug!();
  assert_eq!(foo(), 42);
}
