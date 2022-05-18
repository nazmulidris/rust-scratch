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

use r3bl_rs_utils::with_mut;

#[macro_use]
extern crate bitmask;

bitmask! {
  pub mask StyleSetBitmask: u8 where flags Flags {
    FgColorSet = 1,
    BgColorSet = 2,
    BoldSet = 3,
    ItalicSet = 4,
    UnderlineSet = 5,
  }
}

#[test]
fn test_bitmask() {
  with_mut! {
    StyleSetBitmask::none(),
    as mask1,
    run {
      mask1.set(Flags::UnderlineSet);
      mask1.set(Flags::ItalicSet);
      assert!(mask1.contains(Flags::UnderlineSet));
      assert!(mask1.contains(Flags::ItalicSet));
    }
  };

  with_mut! {
    StyleSetBitmask::from(Flags::BoldSet | Flags::ItalicSet),
    as mask2,
    run {
      assert!(mask2.contains(Flags::BoldSet));
      assert!(mask2.contains(Flags::ItalicSet));
    }
  }

  assert_eq!(mask1.contains(mask2), false);
}
