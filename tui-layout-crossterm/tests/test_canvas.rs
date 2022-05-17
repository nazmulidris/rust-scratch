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

// TODO: write assertions for this test
#[test]
fn test_simple_2_col_layout() -> ResultCommon<()> {
  let mut canvas = Canvas::default();
  canvas.start(
    Position::new(0, 0),
    Size::new(500, 500),
  )?;
  main_container_layout(&mut canvas)?;
  canvas.end()?;
  Ok(())
}

fn main_container_layout(canvas: &mut Canvas) -> ResultCommon<()> {
  canvas.start_layout(
    "container",
    Direction::Horizontal,
    RequestedSize::from(100, 100)?,
  )?;
  col_1_layout(canvas)?;
  col_2_layout(canvas)?;
  canvas.end_layout()?;
  Ok(())
}

fn col_1_layout(canvas: &mut Canvas) -> ResultCommon<()> {
  // start layout (left column)
  canvas.start_layout(
    "col_1",
    Direction::Vertical,
    RequestedSize::from(50, 100)?,
  )?;
  canvas.print(vec!["col 1 - Hello"])?;
  canvas.print(vec!["col 1 - World"])?;
  debug!(canvas);
  println!("🏳️‍🌈🏳️‍🌈🏳️‍🌈");
  canvas.end_layout()?;
  Ok(())
}

fn col_2_layout(canvas: &mut Canvas) -> ResultCommon<()> {
  // start layout (right column)
  canvas.start_layout(
    "col_2",
    Direction::Vertical,
    RequestedSize::from(50, 100)?,
  )?;
  canvas.print(vec!["col 2 - Hello"])?;
  canvas.print(vec!["col 2 - World"])?;
  debug!(canvas);
  println!("🏳️‍🌈🏳️‍🌈🏳️‍🌈");
  canvas.end_layout()?;
  Ok(())
}
