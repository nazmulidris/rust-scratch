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

use crate::*;
use std::{
  error::Error,
  fmt::{Display, Result},
};

/// Main error struct.
/// https://learning-rust.github.io/docs/e7.custom_error_types.html
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LayoutError {
  err_type: LayoutErrorType,
  msg: String,
}

/// Specific types of errors.
#[derive(Debug, Clone, Copy)]
pub enum LayoutErrorType {
  MismatchedEnd,
  MismatchedStart,
  LayoutStackUnderflow,
}

/// Implement [`Error`] trait.
impl Error for LayoutError {}

/// Implement [`Display`] trait (needed by [`Error`] trait).
impl Display for LayoutError {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> Result {
    write!(f, "{:?}", self)
  }
}

/// Implement constructor that is compatible w/ [`ResultCommon<T>`].
impl LayoutError {
  pub fn new(
    err_type: LayoutErrorType,
    msg: String,
  ) -> Box<Self> {
    Box::new(LayoutError { err_type, msg })
  }

  pub fn format_msg_with_stack_len(
    layout_stack: &Vec<Layout>,
    msg: &str,
  ) -> String {
    format!(
      "{}, layout_stack.len(): {}",
      msg,
      layout_stack.len()
    )
  }
}
