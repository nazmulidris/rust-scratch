use std::sync::{Arc, Mutex};

/// Experiment - unwrap an Arc.
/// https://users.rust-lang.org/t/lock-an-arc-mutex-in-one-call/65337
/// https://docs.rs/sugars/2.0.0/sugars/macro.arc.html
macro_rules! lock_unwrap_arc {
  ( $mutex_arc:expr ) => {
    *$mutex_arc.lock().unwrap()
  };
}

#[test]
fn test_lock_mutex_arc() {
  let arc = Arc::new(Mutex::new(12));
  let foo = lock_unwrap_arc!(arc);
  assert_eq!(foo, 12);
}
