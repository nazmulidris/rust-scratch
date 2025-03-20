/*
 *   Copyright (c) 2025 Nazmul Idris
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

use super::ex_2::*;

pub struct XDrop<'a> {
    pub x: X,
    pub dropped_list: &'a mut Vec<X>,
}

impl Drop for XDrop<'_> {
    fn drop(&mut self) {
        println!("XDrop: Dropping {:?} & adding to dropped_list", self.x);
        self.dropped_list.push(self.x);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ex_4::x;

    #[test]
    fn test_x_drop() {
        let dropped_list = &mut vec![];

        {
            let x_val = XDrop {
                x: x(10),
                dropped_list,
            };
            println!("XDrop: Created {:?}", x_val.x);
        }

        assert_eq!(dropped_list.len(), 1);
        assert_eq!(dropped_list[0], x(10));

        {
            let x_val = XDrop {
                x: x(20),
                dropped_list,
            };
            println!("XDrop: Created {:?}", x_val.x);
        }

        assert_eq!(dropped_list.len(), 2);
        assert_eq!(dropped_list[0], x(10));
        assert_eq!(dropped_list[1], x(20));

        println!("XDrop: dropped_list: {:?}", dropped_list);
    }
}
