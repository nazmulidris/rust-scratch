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

use r3bl_cmdr::*;

#[tokio::test]
async fn test_add_box_size_to_pos() {
  // [10, 10] + [30, 10] = [40, 20]
  let pos = Position::from((10, 10));
  let size = Size::from((30, 10));
  let new_pos = pos + size; // `size + pos` is not defined.
  assert_eq!(new_pos.x, 40);
  assert_eq!(new_pos.y, 20);
}

#[tokio::test]
async fn test_mul_box_pos_to_pair() {
  // [30, 10] * [1, 0] = [30, 0]
  {
    let pos: Position = (30, 10).into();
    let pair_cancel_y = (1, 0).into();
    let new_pair = pos * pair_cancel_y;
    assert_eq!(new_pair.x, 30);
    assert_eq!(new_pair.y, 0);
  }

  // [30, 10] * [0, 1] = [0, 10]
  {
    let pos: Position = (30, 10).into();
    let pair_cancel_x = (0, 1).into();
    let new_pair = pos * pair_cancel_x;
    assert_eq!(new_pair.x, 0);
    assert_eq!(new_pair.y, 10);
  }
}

#[test]
fn test_percent_works_as_expected() {
  let pc_100 = Percent::try_from(100i32).unwrap();
  assert_eq!(pc_100.value, 100);
  let result = calc_percentage(pc_100, 500);
  assert_eq!(result, 500);

  let pc_50 = Percent::try_from(50i32).unwrap();
  assert_eq!(pc_50.value, 50);
  let result = calc_percentage(pc_50, 500);
  assert_eq!(result, 250);

  let pc_0 = Percent::try_from(0i32).unwrap();
  assert_eq!(pc_0.value, 0);
  let result = calc_percentage(pc_0, 500);
  assert_eq!(result, 0);
}

#[test]
fn test_percent_parsing_fails_as_expected() {
  assert!(Percent::try_from(-1i32).is_err());

  assert!(Percent::try_from(0i32).is_ok());
  assert!(Percent::try_from(0u16).is_ok());

  assert!(Percent::try_from(100i32).is_ok());
  assert!(Percent::try_from(100u16).is_ok());

  assert!(Percent::try_from(101i32).is_err());
  assert!(Percent::try_from(101u16).is_err());
}
