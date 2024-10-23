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

use crossterm::style::Stylize;
use miette::IntoDiagnostic;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::constants::{DATA_TABLE_NAME, SQLITE_FILE};

/// Create a SQLite database, a schema, write data to it, and read it back. The data is a
/// struct containing some JSON data. Parse the raw JSON string into a JSON object.
pub fn run_db() -> miette::Result<()> {
    #[derive(Serialize, Deserialize, Debug)]
    struct Record {
        id: String,
        name: String,
        raw_json_data: String,
    }

    // Connect to SQLite database.
    let db_connection = Connection::open(SQLITE_FILE).into_diagnostic()?;

    // Create table.
    db_connection
        .execute(
            (format!(
                "CREATE TABLE IF NOT EXISTS {DATA_TABLE_NAME} (
                  id              TEXT PRIMARY KEY,
                  name            TEXT NOT NULL,
                  data            TEXT
                  )",
            ))
            .as_str(),
            [],
        )
        .into_diagnostic()?;

    // Create a JSON string from a struct.
    let record = Record {
        id: r3bl_core::generate_friendly_random_id(),
        name: petname::petname(2, " ").unwrap_or("John Doe".into()),
        raw_json_data: r#"{"key":"value"}"#.into(),
    };

    // Insert person into table.
    db_connection
        .execute(
            format!("INSERT INTO {DATA_TABLE_NAME} (id, name, data) VALUES (?1, ?2, ?3)").as_str(),
            params![record.id, record.name, record.raw_json_data],
        )
        .into_diagnostic()?;

    // Read data from the table.
    let mut prepared_statement = db_connection
        .prepare(format!("SELECT id, name, data FROM {DATA_TABLE_NAME}").as_str())
        .into_diagnostic()?;
    let result_set = prepared_statement
        .query_map([], |row| {
            Ok(Record {
                id: row.get(0)?,
                name: row.get(1)?,
                raw_json_data: row.get(2)?,
            })
        })
        .into_diagnostic()?;
    for result in result_set {
        let person = result.into_diagnostic()?;
        // Convert record.raw_json_data to a JSON object.
        let json_data: serde_json::Value =
            serde_json::from_str(&person.raw_json_data).into_diagnostic()?;
        let id = person.id;
        let name = person.name;
        let json_data = format!("{:#?}", json_data).blue();
        println!("Found person: id: {id}, name: {name}, data: \n{json_data}");
    }

    Ok(())
}
