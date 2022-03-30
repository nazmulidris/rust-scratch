/*
 *   Copyright (c) 2022 Nazmul Idris
 *   All rights reserved.

 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at

 *   http://www.apache.org/licenses/LICENSE-2.0

 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
*/

use std::sync::{Arc, Mutex};

/// Experiment - unwrap an Arc.
/// https://users.rust-lang.org/t/lock-an-arc-mutex-in-one-call/65337
/// https://docs.rs/sugars/latest/sugars/macro.arc.html
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
