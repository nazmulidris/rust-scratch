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

use crossterm::style::Stylize as _;
use diesel::prelude::*;
use diesel::SqliteConnection;
use miette::{IntoDiagnostic, Result};

pub mod general_ops {
    use super::*;

    /// Specify your database URL, eg:
    /// - `path/to/your/file.db`
    /// - `file://file.db`
    /// - `:memory:`
    ///
    /// See [SqliteConnection] for more details.
    pub fn create_connection(database_url: &str) -> Result<SqliteConnection> {
        SqliteConnection::establish(database_url).into_diagnostic()
    }
}

pub mod data_table_ops {
    use super::*;
    use crate::diesel_sqlite_ex::{
        data_table,      /* from schema */
        DataTableRecord, /* from models */
    };

    /// # Get the timestamp for current time in UTC
    /// - [chrono::Utc::now()] returns a [chrono::DateTime::naive_utc()] which is a [chrono::NaiveDateTime].
    ///
    /// # Convert the timestamp in the database to a [chrono::DateTime]
    /// - Use [chrono::DateTime::from_naive_utc_and_offset] with the following args:
    ///   1. [chrono::NaiveDateTime] from the previous step.
    ///   2. `0` offset for the timezone.
    pub fn insert_a_few_records(connection: &mut SqliteConnection) -> Result<()> {
        let timestamp = chrono::Utc::now().naive_utc();
        let new_record = DataTableRecord {
            id: r3bl_core::generate_friendly_random_id().into(),
            name: petname::petname(2, " ").unwrap_or("John Doe".into()).into(),
            data: r#"{"key":"value"}"#.into(),
            created_at: timestamp,
        };

        let inserted_record = diesel::insert_into(data_table::table)
            .values(&new_record) // Insert this struct into the table.
            .returning(DataTableRecord::as_returning()) // Return inserted record as this struct.
            .get_result(connection) // Add 'RETURNING *' to the query, and execute it.
            .into_diagnostic()?;

        println!(
            "{} ⴾ {}, {}, {}, {}",
            "Inserted record".magenta(),
            inserted_record.id,
            inserted_record.name,
            inserted_record.data,
            inserted_record.created_at
        );

        Ok(())
    }

    pub fn print_all_records(connection: &mut SqliteConnection) -> Result<()> {
        let result_set = data_table::table
            .filter(data_table::id.is_not("1")) // Doesn't exclude anything, just an example.
            .limit(100) // Limit the number of records to fetch.
            .select(DataTableRecord::as_select())
            .load(connection)
            .into_diagnostic()?;

        println!("{} {}", "Number of records:".green(), result_set.len());

        for (index, record) in result_set.iter().enumerate() {
            println!(
                "{} ⴾ {}, {}, {}, {}",
                format!("Row #{}", (index + 1)).to_string().cyan(),
                record.id,
                record.name,
                record.data,
                // Format options: https://docs.rs/chrono/latest/chrono/format/strftime/index.html
                record.created_at.format("around %I:%M%P UTC on %b %-d")
            );
        }

        Ok(())
    }

    pub fn update_first_record(connection: &mut SqliteConnection) -> Result<()> {
        // Query to get the first record without using created_at.
        let maybe_first_record = data_table::table
            .select(DataTableRecord::as_select())
            .first(connection)
            .optional() // This won't throw error if no records are found.
            .into_diagnostic()?;

        // Only update the first record if it exists.
        if let Some(mut first_record) = maybe_first_record {
            // Save the ID for later.
            let id = first_record.id.as_ref();

            // Update the name field by appending a '*' to it.
            first_record.name = format!("{}{}", first_record.name, "*").into();

            // Update the record in the database.
            let updated_record = diesel::update(data_table::table.find(id))
                .set(&first_record)
                .returning(DataTableRecord::as_returning())
                .get_result(connection)
                .into_diagnostic()?;

            // Print the updated record.
            println!(
                "{} ⴾ {}, {}, {}, {}",
                "Updated record".yellow(),
                updated_record.id,
                updated_record.name,
                updated_record.data,
                // Format options: https://docs.rs/chrono/latest/chrono/format/strftime/index.html
                updated_record
                    .created_at
                    .format("around %I:%M%P UTC on %b %-d")
            );
        } else {
            println!("{}", "No records to update".yellow());
        }

        Ok(())
    }

    pub fn delete_last_record(connection: &mut SqliteConnection) -> Result<()> {
        // TODO: implement this method.
        Ok(())
    }
}

pub mod file_table_ops {
    // TODO: implement this module.
    use crate::diesel_sqlite_ex::schema::file_table::dsl::*;
}
