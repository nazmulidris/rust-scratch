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

//! https://docs.rs/bitmask/latest/bitmask/macro.bitmask.html

use crossterm::style::Color;
use r3bl_rs_utils::{debug, with_mut};
use tui_layout_crossterm::{Style, StyleBuilder, StyleFlag, Stylesheet};

#[test]
fn test_bitflags() {
  with_mut! {
    StyleFlag::empty(),
    as mask1,
    run {
      mask1.insert(StyleFlag::UNDERLINE_SET);
      mask1.insert(StyleFlag::ITALIC_SET);
      assert!(mask1.contains(StyleFlag::UNDERLINE_SET));
      assert!(mask1.contains(StyleFlag::ITALIC_SET));
      assert!(!mask1.contains(StyleFlag::COLOR_FG_SET));
      assert!(!mask1.contains(StyleFlag::COLOR_BG_SET));
      assert!(!mask1.contains(StyleFlag::BOLD_SET));
      assert!(!mask1.contains(StyleFlag::PADDING_SET));
    }
  };

  with_mut! {
    StyleFlag::BOLD_SET | StyleFlag::ITALIC_SET,
    as mask2,
    run {
      assert!(mask2.contains(StyleFlag::BOLD_SET));
      assert!(mask2.contains(StyleFlag::ITALIC_SET));
      assert!(!mask2.contains(StyleFlag::UNDERLINE_SET));
      assert!(!mask2.contains(StyleFlag::COLOR_FG_SET));
      assert!(!mask2.contains(StyleFlag::COLOR_BG_SET));
      assert!(!mask2.contains(StyleFlag::PADDING_SET));
    }
  }

  assert_eq!(mask1.contains(mask2), false);
}

#[test]
fn test_style() {
  let mut style = make_a_style("test_style");
  let bitflags = style.get_bitflags();
  debug!(style);
  debug!(bitflags);
  assert!(bitflags.contains(StyleFlag::BOLD_SET));
  assert!(bitflags.contains(StyleFlag::ITALIC_SET));
  assert_eq!(
    bitflags.contains(StyleFlag::UNDERLINE_SET),
    false
  );
}

#[test]
fn test_stylesheet() {
  let mut stylesheet = Stylesheet::new();
  let style1 = make_a_style("style1");
  let result = stylesheet.add_style(style1);
  assert!(result.is_ok());
  assert_eq!(stylesheet.styles.len(), 1);

  let style2 = make_a_style("style2");
  let result = stylesheet.add_style(style2);
  assert!(result.is_ok());
  assert_eq!(stylesheet.styles.len(), 2);

  assert_eq!(
    stylesheet
      .get_style_by_id("style1")
      .unwrap()
      .id,
    "style1"
  );

  assert_eq!(
    stylesheet
      .get_style_by_id("style2")
      .unwrap()
      .id,
    "style2"
  );

  assert!(stylesheet
    .get_style_by_id("style3")
    .is_none());
}

/// Helper function.
fn make_a_style(id: &str) -> Style {
  let black = Color::Rgb { r: 0, g: 0, b: 0 };
  let style = StyleBuilder::new()
    .set_id(id.to_string())
    .set_color_bg(Some(black))
    .set_color_fg(Some(black))
    .set_italic(true)
    .set_bold(true)
    .build();
  style
}
