/// Rust book: https://doc.rust-lang.org/book/ch03-05-control-flow.html
pub fn run() {}

/// Return values from loop.
#[test]
fn test_loop_1() {
  let mut counter = 0;
  let result = loop {
    counter += 1;
    if counter == 10 {
      break counter * 2; // Return this expression (no terminating semicolon).
    }
  };
  assert_eq!(result, 20);
}

/// Labelled loops and breaking out of them.
#[test]
fn test_loop_2() {
  let mut outer_count = 0;
  let mut inner_count = 0;
  'OUTER: loop {
    outer_count += 1;
    'INNER: loop {
      inner_count += 1;
      if inner_count == 9 { break 'INNER; }
      if outer_count == 2 { break 'OUTER; }
    };
  };
  assert_eq!(inner_count, 10);
  assert_eq!(outer_count, 2);
}

/// for loop.
#[test]
fn test_for_loop() {
  let array = [0, 10, 20];
  for element in array { assert!(array.contains(&element)); }
}

/// for each loop.
#[test]
fn test_for_each_loop() {
  let array = [0, 10, 20];
  array.iter()
    .enumerate()
    .for_each(|(_index, value)| {
      assert!(array.contains(value))
    });
}

/// Range and for loop.
/// Range and borrowing limitations: https://stackoverflow.com/a/62480671/2085356
#[test]
fn test_range_for_loop() {
  let range = 1..4;
  let rev_range = range.clone().rev();
  for number in rev_range {
    assert!(range.contains(&number))
  }
}
