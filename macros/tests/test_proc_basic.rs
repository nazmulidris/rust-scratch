use my_proc_macros_lib::make_answer;

#[test]
fn test_proc_macro() {
  make_answer!();
  assert_eq!(answer(), 42);
}
