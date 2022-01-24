/// Rust book: https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html
/// Rust book: https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
/// Rust book: https://doc.rust-lang.org/book/ch04-03-slices.html
pub fn run() {}

// Functions that exercise borrowing, moving, etc. used in tests.

#[allow(dead_code)]
fn borrow_str(string: &str) -> usize { string.len() }

#[allow(dead_code)]
fn borrow_string(string: &String) -> usize { string.len() }

#[allow(dead_code)]
fn borrow_and_mutate_string(string: &mut String) -> usize {
  string.push_str("_mutated");
  string.len()
}

#[allow(dead_code)]
fn give_ownership_of_string() -> String { String::from("a_new_string") }

#[allow(dead_code)]
fn take_and_gives_back_ownership_of_string(string: String) -> String { string }

// Tests.

#[test]
fn test_string_memory_allocation() {
  // Allocated on stack.
  let str = "hello_world";
  borrow_str(str);
  assert_eq!(str.len(), 11);

  // Allocated on heap.
  let mut string = String::from("hello"); // Same as "hello".to_string();
  borrow_string(&string);
  string.push_str("_world");
  assert_eq!(string.len(), 11);
}

/// https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#memory-and-allocation
/// Meaning of "&*": https://stackoverflow.com/a/41273406/2085356
/// 1. First dereference: *
/// 2. Then reference: & (possibly to a different type!)
///
/// let hello_str: &str = &*String::from("hello");
/// 1. Read above expression RTL
/// 2. String ==deref=> str ==ref=> &str
#[test]
fn test_move_aka_shallow_copy_and_invalidate() {
  let old_owner_of_s1 = String::from("hello"); // Allocate on heap.
  borrow_string(&old_owner_of_s1);
  borrow_str(&*old_owner_of_s1); // https://stackoverflow.com/a/41273406/2085356
  borrow_str(old_owner_of_s1.as_str()); // Same as above.
  borrow_str(old_owner_of_s1.as_ref()); // Same as above.
  assert_eq!(old_owner_of_s1.len(), 5);

  // Move ownership of s1:
  // - new_owner_of_s1 is the new owner of the heap allocated object.
  // - old_owner_of_s1 is de-scoped.
  // - Makes a shallow copy & invalidates first variable, aka, perform "move".
  let new_owner_of_s1 = old_owner_of_s1;
  borrow_string(&new_owner_of_s1);
  assert_eq!(new_owner_of_s1.len(), 5);
}

#[allow(dead_code)]
fn take_ownership_of_string(_param: String) {}

/// https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#ways-variables-and-data-interact-clone
#[test]
fn test_clone() {
  let owner1 = String::from("hello");
  let owner2 = owner1.clone();
  assert_eq!(owner1.len(), 5);
  assert_eq!(owner2.len(), 5);
  take_ownership_of_string(owner2); // This invalidates owner2 reference.
}

/// https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#stack-only-data-copy
/// Copy by value works for types: integer, floating point, boolean, char, tuples of these types.
#[test]
fn test_copy() {
  let owner1 = 12;
  let owner2 = owner1; // Copy by value.
  assert_eq!(owner1, 12);
  assert_eq!(owner2, 12);
}

/// https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#return-values-and-scope
#[test]
fn test_ownership_with_functions() {
  let string_from_fn = give_ownership_of_string();
  assert_eq!(string_from_fn.len(), 12);
  take_ownership_of_string(string_from_fn); // string_from_fn is invalidated.

  let another_string = take_and_gives_back_ownership_of_string("string2".to_string());
  assert_eq!(another_string.len(), 7);
  take_ownership_of_string(another_string); // another_string is invalidated.
}

#[test]
fn test_borrowing_with_functions() {
  let str_1_ref = "str1";
  let string_1 = "string1".to_string();
  let string_1_ref = &string_1;
  assert_eq!(borrow_str(str_1_ref), 4);
  assert_eq!(borrow_string(string_1_ref), 7);
}

/// https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
#[test]
fn test_borrowing_mutably_with_functions() {
  let mut my_string = "string1".to_string();
  let mut_ref_1 = &mut my_string;
  borrow_and_mutate_string(mut_ref_1);
  assert_eq!(mut_ref_1.len(), 7 + 8);

  // 🧨 Can't create more than one mutable ref simultaneously. The following line will fail.
  // let mut_ref_2 = &mut my_string;
}

#[test]
fn test_borrowing_both_mutably_and_immutably_with_functions() {
  let mut my_string = "string1".to_string();
  let mut_ref_1 = &mut my_string;
  assert_eq!(mut_ref_1.len(), 7);

  // 🧙 Can create multiple immutable refs w/ just 1 mutable ref IF they're not used in the same
  // scope. Note that a reference’s scope starts from where it is introduced and continues through
  // the last time that reference is used.
  let immut_ref_1 = &my_string;
  assert_eq!(immut_ref_1.len(), 7);
  let immut_ref_2 = &my_string;
  assert_eq!(immut_ref_2.len(), 7);

  // 🧨 The following line will fail. The scope mixes immutable & mutable references simultaneously.
  // println!("{}, {}, and {}", immut_ref_1, immut_ref_2, mut_ref_1);
}

/// https://doc.rust-lang.org/book/ch04-03-slices.html
#[test]
fn test_slice_string() {
  let mut my_string = "word1 word2".to_string();

  // 👎 This is the sub-optimal way of doing things.
  fn find_first_word_in_string_without_using_slice(
    string: &String
  ) -> usize {
    let bytes = string.as_bytes();
    for (index, &byte) in bytes.iter().enumerate() {
      if byte == b' ' { return index; }
    }
    string.len()
  }
  assert_eq!(find_first_word_in_string_without_using_slice(&my_string), 5);

  // 👍 This is the optimal way of doing things (using slices).
  fn find_first_word_in_string(
    string: &String // 🧙 Note that making the argument type &str is more flexible.
  ) -> &str {
    let bytes = string.as_bytes();
    for (index, &byte) in bytes.iter().enumerate() {
      if byte == b' ' { return &string[..index]; }
    }
    return &string[..];
  }
  let slice_to_my_string = find_first_word_in_string(&my_string);
  assert_eq!(slice_to_my_string, "word1");
  my_string.clear();

  // 🧨 The following line is no longer possible, since clearing the my_string impacted the slice
  // slice_to_my_string that was dependent on it.
  // assert_eq!(slice_to_my_string, "");
}

#[test]
fn test_slice_array() {
  let array = [1, 2, 3, 4, 5];
  assert_eq!(&array[..2], &[1, 2]);
  assert_eq!(&array[2..], &[3, 4, 5]);
}
