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

use r3bl_rs_utils::{debug, ResultCommon};
use tui_layout_crossterm::layout::*;

#[test]
fn test_simple_2_col_layout() -> ResultCommon<()> {
  let mut canvas = Canvas::default();
  canvas.start(
    Position::from_pair(Pair::new(0, 0)),
    Size::from_pair(Pair::new(500, 500)),
  )?;
  layout_container(&mut canvas)?;
  canvas.end()?;
  Ok(())
}

fn layout_container(canvas: &mut Canvas) -> ResultCommon<()> {
  canvas.start_layout(
    "container",
    Direction::Horizontal,
    RequestedSizePercent::parse_pair(Pair::new(100, 100))?,
  )?;

  make_container_assertions(canvas)?;

  layout_col_1(canvas)?;
  layout_col_2(canvas)?;

  canvas.end_layout()?;
  return Ok(());

  fn make_container_assertions(canvas: &Canvas) -> ResultCommon<()> {
    println!("🟢");
    debug!(canvas);

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

// TODO: write assertions for this test
/// Left column.
fn layout_col_1(canvas: &mut Canvas) -> ResultCommon<()> {
  canvas.start_layout(
    "col_1",
    Direction::Vertical,
    RequestedSizePercent::parse_pair(Pair::new(50, 100))?,
  )?;
  canvas.print(vec!["col 1 - Hello"])?;
  canvas.print(vec!["col 1 - World"])?;

  println!("🟢🟢");
  debug!(canvas);

  canvas.end_layout()?;
  Ok(())
}

// TODO: write assertions for this test
/// Right column.
fn layout_col_2(canvas: &mut Canvas) -> ResultCommon<()> {
  canvas.start_layout(
    "col_2",
    Direction::Vertical,
    RequestedSizePercent::parse_pair(Pair::new(50, 100))?,
  )?;
  canvas.print(vec!["col 2 - Hello"])?;
  canvas.print(vec!["col 2 - World"])?;

  println!("🟢🟢🟢");
  debug!(canvas);

  canvas.end_layout()?;
  Ok(())
}
