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

use miette::IntoDiagnostic as _;
use rusqlite::Connection;
use sql::rusqlite_ex::{rw_files, rw_structs_and_strings};

fn main() -> miette::Result<()> {
    // Connect to SQLite database.
    let db_connection =
        Connection::open(sql::rusqlite_ex::constants::SQLITE_FILE).into_diagnostic()?;
    rw_structs_and_strings::run_db(&db_connection)?;
    rw_files::run_db(&db_connection)?;
    Ok(())
}
