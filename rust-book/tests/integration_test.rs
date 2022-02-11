use ansi_term::Colour::Green;
use rust_book_lib::utils::{style_primary, type_of};

/// Rust book: https://doc.rust-lang.org/book/ch11-03-test-organization.html#the-tests-directory
#[test]
fn test_color_styles_work() {
  let text = "foo";
  let styled_text = style_primary(text);
  assert_eq!(Green.bold().paint(text), styled_text);
}

#[test]
fn test_type_of_works() {
  let text = "foo".to_string();
  let type_of_text = type_of(&text);
  assert_eq!(type_of_text, "alloc::string::String");
}
