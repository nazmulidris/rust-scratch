/*
 *   Copyright (c) 2024 Nazmul Idris
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

use super::constants::{FILENAME_TO_READ, FILE_TABLE_NAME};
use crossterm::style::Stylize;
use miette::IntoDiagnostic;
use r3bl_core::StringLength;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct FileEntry {
    id: String,
    name: String,
    content: Vec<u8>,
}

/// Create a SQLite database, a schema, write data to it, and read it back. The data is a
/// byte array (Vec<u8>) that contains the contents of a file. Read the byte array back
/// and convert it to a string.
pub fn run_db(connection: &Connection) -> miette::Result<()> {
    println!(
        "{}",
        "Running rw_files::run_db".magenta().bold().underlined()
    );

    // Create a the FILE_TABLE table, which has id: String, name: String, data: BLOB.
    connection
        .execute(
            (format!(
                "CREATE TABLE IF NOT EXISTS {FILE_TABLE_NAME} (
                  id              TEXT PRIMARY KEY,
                  name            TEXT NOT NULL,
                  data            BLOB
                  )",
            ))
            .as_str(),
            [],
        )
        .into_diagnostic()?;

    // Read the contents of the `Cargo.toml` file into a byte array (Vec<u8>).
    let cargo_toml_bytes = std::fs::read(FILENAME_TO_READ).into_diagnostic()?;

    // Write the contents of the `Cargo.toml` file into the FILE_TABLE table.
    connection
        .execute(
            "INSERT INTO file_table (id, name, data) VALUES (?1, ?2, ?3)",
            params![
                r3bl_core::generate_friendly_random_id(),
                FILENAME_TO_READ,
                cargo_toml_bytes
            ],
        )
        .into_diagnostic()?;

    // Read all the entries in the FILE_TABLE table.
    let mut prepared_statement = connection
        .prepare("SELECT id, name, data FROM file_table")
        .into_diagnostic()?;
    let result_set = prepared_statement
        .query_map([], |row| {
            Ok(FileEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                content: row.get(2)?,
            })
        })
        .into_diagnostic()?;
    for result in result_set {
        let file_entry = result.into_diagnostic()?;
        let id = file_entry.id;
        let name = file_entry.name;
        let content_as_string = String::from_utf8_lossy(file_entry.content.as_slice());
        let sha = StringLength::calculate_sha256(content_as_string.as_ref());
        println!(
            "{}, {}, \ncontent (sha): {}",
            format!("Found file: id: {id}").green().underlined(),
            name.to_string().grey(),
            format!("{sha}").dark_blue()
        );
    }

    Ok(())
}
