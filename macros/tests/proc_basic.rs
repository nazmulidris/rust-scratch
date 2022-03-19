/// Add lib crate for macros: <https://dev.to/dandyvica/rust-procedural-macros-step-by-step-tutorial-36n8>
/// Macro how to: <https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros>
/// Macro how to: <https://doc.rust-lang.org/book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes>
#[test]
fn test_proc_macro() {
  use my_proc_macros_lib::make_answer;
  make_answer!();
  assert_eq!(answer(), 42);
}
