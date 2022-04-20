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

use crate::{Action, Mw, State};
use async_trait::async_trait;
use r3bl_rs_utils::{
  fire_and_forget, print_header,
  redux::{AsyncMiddleware, StoreStateMachine},
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct SaveCmdMw;

#[async_trait]
impl AsyncMiddleware<State, Action> for SaveCmdMw {
  async fn run(
    &self,
    action: Action,
    _store_ref: Arc<RwLock<StoreStateMachine<State, Action>>>,
  ) {
    if let Action::Mw(Mw::SaveCmd) = action {
      fire_and_forget![{
        // TODO: impl this!
        print_header("╭──────────────────────────────────────────────────────╮");
        print_header("│ SaveCmdMw coming soon!                               │");
        print_header("╰──────────────────────────────────────────────────────╯");
      }];
    }
  }
}
