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

type ThunkResult<T> = Result<T, Box<ThunkError>>;
type ThunkFunction<T> = fn() -> ThunkResult<T>;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ThunkError {
  err_type: ThunkErrorType,
  msg: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum ThunkErrorType {
  ComputeFieldFnError,
}

#[derive(Debug)]
enum ThunkState<T>
where
  T: Clone,
{
  NotComputedYet,
  ComputedResultingInError(ThunkError),
  ComputedResultingInValue(T),
}

#[derive(Debug)]
struct Thunk<T>
where
  T: Clone,
{
  pub field: ThunkState<T>,
  pub compute_field_fn: ThunkFunction<T>,
}

impl<T> Thunk<T>
where
  T: Clone,
{
  pub fn new(expensive_method: ThunkFunction<T>) -> Self {
    Self {
      field: ThunkState::NotComputedYet,
      compute_field_fn: expensive_method,
    }
  }

  pub fn access_field(&mut self) -> ThunkResult<T> {
    match self.field {
      ThunkState::NotComputedYet => {
        let computed_field_value_result = (self.compute_field_fn)();
        match computed_field_value_result {
          Ok(computed_field_value) => {
            self.field =
              ThunkState::ComputedResultingInValue(computed_field_value.clone());
            return Ok(computed_field_value);
          }
          Err(e) => {
            let e_clone = *e.clone();
            self.field = ThunkState::ComputedResultingInError(e_clone);
            return Err(e);
          }
        }
      }
      ThunkState::ComputedResultingInError(_) => todo!(),
      ThunkState::ComputedResultingInValue(_) => todo!(),
    }

    // match self.field {
    //   ThunkState::ComputedResultingInValue(&value) => Ok(value.clone()),
    //   _ => Err(Box::new(std::io::Error::new(
    //     std::io::ErrorKind::Other,
    //     "Can't be evaluated",
    //   ))),
    // }

    // if self.field.is_ok() {
    //   let field_value = self
    //     .field
    //     .as_ref()
    //     .unwrap()
    //     .clone();
    //   Ok(field_value.clone())
    // } else {
    //   Err(Box::new(std::io::Error::new(
    //     std::io::ErrorKind::Other,
    //     "Can't be evaluated",
    //   )))
    // }
  }
}

#[test]
fn test_name() {
  let mut thunk = Thunk::new(|| Ok(1));
  let result = thunk.access_field();
  if result.is_err() {
    panic!("error");
  } else {
    let field_value = result.unwrap();
    assert_eq!(field_value, 1);
  }
}
