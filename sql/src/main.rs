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

use miette::IntoDiagnostic;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Record {
    id: String,
    name: String,
    raw_json_data: String,
}

const DB_FILE: &str = "test.db";

fn main() -> miette::Result<()> {
    write_and_read_data()
}

/// Create a SQLite database, a schema, write data to it, and read it back.
fn write_and_read_data() -> miette::Result<()> {
    // Connect to SQLite database.
    let conn = Connection::open(DB_FILE).into_diagnostic()?;

    // Create table.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS person (
                  id              TEXT PRIMARY KEY,
                  name            TEXT NOT NULL,
                  data            TEXT
                  )",
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
    conn.execute(
        "INSERT INTO person (id, name, data) VALUES (?1, ?2, ?3)",
        params![record.id, record.name, record.raw_json_data],
    )
    .into_diagnostic()?;

    // Read data from the table.
    let mut stmt = conn
        .prepare("SELECT id, name, data FROM person")
        .into_diagnostic()?;
    let result_set = stmt
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
        println!("Found person: id: {id}, name: {name}, data: {json_data:#?}");
    }

    Ok(())
}
