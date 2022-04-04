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
use std::error::Error;
use rand::random;
use r3bl_rs_utils::redux::{
  Store, async_subscriber::SafeSubscriberFnWrapper, sync_reducers::ReducerFnWrapper,
  async_middleware::SafeMiddlewareFnWrapper,
};
use r3bl_rs_utils::{
  utils::{print_header, style_error, style_primary, style_dimmed, readline_with_prompt},
};
use crate::address_book::{address_book_reducer, Action, State};
use super::{render_fn, logger_mw};

#[tokio::main]
pub async fn run_tui_app(_args: Vec<String>) -> Result<(), Box<dyn Error>> {
  repl_loop(create_store().await).await?;
  Ok(())
}

async fn create_store() -> Store<State, Action> {
  let mut store = Store::<State, Action>::new();
  store
    .add_subscriber(SafeSubscriberFnWrapper::from(render_fn))
    .await
    .add_middleware(SafeMiddlewareFnWrapper::from(logger_mw))
    .await
    .add_reducer(ReducerFnWrapper::from(address_book_reducer))
    .await;
  store
}

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
          style_dimmed("quit, exit, add-async, add-sync, clear, remove, reset, search, history, help")
        );
      }
      "quit" => break,
      "exit" => break,
      "add-sync" => {
        let id = random::<u8>();
        store
          .dispatch(&Action::AddContact(
            format!("John Doe #{}", id),
            format!("jd@gmail.com #{}", id),
            format!("123-456-7890 #{}", id),
          ))
          .await;
      }
      "add-async" => {
        let id = random::<u8>();
        store
          .dispatch_spawn(Action::AddContact(
            format!("John Doe #{}", id),
            format!("jd@gmail.com #{}", id),
            format!("123-456-7890 #{}", id),
          ))
          .await;
      }
      "clear" => {
        store.dispatch(&Action::RemoveAllContacts).await;
      }
      "remove" => {
        match readline_with_prompt("id> ") {
          Ok(id) => {
            store
              .dispatch(&Action::RemoveContactById(id.parse().unwrap()))
              .await
          }
          Err(_) => println!("{}", style_error("Invalid id")),
        };
      }
      "search" => {
        match readline_with_prompt("search_term> ") {
          Ok(search_term) => store.dispatch(&Action::Search(search_term)).await,
          Err(_) => println!("{}", style_error("Invalid id")),
        };
      }
      "reset" => {
        store.dispatch(&Action::ResetState(State::default())).await;
      }
      "history" => {
        println!("{:#?}", store.get_history().await);
      }
      // Catchall.
      _ => {
        println!("{}", style_error("Unknown command"));
      }
    } // end match user_input.

    // Print confirmation at the end of 1 repl loop.
    println!(
      "{} {}",
      style_primary(&user_input),
      style_dimmed("was executed.")
    );
  }

  Ok(())
}
