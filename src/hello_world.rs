pub fn run() {
  let name: String = String::from("John Doe");
  print_hello(&name)
}

pub fn print_hello(arg: &String) {
  // let copy = String::from(arg);
  println!("Hello, {}", arg);
}
