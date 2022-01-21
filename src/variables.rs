use crate::utils::print_header;

/// https://stackoverflow.com/a/25877389/2085356
#[allow(dead_code)]
const MS_IN_MIN: i32 = 60 * 1_000; // Must not be something that has to be calculated at runtime.

/// Rust book - https://doc.rust-lang.org/book/ch03-02-data-types.html
pub fn run() {
  print_header("variables");
}

#[test]
fn test_shadowing() {
  let _x = 2 * MS_IN_MIN; // Use underscore in variables to bypass warning it isn't used.
  let _x = 10 * _x; // Shadowing.
  {
    let _x = 1_000.to_string();
    assert_eq!(_x, "1000");
  }
  assert_eq!(_x, 10 * 2 * MS_IN_MIN);
}

#[test]
fn test_tuples_simple() {
  let tuple1: (i32, String) = (100, "123".to_string());
  let (number, text) = tuple1;
  assert_eq!(number, 100);
  // assert_eq!(tuple1.0, 100); // Works because i32 gets copied.
  assert_eq!(text, "123");
  // assert_eq!(tuple1.1, "123"); // Fails because tuple1.1 (String) gets moved above.
}

#[test]
fn test_tuples_complex_1() {
  let tuple1: (i32, &String) = (100, &"123".to_string());
  let (number, text_ptr) = tuple1;

  assert_eq!(number, 100);
  assert_eq!(text_ptr, "123");
  assert_eq!(*text_ptr, "123");

  assert_eq!(tuple1.0, 100);
  assert_eq!(tuple1.1, "123");
  assert_eq!(*tuple1.1, "123");
}

#[test]
fn test_tuples_complex_2() {
  let tuple1: (&i32, &String) = (&100, &"123".to_string());
  let (number_ptr, text_ptr) = tuple1;

  assert_eq!(*number_ptr, 100);
  assert_eq!(*text_ptr, "123");

  assert_eq!(*tuple1.0, 100);
  assert_eq!(*tuple1.1, "123");
}
