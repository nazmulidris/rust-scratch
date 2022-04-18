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

use r3bl_rs_utils::tree_memory_arena::HasId;
use serde::{Deserialize, Serialize};

// State.
#[derive(Clone, Default, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub struct State {
  pub address_book: Vec<Contact>,
  pub search_term: Option<String>,
}

// Contact.
#[derive(Clone, Default, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub struct Contact {
  pub id: usize,
  pub name: String,
  pub email: String,
  pub phone: String,
}

impl HasId for Contact {
  type IdType = usize;

  /// Delegate this to `self.id`, which is type `usize`.
  fn get_id(&self) -> usize {
    self.id.get_id()
  }

  /// Delegate this to `self.id`, which is type `usize`.
  fn into_some(&self) -> Option<usize> {
    self.id.into_some()
  }
}
