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
use sql::diesel_sqlite_ex::{data_table_ops, general_ops, DATABASE_URL};

fn main() -> miette::Result<()> {
    println!(
        "{}",
        "Running diesel_sqlite_ex".magenta().bold().underlined()
    );

    let connection = &mut general_ops::create_connection(DATABASE_URL)?;

    data_table_ops::insert_a_few_records(connection)?;
    data_table_ops::update_first_record(connection)?;
    data_table_ops::delete_last_record(connection)?;
    data_table_ops::print_all_records(connection)?;

    Ok(())
}
