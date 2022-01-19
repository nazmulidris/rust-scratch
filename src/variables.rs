use crate::utils::print_header;

const MS_IN_MIN: i32 = 60 * 1_000; // Must not be something that has to be calculated at runtime.

pub fn run() {
  print_header("variables");
  simple();
}

fn simple() {
  let _x = 2 * MS_IN_MIN; // Use underscore in variables to bypass warning it isn't used.
  let _x = 10 * _x; // Shadowing.
  {
    let _x = 1_000;
  }
}

/// https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html
#[test]
fn test_simple() {
  let _x = 2 * MS_IN_MIN; // Use underscore in variables to bypass warning it isn't used.
  let _x = 10 * _x; // Shadowing.
  {
    let _x = 1_000.to_string();
    assert_eq!(_x, "1000");
  }
  assert_eq!(_x, 10 * 2 * MS_IN_MIN);
}

/*
/// Use the following if you want a test module.
/// https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_template() {}
}
*/
