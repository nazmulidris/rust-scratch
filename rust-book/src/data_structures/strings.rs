/// Rust book: https://doc.rust-lang.org/book/ch08-02-strings.html
pub fn run() {}

#[test]
fn test_basic() {
  struct Person {
    name: String,
    age: u64,
  }

  impl Person {
    fn new(name: &str, age: u64) -> Person {
      Person {
        name: String::from(name), // Clone.
        age, // Copy.
      }
    }

    fn get_name<'a>(self: &'a Self) -> &'a str {
      &self.name
    }

    fn get_age(self: &Self) -> u64 {
      self.age
    }
  }

  let p1 = Person::new("John", 42);
  let p2 = Person::new("Jane", 42);

  assert_eq!(p1.get_name(), "John");
  assert_eq!(p1.get_age(), 42);
  assert_eq!(p2.get_name(), "Jane");
  assert_eq!(p2.get_age(), 42);
}
