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

use super::{db_support, models::DataTableRecord, schema::data_table};
use crossterm::style::Stylize;
use diesel::prelude::*;
use diesel::{
    ExpressionMethods, OptionalExtension, RunQueryDsl, SelectableHelper, SqliteConnection,
    SqliteExpressionMethods,
};
use miette::{IntoDiagnostic, Result};
use r3bl_core::ok;

/// Generate 2 records, and insert them into [data_table].
///
/// # Get the timestamp for current time in UTC
///
/// - [chrono::Utc::now()] returns a [chrono::DateTime::naive_utc()] which is a
///   [chrono::NaiveDateTime].
///
/// # Convert the timestamp in the database to a [chrono::DateTime]
///
/// - Use [chrono::DateTime::from_naive_utc_and_offset] with the following args:
///   1. [chrono::NaiveDateTime] from the previous step.
///   2. `0` offset for the timezone.
pub fn insert_a_few_records(connection: &mut SqliteConnection) -> Result<()> {
    let timestamp = chrono::Utc::now().naive_utc();
    let new_records = vec![
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
    ];

    let num_records_inserted = diesel::insert_into(data_table::table)
        .values(&new_records) // Insert these structs into the table.
        .execute(connection) // Execute the insert query.
        .into_diagnostic()?;

    // Manually load the 2 records that were just inserted.
    let inserted_records = data_table::table
        .order(
            /*`desc()` stands for "descending" order, which means the results will be
            sorted from the highest value to the lowest (for dates or timestamps, this is from
            newest to oldest). */
            data_table::created_at.desc(),
        )
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

    ok!()
}

/// Update the first record in the [data_table] by adding `*` to [DataTableRecord::name].
/// Note that the "first" record is determined by the order in which the records are
/// stored. To enforce a specific order, you can use the `ORDER BY` clause.
pub fn update_first_record(connection: &mut SqliteConnection) -> Result<()> {
    // Get the first record ordered by created_at.
    let maybe_first_record = data_table::table
        .order(
            /*`desc()` stands for "descending" order, which means the results will be
            sorted from the highest value to the lowest (for dates or timestamps, this is from
            newest to oldest). */
            data_table::created_at.desc(),
        )
        .limit(1)
        .get_result::<DataTableRecord>(connection)
        .optional()
        .into_diagnostic()?;

    // Only update the first record if it exists.
    if let Some(mut first_record) = maybe_first_record {
        // Update the name field by appending a '*' to it.
        first_record.name = format!("{}{}", first_record.name, "*").into();

        // Update the record in the database.
        let updated_record = diesel::update(data_table::table.find(&first_record.id))
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

    ok!()
}

/// Delete the last record in the [data_table]. Note that the "last" record is determined
/// by the order in which the records are stored. To enforce a specific order, you can use
/// the `ORDER BY` clause.
pub fn delete_last_record(connection: &mut SqliteConnection) -> Result<()> {
    // Get the last record ordered by created_at.
    let maybe_last_record = data_table::table
        .order(
            /*`asc()` stands for "ascending" order, which means the results will be
            sorted from the lowest value to the highest (for dates or timestamps, this is from
            oldest to newest). */
            data_table::created_at.asc(),
        )
        .limit(1)
        .get_result::<DataTableRecord>(connection)
        .optional()
        .into_diagnostic()?;

    // Only delete the last record if it exists.
    if let Some(last_record) = maybe_last_record {
        // Delete the record from the database.
        let deleted_record = diesel::delete(data_table::table.find(last_record.id))
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

    ok!()
}

/// Print all records in the [data_table]:
/// - `id`
/// - `name`
/// - formatted [serde_json::Value]
/// - human readable `created_at`
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
            "{} ⴾ {}, {}, {:?}, {}",
            format!("Row #{}", (index + 1)).to_string().cyan(),
            record.id,
            record.name,
            db_support::convert_string_to_json(&record.data),
            db_support::format_datetime(record.created_at)
        );
    }

    ok!()
}
