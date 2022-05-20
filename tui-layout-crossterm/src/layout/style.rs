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

use crate::UnitType;
use bitflags::bitflags;
use crossterm::style::Color;
use r3bl_rs_utils::Builder;

/// Use the `StyleBuilder` to create a `Style`. `Style` objects are meant to be immutable.
/// If you need to modify a `Style`, you should use the `StyleBuilder` to create a new
/// one.
#[derive(Default, Builder, Debug)]
pub struct Style {
  pub color_fg: Option<Color>,
  pub color_bg: Option<Color>,
  pub padding: Option<UnitType>,
  pub bold: bool,
  pub italic: bool,
  pub underline: bool,
  pub cached_bitflags: Option<StyleFlag>,
}

bitflags! {
  /// https://docs.rs/bitflags/0.8.2/bitflags/macro.bitflags.html
  pub struct StyleFlag: u8 {
    const COLOR_FG_SET  = 0b00000001;
    const COLOR_BG_SET  = 0b00000010;
    const BOLD_SET      = 0b00000100;
    const ITALIC_SET    = 0b00001000;
    const UNDERLINE_SET = 0b00010000;
    const PADDING_SET   = 0b00100000;
  }
}

/// https://crates.io/crates/lazy-st
impl Style {
  /// The `StyleFlag` is lazily computed and cached after the first time it is evaluated.
  /// A `Style` should be built using via `StyleBuilder and the expectation is that once
  /// built, the style won't be modified.
  pub fn get_bitflags(&mut self) -> StyleFlag {
    if self.cached_bitflags.is_none() {
      self.cached_bitflags = Some(self.gen_bitflags());
    }
    self.cached_bitflags.unwrap()
  }

  fn gen_bitflags(&self) -> StyleFlag {
    let mut mask = StyleFlag::empty();

    if self.color_fg.is_some() {
      mask.insert(StyleFlag::COLOR_FG_SET);
    }
    if self.color_bg.is_some() {
      mask.insert(StyleFlag::COLOR_BG_SET);
    }
    if self.padding.is_some() {
      mask.insert(StyleFlag::PADDING_SET);
    }
    if self.bold {
      mask.insert(StyleFlag::BOLD_SET);
    }
    if self.italic {
      mask.insert(StyleFlag::ITALIC_SET);
    }
    if self.underline {
      mask.insert(StyleFlag::UNDERLINE_SET);
    }

    mask
  }
}
