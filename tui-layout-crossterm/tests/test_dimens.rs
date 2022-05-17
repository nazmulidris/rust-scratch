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

use tui_layout_crossterm::*;

#[tokio::test]
async fn test_add_box_size_to_pos() {
  // [10, 10] + [30, 10] = [40, 20]
  let pos = Position::new(10, 10);
  let size = Size::new(30, 10);
  let new_pos = pos + size; // `size + pos` is not defined.
  assert_eq!(new_pos.x, 40);
  assert_eq!(new_pos.y, 20);
}

#[tokio::test]
async fn test_mul_box_pos_to_pair() {
  // [30, 10] * [1, 0] = [30, 0]
  {
    let pos = Position::new(30, 10);
    let pair_cancel_y = Pair::new(1, 0);
    let new_pair = pos * pair_cancel_y;
    assert_eq!(new_pair.x, 30);
    assert_eq!(new_pair.y, 0);
  }

  // [30, 10] * [0, 1] = [0, 10]
  {
    let pos = Position::new(30, 10);
    let pair_cancel_x = Pair::new(0, 1);
    let new_pair = pos * pair_cancel_x;
    assert_eq!(new_pair.x, 0);
    assert_eq!(new_pair.y, 10);
  }
}

#[test]
fn test_percent_works_as_expected() {
  let pc_100 = PerCent::from(100).unwrap();
  assert_eq!(pc_100.value, 100);
  let result = calc_percentage(pc_100, 500);
  assert_eq!(result, 500);

  let pc_50 = PerCent::from(50).unwrap();
  assert_eq!(pc_50.value, 50);
  let result = calc_percentage(pc_50, 500);
  assert_eq!(result, 250);

  let pc_0 = PerCent::from(0).unwrap();
  assert_eq!(pc_0.value, 0);
  let result = calc_percentage(pc_0, 500);
  assert_eq!(result, 0);
}

#[test]
fn test_percent_fails_as_expected() {
  fn assert_is_none(raw_num: i32) {
    assert!(PerCent::from(raw_num).is_none());
  }
  assert_is_none(101);
}
