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

use std::ops::{Add, AddAssign};

use crate::UnitType;
use bitflags::bitflags;
use crossterm::style::Color;
use r3bl_rs_utils::{unwrap_option_or_compute_if_none, Builder};

/// Use the `StyleBuilder` to create a `Style`. `Style` objects are meant to be immutable.
/// If you need to modify a `Style`, you should use the `StyleBuilder` to create a new
/// one.
#[derive(Default, Builder, Debug, Clone, PartialEq, Eq)]
pub struct Style {
  pub id: String,
  pub bold: bool,
  pub italic: bool,
  pub underline: bool,
  pub computed: bool,
  pub color_fg: Option<Color>,
  pub color_bg: Option<Color>,
  pub margin: Option<UnitType>,
  pub cached_bitflags: Option<StyleFlag>,
}

bitflags! {
  /// https://docs.rs/bitflags/0.8.2/bitflags/macro.bitflags.html
  pub struct StyleFlag: u8 {
    const COLOR_FG_SET  = 0b0000_0001;
    const COLOR_BG_SET  = 0b0000_0010;
    const BOLD_SET      = 0b0000_0100;
    const ITALIC_SET    = 0b0000_1000;
    const UNDERLINE_SET = 0b0001_0000;
    const MARGIN_SET    = 0b0010_0000;
    const COMPUTED_SET  = 0b0100_0000;
  }
}

impl Style {
  /// The `StyleFlag` is lazily computed and cached after the first time it is evaluated.
  /// A `Style` should be built using via `StyleBuilder and the expectation is that once
  /// built, the style won't be modified.
  pub fn get_bitflags(&mut self) -> StyleFlag {
    unwrap_option_or_compute_if_none! {
      self.cached_bitflags,
      || self.gen_bitflags()
    }
  }

  pub fn reset_bitflags(&mut self) {
    self.cached_bitflags = None;
  }

  fn gen_bitflags(&self) -> StyleFlag {
    let mut it = StyleFlag::empty();

    if self.color_fg.is_some() {
      it.insert(StyleFlag::COLOR_FG_SET);
    }
    if self.color_bg.is_some() {
      it.insert(StyleFlag::COLOR_BG_SET);
    }
    if self.margin.is_some() {
      it.insert(StyleFlag::MARGIN_SET);
    }
    if self.bold {
      it.insert(StyleFlag::BOLD_SET);
    }
    if self.italic {
      it.insert(StyleFlag::ITALIC_SET);
    }
    if self.underline {
      it.insert(StyleFlag::UNDERLINE_SET);
    }
    if self.computed {
      it.insert(StyleFlag::COMPUTED_SET);
    }

    it
  }
}

/// Implement specificity behavior for [Style] by implementing [Add] trait. Here's the
/// rule: `Style + Style (overrides) = Style`.
/// - https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
impl Add<Self> for Style {
  type Output = Self;

  fn add(
    self,
    other: Self,
  ) -> Self {
    let mut new_style = self.clone();

    // Computed style has no id.
    new_style.computed = true;
    new_style.id = "".to_string();

    // other (if set) overrides self.
    if let Some(color_fg) = other.color_fg {
      new_style.color_fg = Some(color_fg);
    }
    if let Some(color_bg) = other.color_bg {
      new_style.color_bg = Some(color_bg);
    }
    if let Some(margin) = other.margin {
      new_style.margin = Some(margin);
    }
    if other.bold {
      new_style.bold = true;
    }
    if other.italic {
      new_style.italic = true;
    }
    if other.underline {
      new_style.underline = true;
    }

    // Recalculate the bitflags.
    new_style.reset_bitflags();
    new_style.get_bitflags();

    new_style
  }
}

impl AddAssign<&Style> for Style {
  fn add_assign(
    &mut self,
    other: &Style,
  ) {
    *self = self.clone() + other.clone();
  }
}
