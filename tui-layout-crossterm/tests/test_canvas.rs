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

use crossterm::style::Color;
use r3bl_rs_utils::*;
use r3bl_cmdr::layout::*;

#[test]
fn test_simple_2_col_layout() -> CommonResult<()> {
  throws!({
    let mut canvas = Canvas::default();
    canvas.stylesheet = create_stylesheet()?;
    canvas.canvas_start(
      CanvasPropsBuilder::new()
        .set_pos((0, 0).into())
        .set_size((500, 500).into())
        .build(),
    )?;
    layout_container(&mut canvas)?;
    canvas.canvas_end()?;
  });
}

/// Main container "container".
fn layout_container(canvas: &mut Canvas) -> CommonResult<()> {
  throws!({
    canvas.layout_start(
      LayoutPropsBuilder::new()
        .set_id("container".to_string())
        .set_dir(Direction::Horizontal)
        .set_req_size((100, 100).try_into()?)
        .build(),
    )?;
    make_container_assertions(canvas)?;
    layout_left_col(canvas)?;
    layout_right_col(canvas)?;
    canvas.layout_end()?;
  });

  fn make_container_assertions(canvas: &Canvas) -> CommonResult<()> {
    throws!({
      let layout_item = canvas.layout_stack.first().unwrap();
      assert_eq!(layout_item.id, "container");
      assert_eq!(layout_item.dir, Direction::Horizontal);
      assert_eq!(layout_item.origin_pos, Some((0, 0).into()));
      assert_eq!(layout_item.bounds_size, Some((500, 500).into()));
      assert_eq!(layout_item.req_size_percent, Some((100, 100).try_into()?));
      assert_eq!(layout_item.layout_cursor_pos, Some((0, 0).into()));
      assert_eq!(layout_item.content_cursor_pos, None);
      assert_eq!(layout_item.styles, None);
    });
  }
}

/// Left column "col_1".
fn layout_left_col(canvas: &mut Canvas) -> CommonResult<()> {
  throws!({
    canvas.layout_start(
      LayoutPropsBuilder::new()
        .set_styles(canvas.stylesheet.find_styles_by_ids(vec!["style1"]))
        .set_id("col_1".to_string())
        .set_dir(Direction::Vertical)
        .set_req_size((50, 100).try_into()?)
        .build(),
    )?;
    canvas.paint(vec!["col 1 - Hello"])?;
    canvas.paint(vec!["col 1 - World"])?;
    make_left_col_assertions(canvas)?;
    canvas.layout_end()?;
  });

  fn make_left_col_assertions(canvas: &Canvas) -> CommonResult<()> {
    throws!({
      let layout_item = canvas.layout_stack.last().unwrap();
      assert_eq!(layout_item.id, "col_1");
      assert_eq!(layout_item.dir, Direction::Vertical);
      assert_eq!(layout_item.origin_pos, Some((2, 2).into())); // Take margin into account.
      assert_eq!(layout_item.bounds_size, Some((246, 496).into())); // Take margin into account.
      assert_eq!(layout_item.req_size_percent, Some((50, 100).try_into()?));
      assert_eq!(layout_item.layout_cursor_pos, None);
      assert_eq!(layout_item.content_cursor_pos, Some((0, 2).into()));
      assert_eq!(
        layout_item.styles.clone(),
        Stylesheet::compute(canvas.stylesheet.find_styles_by_ids(vec!["style1"]))
      );
    });
  }
}

/// Right column "col_2".
fn layout_right_col(canvas: &mut Canvas) -> CommonResult<()> {
  throws!({
    canvas.layout_start(
      LayoutPropsBuilder::new()
        .set_styles(canvas.stylesheet.find_styles_by_ids(vec!["style2"]))
        .set_id("col_2".to_string())
        .set_dir(Direction::Vertical)
        .set_req_size((50, 100).try_into()?)
        .build(),
    )?;
    canvas.paint(vec!["col 2 - Hello"])?;
    canvas.paint(vec!["col 2 - World"])?;
    make_right_col_assertions(canvas)?;
    canvas.layout_end()?;
  });

  fn make_right_col_assertions(canvas: &Canvas) -> CommonResult<()> {
    throws!({
      let layout_item = canvas.layout_stack.last().unwrap();
      assert_eq!(layout_item.id, "col_2");
      assert_eq!(layout_item.dir, Direction::Vertical);
      assert_eq!(layout_item.origin_pos, Some((252, 2).into())); // Take margin into account.
      assert_eq!(layout_item.bounds_size, Some((246, 496).into())); // Take margin into account.
      assert_eq!(layout_item.req_size_percent, Some((50, 100).try_into()?));
      assert_eq!(layout_item.layout_cursor_pos, None);
      assert_eq!(layout_item.content_cursor_pos, Some((0, 2).into()));
      assert_eq!(
        layout_item.styles.clone(),
        Stylesheet::compute(canvas.stylesheet.find_styles_by_ids(vec!["style2"]))
      );
    });
  }
}

/// Create a stylesheet containing styles.
fn create_stylesheet() -> CommonResult<Stylesheet> {
  let mut stylesheet = Stylesheet::new();
  stylesheet.add_styles(vec![create_style("style1"), create_style("style2")])?;
  Ok(stylesheet)
}

/// Create a style.
fn create_style(id: &str) -> Style {
  let black = Color::Rgb { r: 0, g: 0, b: 0 };
  let style = StyleBuilder::new()
    .set_id(id.to_string())
    .set_color_bg(Some(black))
    .set_color_fg(Some(black))
    .set_italic(true)
    .set_bold(true)
    .set_margin(Some(2))
    .build();
  style
}
