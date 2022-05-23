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

use crate::Style;
use r3bl_rs_utils::{CommonError, CommonResult};

#[derive(Default, Debug, Clone)]
pub struct Stylesheet {
  pub styles: Vec<Style>,
}

impl Stylesheet {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn add_style(
    &mut self,
    style: Style,
  ) -> CommonResult<()> {
    if style.id.is_empty() {
      return CommonError::new_err_with_only_msg("Style id cannot be empty");
    }
    self.styles.push(style);
    Ok(())
  }

  pub fn add_styles(
    &mut self,
    styles: Vec<Style>,
  ) -> CommonResult<()> {
    for style in styles {
      self.add_style(style)?;
    }
    Ok(())
  }

  pub fn get_style_by_id(
    &self,
    id: &str,
  ) -> Option<&Style> {
    self
      .styles
      .iter()
      .find(|style| style.id == id)
  }
}
