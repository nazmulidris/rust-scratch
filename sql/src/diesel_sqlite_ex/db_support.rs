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

use diesel::prelude::*;
use diesel::SqliteConnection;
use miette::{IntoDiagnostic, Result};

/// Specify your database URL, eg:
/// - `path/to/your/file.db` - Save the database file in the given path.
/// - `file://file.db` - Save the database file in given path.
/// - `:memory:` - Create an in-memory database.
///
/// See [SqliteConnection] for more details.
pub fn create_connection(database_url: &str) -> Result<SqliteConnection> {
    SqliteConnection::establish(database_url).into_diagnostic()
}

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Run the migrations automatically when the program starts.
/// More info:
/// - <https://docs.rs/diesel_migrations/latest/diesel_migrations/>
/// - <https://docs.rs/diesel_migrations/latest/diesel_migrations/macro.embed_migrations.html>
pub fn try_run_migrations(
    connection: &mut SqliteConnection,
) -> std::result::Result<
    Vec<diesel::migration::MigrationVersion<'_>>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    connection.run_pending_migrations(MIGRATIONS)
}

