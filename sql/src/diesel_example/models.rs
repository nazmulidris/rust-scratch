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

//! Here are a list of descriptions of what each of the annotations see in the Diesel
//! derive macros below.
//!
//! [Queryable] generates all the code needed the load the annotated struct from a SQL
//! query.
//!
//! [Selectable] generates all the code needed to create a select clause based on the
//! struct and table annotation.
//!
//! [Insertable] generates all the code needed to create an insert clause based on the
//! struct and table annotation.
//!
//! [AsChangeset] generates all the code needed to create an update clause based on the
//! struct and table annotation.
//!
//! The attribute
//! [check_for_backend(diesel::sqlite::Sqlite)](https://docs.diesel.rs/master/diesel/prelude/derive.Selectable.html#optional-type-attributes)
//! can improve the error messages generated by the compiler significantly. It adds
//! additional compile time checks to verify that all field types in your struct are
//! compatible with their corresponding SQL side expressions. This part is optional, but
//! it greatly improves the generated compiler error messages.

use diesel::prelude::*;
use std::borrow::Cow;

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::diesel_example::schema::data_table)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DataTableRecord<'a> {
    pub id: Cow<'a, str>,
    pub name: Cow<'a, str>,
    pub data: Cow<'a, str>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::diesel_example::schema::file_table)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct FileTableRecord<'a> {
    pub id: Cow<'a, str>,
    pub name: Cow<'a, str>,
    pub data: Cow<'a, Vec<u8>>,
    pub created_at: chrono::NaiveDateTime,
}
