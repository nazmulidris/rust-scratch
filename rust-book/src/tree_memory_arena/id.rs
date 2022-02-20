/*
 Copyright 2022 Nazmul Idris

 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at

      https://www.apache.org/licenses/LICENSE-2.0

 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
*/

//! UID for Node.

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct Uid(usize);

impl Uid {
  pub fn new(index: usize) -> Uid {
    Uid(index)
  }
}

pub trait HasId {
  fn get_id(&self) -> usize;
  fn get_copy_of_id(&self) -> Uid;
}

impl HasId for Uid {
  fn get_id(&self) -> usize {
    self.0
  }
  fn get_copy_of_id(&self) -> Uid {
    Uid(self.0)
  }
}
