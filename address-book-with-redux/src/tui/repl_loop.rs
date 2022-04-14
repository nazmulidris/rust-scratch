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

// Imports.
use crate::{
  add_async_cmd_mw, address_book_reducer, air_cmd_mw, ip_cmd_mw, logger_mw, render_fn,
  Action, Mw, State, Std,
};
use r3bl_rs_utils::{
  print_header,
  redux::{
    async_middleware::SafeMiddlewareFnWrapper, async_subscriber::SafeSubscriberFnWrapper,
    sync_reducers::ShareableReducerFn, Store,
  },
  style_dimmed, style_error, style_primary,
  utils::readline_with_prompt,
};
use rand::random;
use std::error::Error;

#[tokio::main]
pub async fn run_tui_app(_args: Vec<String>) -> Result<(), Box<dyn Error>> {
  repl_loop(create_store().await).await?;
  Ok(())
}

async fn create_store() -> Store<State, Action> {
  let mut store = Store::<State, Action>::default();
  store
    .add_subscriber(SafeSubscriberFnWrapper::from(
      render_fn,
    ))
    .await
    .add_middleware(SafeMiddlewareFnWrapper::from(
      logger_mw,
    ))
    .await
    .add_middleware(SafeMiddlewareFnWrapper::from(
      air_cmd_mw,
    ))
    .await
    .add_middleware(SafeMiddlewareFnWrapper::from(
      ip_cmd_mw,
    ))
    .await
    .add_middleware(SafeMiddlewareFnWrapper::from(
      add_async_cmd_mw,
    ))
    .await
    .add_reducer(ShareableReducerFn::from(
      address_book_reducer,
    ))
    .await;
  store
}

const AVAIL_CMDS: &str =
  "quit, exit, add-async, add-sync, clear, remove, reset, search, history, ip, help";

pub async fn repl_loop(store: Store<State, Action>) -> Result<(), Box<dyn Error>> {
  print_header("Starting repl");

  // Repl.
  loop {
    let user_input = readline_with_prompt("r3bl> ")?;
    match user_input.as_str() {
      "help" => {
        println!(
          "{}: {}",
          style_primary("Available commands"),
          style_dimmed(AVAIL_CMDS)
        );
      }
      "quit" => break,
      "exit" => break,
      "add-sync" => {
        let id = random::<u8>();
        let action = Action::Std(Std::AddContact(
          format!("John Doe #{}", id),
          format!("jd@gmail.com #{}", id),
          format!("123-456-7890 #{}", id),
        ));
        store.dispatch(action).await;
      }
      "clear" => {
        let action = Action::Std(Std::RemoveAllContacts);
        store.dispatch(action).await;
      }
      "remove" => {
        match readline_with_prompt("id> ") {
          Ok(id) => {
            let action = Action::Std(Std::RemoveContactById(
              id.parse().unwrap(),
            ));
            store.dispatch(action).await
          }
          Err(_) => println!("{}", style_error("Invalid id")),
        };
      }
      "search" => {
        match readline_with_prompt("search_term> ") {
          Ok(search_term) => {
            let action = Action::Std(Std::Search(search_term));
            store.dispatch(action).await
          }
          Err(_) => println!("{}", style_error("Invalid id")),
        };
      }
      "reset" => {
        let action = Action::Std(Std::ResetState(State::default()));
        store.dispatch(action).await;
      }
      "history" => {
        println!("{:#?}", store.get_history().await);
      }
      "add-async" => {
        let action = Action::Mw(Mw::AsyncAddCmd);
        store.dispatch(action).await;
        println!(
          "{}",
          "🧵 Spawning exec_add_async_cmd ..."
        );
      }
      "ip" => {
        let action = Action::Mw(Mw::AsyncIpCmd);
        store.dispatch(action).await;
        println!("{}", "🧵 Spawning get_ip_api()...");
      }
      "air" => {
        let action = Action::Mw(Mw::AsyncAirCmd);
        store.dispatch(action).await;
        println!(
          "{}",
          "🧵 Spawning awair_local_api()..."
        );
      }
      // Catchall.
      _ => {
        println!(
          "{}",
          style_error("Unknown command")
        );
      }
    }; // end match user_input.

    // Print confirmation at the end of 1 repl loop.
    println!(
      "{} {}",
      style_primary(&user_input),
      style_dimmed("was executed.")
    );
  }

  Ok(())
}
