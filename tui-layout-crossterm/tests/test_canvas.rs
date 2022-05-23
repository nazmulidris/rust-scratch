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
use r3bl_rs_utils::CommonResult;
use tui_layout_crossterm::layout::*;

#[test]
fn test_simple_2_col_layout() -> CommonResult<()> {
  let mut canvas = Canvas::default();
  canvas.stylesheet = create_stylesheet()?;
  canvas.start(
    Position::from_pair(Pair::new(0, 0)),
    Size::from_pair(Pair::new(500, 500)),
  )?;
  layout_container(&mut canvas)?;
  canvas.end()?;
  Ok(())
}

/// Create stylesheet.
fn create_stylesheet() -> CommonResult<Stylesheet> {
  let mut stylesheet = Stylesheet::new();
  stylesheet.add_styles(vec![
    create_style("style1"),
    create_style("style2"),
  ])?;
  Ok(stylesheet)
}

/// Helper function.
fn create_style(id: &str) -> Style {
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

/// Main container.
fn layout_container(canvas: &mut Canvas) -> CommonResult<()> {
  canvas.start_layout(
    "container",
    Direction::Horizontal,
    RequestedSizePercent::parse_pair(Pair::new(100, 100))?,
  )?;
  make_container_assertions(canvas)?;
  layout_left_col(canvas)?;
  layout_right_col(canvas)?;
  canvas.end_layout()?;
  return Ok(());

  fn make_container_assertions(canvas: &Canvas) -> CommonResult<()> {
    let layout_item = canvas
      .layout_stack
      .first()
      .unwrap();

    assert_eq!(layout_item.id, "container");
    assert_eq!(
      layout_item.dir,
      Direction::Horizontal
    );
    assert_eq!(
      layout_item.origin_pos,
      Some(Position::new(0, 0))
    );
    assert_eq!(
      layout_item.bounds_size,
      Some(Size::new(500, 500))
    );
    assert_eq!(
      layout_item.req_size_percent,
      Some(RequestedSizePercent::parse_pair(
        Pair::new(100, 100)
      )?)
    );
    assert_eq!(
      layout_item.layout_cursor_pos,
      Some(Position::new(0, 0))
    );
    assert_eq!(
      layout_item.content_cursor_pos,
      None
    );

    Ok(())
  }
}

/// Left column.
fn layout_left_col(canvas: &mut Canvas) -> CommonResult<()> {
  canvas.start_layout(
    "col_1",
    Direction::Vertical,
    RequestedSizePercent::parse_pair(Pair::new(50, 100))?,
  )?;
  canvas.print(vec!["col 1 - Hello"])?;
  canvas.print(vec!["col 1 - World"])?;
  make_left_col_assertions(canvas)?;
  canvas.end_layout()?;
  return Ok(());

  fn make_left_col_assertions(canvas: &Canvas) -> CommonResult<()> {
    let layout_item = canvas.layout_stack.last().unwrap();
    assert_eq!(layout_item.id, "col_1");
    assert_eq!(
      layout_item.dir,
      Direction::Vertical
    );
    assert_eq!(
      layout_item.origin_pos,
      Some(Position::new(0, 0))
    );
    assert_eq!(
      layout_item.bounds_size,
      Some(Size::new(250, 500))
    );
    assert_eq!(
      layout_item.req_size_percent,
      Some(RequestedSizePercent::parse_pair(
        Pair::new(50, 100)
      )?)
    );
    assert_eq!(layout_item.layout_cursor_pos, None);
    assert_eq!(
      layout_item.content_cursor_pos,
      Some(Position::new(0, 2))
    );
    Ok(())
  }
}

/// Right column.
fn layout_right_col(canvas: &mut Canvas) -> CommonResult<()> {
  canvas.start_layout(
    "col_2",
    Direction::Vertical,
    RequestedSizePercent::parse_pair(Pair::new(50, 100))?,
  )?;
  canvas.print(vec!["col 2 - Hello"])?;
  canvas.print(vec!["col 2 - World"])?;
  make_right_col_assertions(canvas)?;
  canvas.end_layout()?;
  return Ok(());

  fn make_right_col_assertions(canvas: &Canvas) -> CommonResult<()> {
    let layout_item = canvas.layout_stack.last().unwrap();
    assert_eq!(layout_item.id, "col_2");
    assert_eq!(
      layout_item.dir,
      Direction::Vertical
    );
    assert_eq!(
      layout_item.origin_pos,
      Some(Position::new(250, 0))
    );
    assert_eq!(
      layout_item.bounds_size,
      Some(Size::new(250, 500))
    );
    assert_eq!(
      layout_item.req_size_percent,
      Some(RequestedSizePercent::parse_pair(
        Pair::new(50, 100)
      )?)
    );
    assert_eq!(layout_item.layout_cursor_pos, None);
    assert_eq!(
      layout_item.content_cursor_pos,
      Some(Position::new(0, 2))
    );
    Ok(())
  }
}
