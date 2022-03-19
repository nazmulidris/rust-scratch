//! YT video of macros: <https://youtu.be/q6paRBbLgNw?t=657>
//!
//! Declarative macros are very limited in what they can do. Use proc macros when you hit
//! a wall with them. One limitation is that we can't pass generics to the macro.
//!
//! Eg: you can do this w/ the `alias!` macro: `alias! { use Arc<T> as T0<T> }`

use std::sync::{Arc, Mutex};

#[test]
fn test_strange_syntax() {
  macro_rules! alias {
    ($original_type:ty, $alias: ident) => {
      type $alias = $original_type;
    };
    ($original_type:ty as $alias: ident) => {
      type $alias = $original_type;
    };
    ($original_type:ty => $alias: ident) => {
      type $alias = $original_type;
    };
    (@ALIAS $original_type:ty | #TO $alias: ident) => {
      type $alias = $original_type;
    };
    (MAKE $alias: ident FROM $original_type:ty) => {
      type $alias = $original_type;
    };
    (|USE| $original_type:ty |AS| $alias: ident) => {
      type $alias = $original_type;
    };
    (use $original_type:ty as $alias: ident) => {
      type $alias = $original_type;
    };
  }

  // Most ergonomic version.
  alias! { use Arc<Mutex<i32>> as T0 };
  assert_eq!(
    T0::default().lock().unwrap().to_owned(),
    Arc::new(Mutex::new(0 as i32)).lock().unwrap().to_owned()
  );

  alias! { |USE| Arc<Mutex<i32>> |AS| T1 };
  assert_eq!(
    T1::default().lock().unwrap().to_owned(),
    Arc::new(Mutex::new(0 as i32)).lock().unwrap().to_owned()
  );

  alias! { Arc<Mutex<i32>> as T2 };
  assert_eq!(
    T2::default().lock().unwrap().to_owned(),
    Arc::new(Mutex::new(0 as i32)).lock().unwrap().to_owned()
  );

  alias! [ Arc<Mutex<i32>> => T3 ];
  assert_eq!(
    T3::default().lock().unwrap().to_owned(),
    Arc::new(Mutex::new(0 as i32)).lock().unwrap().to_owned()
  );

  alias! [ MAKE T4 FROM Arc<Mutex<i32>> ];
  assert_eq!(
    T4::default().lock().unwrap().to_owned(),
    Arc::new(Mutex::new(0 as i32)).lock().unwrap().to_owned()
  );

  alias! { @ALIAS Arc<Mutex<i32>> | #TO T5 };
  assert_eq!(
    T5::default().lock().unwrap().to_owned(),
    Arc::new(Mutex::new(0 as i32)).lock().unwrap().to_owned()
  );

  alias!(Arc<Mutex<i32>>, T6);
  assert_eq!(
    T6::default().lock().unwrap().to_owned(),
    Arc::new(Mutex::new(0 as i32)).lock().unwrap().to_owned()
  );
}
