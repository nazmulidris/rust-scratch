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

use super::State;

/// Action.
/// https://stackoverflow.com/questions/55032236/compare-nested-enum-variants-in-rust
#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Action {
  Mw(Mw),
  Std(Std),
  Noop, /* For Default impl. */
}

impl Default for Action {
  fn default() -> Self {
    Action::Noop
  }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Std {
  AddContact(String, String, String),
  RemoveAllContacts,
  RemoveContactById(usize),
  ResetState(State),
  Search(String),
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Mw {
  AsyncAddCmd,
  AsyncAirCmd,
  AsyncIpCmd,
}
