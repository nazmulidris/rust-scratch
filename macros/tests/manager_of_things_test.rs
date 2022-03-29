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

use my_lib::make_mutex_manager;
use std::sync::{Arc, Mutex, MutexGuard};

#[test]
fn test_make_manager_struct() {
  // Generate the struct.
  make_mutex_manager! { ThingManager manages i32 };

  // Create an manager_instance of the "manager" struct.
  let manager_instance = ThingManager::default();

  // 🔒 Each of the locked objects need to be wrapped in a block, or call `drop()` so the
  // mutex guard can be dropped and the tests won't deadlock.

  // 1. Test that `wrapped_thing` is created.
  {
    let locked_thing = manager_instance.wrapped_thing.lock().unwrap();
    assert_eq!(*locked_thing, 0);
  }

  // 2. Test that `get_locked_thing()` works.
  {
    let locked_thing = manager_instance.get_locked_thing();
    assert_eq!(*locked_thing, 0);
  }

  // 3. Test that `set_value_of_wrapped_thing()` works.
  {
    manager_instance.set_value_of_wrapped_thing(12);
    let locked_thing = manager_instance.get_locked_thing();
    assert_eq!(*locked_thing, 12);
  }

  // 4. Test that `get_arc()` => `with_arc_get_locked_thing()` &
  //    `with_arc_set_value_of_wrapped_thing()` works. Watch out for deadlock.
  {
    let arc_clone = manager_instance.get_arc();
    let locked_thing = ThingManager::with_arc_get_locked_thing(&arc_clone);
    assert_eq!(*locked_thing, 12);
    drop(locked_thing); // 🔒 Prevents deadlock below.

    ThingManager::with_arc_set_value_of_wrapped_thing(&arc_clone, 13);
    assert_eq!(*ThingManager::with_arc_get_locked_thing(&arc_clone), 13);
  }
}
