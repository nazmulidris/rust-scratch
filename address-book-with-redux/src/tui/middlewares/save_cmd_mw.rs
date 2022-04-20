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

use crate::{Action, Mw, State, PROMPT_STR, STATE_JSON_FNAME};
use async_trait::async_trait;
use r3bl_rs_utils::{
  print_header, redux::AsyncMiddleware, style_error, style_primary, utils::print_prompt,
};
use std::io::Result;
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Default)]
pub struct SaveCmdMw;

/// https://docs.serde.rs/serde_json/#creating-json-by-serializing-data-structures
/// https://docs.rs/tokio/latest/tokio/fs/struct.File.html
#[async_trait]
impl AsyncMiddleware<State, Action> for SaveCmdMw {
  async fn run(
    &self,
    action: Action,
    state: State,
  ) -> Option<Action> {
    if let Action::Mw(Mw::SaveCmd) = action {
      println!();
      print_header("╭──────────────────────────────────────────────────────╮");
      print_header("│ SaveCmdMw: save to `state.json`                      │");
      print_header("╰──────────────────────────────────────────────────────╯");
      print_prompt(PROMPT_STR).unwrap();
      return do_save(&state).await;
    }
    None
  }
}

pub async fn do_save(state: &State) -> Option<Action> {
  let result = save_state_to_file(&state, STATE_JSON_FNAME).await;
  match result {
    Err(error) => {
      println!(
        "Could not save state to: `{}` due to: {}",
        style_primary(STATE_JSON_FNAME),
        style_error(&format!("{:#?}", error))
      );
    }
    _ => {}
  }
  None
}

/// Produces error if:
/// 1. Can't open the `fname` file for writing.
/// 2. If something goes wrong when writing the bytes to the file.
/// 3. If `state` can't be serialized to pretty JSON string.
async fn save_state_to_file(
  state: &State,
  fname: &str,
) -> Result<()> {
  let mut file = File::create(fname).await?;
  let json = serde_json::to_string_pretty(&state)?;
  file
    .write_all(json.as_bytes())
    .await?;
  Ok(())
}
