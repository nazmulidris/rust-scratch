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
use r3bl_cmdr::{Style, StyleBuilder, StyleFlag, Stylesheet};

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
      assert!(!mask1.contains(StyleFlag::MARGIN_SET));
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
      assert!(!mask2.contains(StyleFlag::MARGIN_SET));
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
  assert_eq!(bitflags.contains(StyleFlag::UNDERLINE_SET), false);
}

#[test]
fn test_cascade_style() {
  let style_bold_green_fg = StyleBuilder::new()
    .set_bold(true)
    .set_color_fg(Some(Color::Green))
    .build();
  let style_italic = StyleBuilder::new().set_italic(true).build();
  let style_yellow_bg = StyleBuilder::new()
    .set_color_bg(Some(Color::Yellow))
    .build();
  let style_margin = StyleBuilder::new().set_margin(Some(2)).build();
  let style_red_fg = StyleBuilder::new().set_color_fg(Some(Color::Red)).build();

  let mut computed_style =
    style_bold_green_fg + style_italic + style_yellow_bg + style_margin + style_red_fg;

  assert!(computed_style.get_bitflags().contains(
    StyleFlag::COLOR_FG_SET
      | StyleFlag::COLOR_BG_SET
      | StyleFlag::BOLD_SET
      | StyleFlag::ITALIC_SET
      | StyleFlag::MARGIN_SET
      | StyleFlag::COMPUTED_SET
  ));

  assert_eq!(computed_style.color_bg.unwrap(), Color::Yellow);
  assert_eq!(computed_style.color_fg.unwrap(), Color::Red);
  assert_eq!(computed_style.bold, true);
  assert_eq!(computed_style.italic, true);
  assert_eq!(computed_style.computed, true);
  assert_eq!(computed_style.margin.unwrap(), 2);
  assert_eq!(computed_style.underline, false);
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

  assert_eq!(stylesheet.find_style_by_id("style1").unwrap().id, "style1");
  assert_eq!(stylesheet.find_style_by_id("style2").unwrap().id, "style2");
  assert!(stylesheet.find_style_by_id("style3").is_none());

  let result = stylesheet.find_styles_by_ids(vec!["style1", "style2"]);
  assert_eq!(result.as_ref().unwrap().len(), 2);
  assert_eq!(result.as_ref().unwrap()[0].id, "style1");
  assert_eq!(result.as_ref().unwrap()[1].id, "style2");
  assert_eq!(
    stylesheet.find_styles_by_ids(vec!["style3", "style4"]),
    None
  );
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
