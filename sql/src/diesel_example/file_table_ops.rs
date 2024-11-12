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

use super::{constants::FILENAME_TO_READ, db_support, models::FileTableRecord, schema::file_table};
use crossterm::style::Stylize as _;
use diesel::prelude::*;
use miette::IntoDiagnostic;
use r3bl_core::ok;
use std::borrow::Cow;

/// Generate 2 records, by reading the [FILENAME_TO_READ] file, insert into [file_table].
pub fn insert_a_few_records(connection: &mut diesel::SqliteConnection) -> miette::Result<()> {
    let blob = db_support::try_read_bytes_from_file(FILENAME_TO_READ)?;
    let timestamp = db_support::get_current_timestamp();
    let id_1 = r3bl_core::generate_friendly_random_id();
    let id_2 = r3bl_core::generate_friendly_random_id();
    let new_records = vec![
        FileTableRecord {
            id: Cow::Borrowed(&id_1),
            name: petname::petname(2, " ").unwrap_or("John Doe".into()).into(),
            data: Cow::Borrowed(&blob),
            created_at: timestamp,
        },
        FileTableRecord {
            id: Cow::Borrowed(&id_2),
            name: petname::petname(2, " ").unwrap_or("Jane Doe".into()).into(),
            data: Cow::Borrowed(&blob),
            created_at: timestamp,
        },
    ];

    let num_records_inserted = diesel::insert_into(file_table::table)
        .values(&new_records)
        .execute(connection)
        .into_diagnostic()?;

    println!(
        "{} {}",
        "Number of records inserted into file_table:".magenta(),
        num_records_inserted
    );

    // Manually load the 2 records that were inserted into the file_table with id_1 and
    // id_2.
    let inserted_records = file_table::table
        .filter(file_table::id.eq(&id_1).or(file_table::id.eq(&id_2)))
        .load::<FileTableRecord>(connection)
        .into_diagnostic()?;

    for inserted_record in inserted_records {
        println!(
            "{} ⴾ {}, {}, {}, {}",
            "Inserted record".magenta(),
            inserted_record.id,
            inserted_record.name,
            db_support::get_sha_for_bytes(&inserted_record.data),
            inserted_record.created_at
        );
    }

    ok!()
}

/// Update the first record in the [file_table] by adding `*` to [FileTableRecord::name].
/// Note that the "first" record is determined by the order in which the records are
/// stored. To enforce a specific order, you can use the `ORDER BY` clause.
pub fn update_first_record(connection: &mut diesel::SqliteConnection) -> miette::Result<()> {
    // Get the first record ordered by created_at.
    let maybe_first_record = file_table::table
        .order(
            /*`desc()` stands for "descending" order, which means the results will be
            sorted from the highest value to the lowest (for dates or timestamps, this is from
            newest to oldest). */
            file_table::created_at.desc(),
        )
        .limit(1)
        .get_result::<FileTableRecord>(connection)
        .optional()
        .into_diagnostic()?;

    // Only update the first record if it exists.
    if let Some(mut first_record) = maybe_first_record {
        // Update the first record by adding `*` to the name & return it.
        first_record.name = format!("{}*", first_record.name).into();

        // Update the record in the database.
        let updated_record = diesel::update(file_table::table.find(&first_record.id))
            .set(&first_record)
            .returning(FileTableRecord::as_returning())
            .get_result(connection)
            .into_diagnostic()?;

        // Print the updated record.
        println!(
            "{} ⴾ {}, {}, {}, {}",
            "Updated record".yellow(),
            updated_record.id,
            updated_record.name,
            db_support::get_sha_for_bytes(&updated_record.data),
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

/// Delete the last record in the [file_table]. Note that the "last" record is determined
/// by the order in which the records are stored. To enforce a specific order, you can use
/// the `ORDER BY` clause.
pub fn delete_last_record(connection: &mut diesel::SqliteConnection) -> miette::Result<()> {
    // Get the last record ordered by created_at.
    let maybe_last_record = file_table::table
        .order(
            /*`asc()` stands for "ascending" order, which means the results will be
            sorted from the lowest value to the highest (for dates or timestamps, this is from
            oldest to newest). */
            file_table::created_at.asc(),
        )
        .limit(1)
        .get_result::<FileTableRecord>(connection)
        .optional()
        .into_diagnostic()?;

    // Only delete the last record if it exists.
    if let Some(last_record) = maybe_last_record {
        // Delete the record from the database.
        let deleted_record = diesel::delete(file_table::table.find(&last_record.id))
            .returning(FileTableRecord::as_returning())
            .get_result::<FileTableRecord>(connection)
            .into_diagnostic()?;

        // Print the deleted record.
        println!(
            "{} ⴾ {}, {}, {}, {}",
            "Deleted record".red(),
            deleted_record.id,
            deleted_record.name,
            db_support::get_sha_for_bytes(&deleted_record.data),
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

/// Print all records in the [file_table]:
/// - `id`
/// - `name`
/// - sha of the bytes
/// - human readable `created_at`
pub fn print_all_records(connection: &mut diesel::SqliteConnection) -> miette::Result<()> {
    let result_set = file_table::table
        .select(FileTableRecord::as_select())
        .load(connection)
        .into_diagnostic()?;

    println!("{} {}", "Number of records:".green(), result_set.len());

    for (index, record) in result_set.iter().enumerate() {
        println!(
            "{} ⴾ {}, {}, {:?}, {}",
            format!("Row #{}", (index + 1)).to_string().cyan(),
            record.id,
            record.name,
            db_support::get_sha_for_bytes(&record.data),
            db_support::format_datetime(record.created_at)
        );
    }

    ok!()
}
