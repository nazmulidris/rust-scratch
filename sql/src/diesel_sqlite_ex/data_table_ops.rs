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

use crate::diesel_sqlite_ex::{
    data_table,      /* from schema */
    DataTableRecord, /* from models */
};
use crossterm::style::Stylize;
use diesel::prelude::*;
use diesel::{
    ExpressionMethods, OptionalExtension, RunQueryDsl, SelectableHelper, SqliteConnection,
    SqliteExpressionMethods,
};
use miette::{IntoDiagnostic, Result};

/// # Get the timestamp for current time in UTC
/// - [chrono::Utc::now()] returns a [chrono::DateTime::naive_utc()] which is a [chrono::NaiveDateTime].
///
/// # Convert the timestamp in the database to a [chrono::DateTime]
/// - Use [chrono::DateTime::from_naive_utc_and_offset] with the following args:
///   1. [chrono::NaiveDateTime] from the previous step.
///   2. `0` offset for the timezone.
pub fn insert_a_few_records(connection: &mut SqliteConnection) -> Result<()> {
    let new_records = {
        let timestamp = chrono::Utc::now().naive_utc();
        vec![
            DataTableRecord {
                id: r3bl_core::generate_friendly_random_id().into(),
                name: petname::petname(2, " ").unwrap_or("John Doe".into()).into(),
                data: r#"{"key":"value1"}"#.into(),
                created_at: timestamp,
            },
            DataTableRecord {
                id: r3bl_core::generate_friendly_random_id().into(),
                name: petname::petname(2, " ").unwrap_or("Jane Doe".into()).into(),
                data: r#"{"key":"value2"}"#.into(),
                created_at: timestamp,
            },
        ]
    };

    let num_records_inserted = diesel::insert_into(data_table::table)
        .values(&new_records) // Insert these structs into the table.
        .execute(connection) // Execute the insert query.
        .into_diagnostic()?;

    // Fetch the inserted records manually.
    let inserted_records = data_table::table
        .order(data_table::id.desc())
        .limit(num_records_inserted as i64)
        .load::<DataTableRecord>(connection)
        .into_diagnostic()?;

    for inserted_record in inserted_records {
        println!(
            "{} ⴾ {}, {}, {}, {}",
            "Inserted record".magenta(),
            inserted_record.id,
            inserted_record.name,
            inserted_record.data,
            inserted_record.created_at
        );
    }

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
        // Convert the String containing JSON to a serde_json::Value.
        let json_data: serde_json::Value = serde_json::from_str(&record.data).into_diagnostic()?;

        println!(
            "{} ⴾ {}, {}, {:?}, {}",
            format!("Row #{}", (index + 1)).to_string().cyan(),
            record.id,
            record.name,
            json_data,
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
    // Query to get the last record without using created_at.
    let maybe_last_record = data_table::table
        .select(DataTableRecord::as_select())
        .order(data_table::id.desc())
        .first(connection)
        .optional() // This won't throw error if no records are found.
        .into_diagnostic()?;

    // Only delete the last record if it exists.
    if let Some(last_record) = maybe_last_record {
        // Save the ID for later.
        let id = last_record.id.as_ref();

        // Delete the record from the database.
        let deleted_record = diesel::delete(data_table::table.find(id))
            .returning(DataTableRecord::as_returning())
            .get_result(connection)
            .into_diagnostic()?;

        // Print the deleted record.
        println!(
            "{} ⴾ {}, {}, {}, {}",
            "Deleted record".red(),
            deleted_record.id,
            deleted_record.name,
            deleted_record.data,
            // Format options: https://docs.rs/chrono/latest/chrono/format/strftime/index.html
            deleted_record
                .created_at
                .format("around %I:%M%P UTC on %b %-d")
        );
    } else {
        println!("{}", "No records to delete".yellow());
    }

    Ok(())
}
