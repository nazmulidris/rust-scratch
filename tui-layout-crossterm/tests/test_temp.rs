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

use std::error::Error;
type ThunkResult<T> = Result<T, Box<dyn Error>>;
type ThunkFunction<T> = fn() -> T;

#[derive(Debug)]
struct Thunk<T>
where
  T: Clone,
{
  pub field: ThunkResult<T>,
  pub compute_field: ThunkFunction<T>,
}

impl<T> Thunk<T>
where
  T: Clone,
{
  pub fn new(expensive_method: ThunkFunction<T>) -> Self {
    Self {
      field: Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Not yet evaluated",
      ))),
      compute_field: expensive_method,
    }
  }

  pub fn access_field(&mut self) -> ThunkResult<T> {
    if self.field.is_err() {
      self.field = Ok((self.compute_field)());
    }
    if self.field.is_ok() {
      let field_value = self
        .field
        .as_ref()
        .unwrap()
        .clone();
      Ok(field_value.clone())
    } else {
      Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Can't be evaluated",
      )))
    }
  }
}

#[test]
fn test_name() {
  let mut thunk = Thunk::new(|| 1);
  let result = thunk.access_field();
  if result.is_err() {
    panic!("error");
  } else {
    let field_value = result.unwrap();
    assert_eq!(field_value, 1);
  }
}
