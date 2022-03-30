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

use std::sync::{Arc, Mutex, MutexGuard};

/// Experiment - Generate a "manager" for the given "thing". The "thing" is of type
/// `$field_type`. The "manager" wraps it in a lock (`Mutex`), which is wrapped in an arc
/// (`Arc`). One constraint is that the field type has to be `Default`.
///
/// - `$struct_name` = The name of the generated struct (the "manager").
/// - `$field_name` = The name of the instance in the generated struct.
/// - `$field_type` = The type of the instance in the generated struct.
macro_rules! make_manager {
  ($struct_name: ident manages { $field_name: ident: $field_type: ty } ) => {
    #[derive(Debug)]
    struct $struct_name
    where
      $field_type: Default,
    {
      $field_name: Arc<Mutex<$field_type>>,
    }

    impl Default for $struct_name {
      fn default() -> Self {
        Self {
          $field_name: Arc::new(Mutex::new(Default::default())),
        }
      }
    }

    #[allow(dead_code)]
    impl $struct_name {
      pub fn get_arc_clone(&self) -> Arc<Mutex<$field_type>> {
        self.$field_name.clone()
      }

      pub fn set_field(
        &self,
        value: $field_type,
      ) {
        *self.$field_name.lock().unwrap() = value;
      }

      pub fn get_field(&self) -> MutexGuard<$field_type> {
        self.$field_name.lock().unwrap()
      }

      pub fn get_from_arc(my_arc: &Arc<Mutex<$field_type>>) -> MutexGuard<$field_type> {
        my_arc.lock().unwrap()
      }
    }
  };
}

#[test]
fn test_make_manager_struct() {
  // Generate the struct.
  make_manager! {
    ThingManager manages { field_1: i32 }
  };

  // Create an instance of the "manager" struct.
  let instance = ThingManager::default();

  // 🔒 Each of these locked blocks need to be wrapped in a block, so the mutex guard can
  // be dropped and the tests won't deadlock.

  // 1. Test that `field_1` is created.
  {
    let field_1 = instance.field_1.lock().unwrap();
    assert_eq!(*field_1, 0);
  }

  // 2. Test that `get_field()` works.
  {
    let field_1 = instance.get_field();
    assert_eq!(*field_1, 0);
  }

  // 3. Test that `set_field()` works.
  {
    instance.set_field(12);
    let field_1 = instance.get_field();
    assert_eq!(*field_1, 12);
  }

  // 4. Test that `get_arc_clone()` & `get_from_arc()` works.
  {
    let arc_clone = instance.get_arc_clone();
    let field_1 = ThingManager::get_from_arc(&arc_clone);
    assert_eq!(*field_1, 12);
  }
}
