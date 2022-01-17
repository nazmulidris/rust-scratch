use crate::utils::{print_header2};

pub fn run() {
  print_hello("John Doe")
}

pub fn print_hello(text: &str) {
  print_header2("hello_world");
  println!("Hello, {}", text);
}
