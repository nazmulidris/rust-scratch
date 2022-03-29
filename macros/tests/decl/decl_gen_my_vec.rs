//! A `vec`-like macro: <https://gist.github.com/jonhoo/ec57882a976a2d2a92b3138ea25cd45a>

macro_rules! my_vec {
  () => {{
    let vec = Vec::new();
    vec
  }};
  ($($el: expr) => *) => {{
      let mut vec = Vec::new();
      $(
        vec.push($el);
      )*
      vec
  }};
}

#[test]
fn test_empty() {
  let vec: Vec<i32> = my_vec!();
  assert_eq!(vec.len(), 0);
}

#[test]
fn test_double() {
  let vec: Vec<i32> = my_vec![1 => 2 => 3];
  assert_eq!(vec.len(), 3);
}
