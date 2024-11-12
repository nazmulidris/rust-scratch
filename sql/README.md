# sql

**Table of contents**

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Introduction](#introduction)
- [rusqlite and SQLite example](#rusqlite-and-sqlite-example)
- [diesel and SQLite example](#diesel-and-sqlite-example)
  - [Why Diesel and not SQLx?](#why-diesel-and-not-sqlx)
  - [1. Add the Cargo dependencies](#1-add-the-cargo-dependencies)
  - [2. Add Linux packages (sqlite-dev) and Diesel CLI](#2-add-linux-packages-sqlite-dev-and-diesel-cli)
  - [3. Use the Diesel CLI to create database file and migrations](#3-use-the-diesel-cli-to-create-database-file-and-migrations)
  - [4. Write SQL migrations, then run them to create tables and generate schema.rs](#4-write-sql-migrations-then-run-them-to-create-tables-and-generate-schemars)
    - [4.1 What is the difference between redo and run?](#41-what-is-the-difference-between-redo-and-run)
    - [4.2. Location of the generated schema.rs file](#42-location-of-the-generated-schemars-file)
    - [4.3. For the current migration, change the up.sql file and run it again](#43-for-the-current-migration-change-the-upsql-file-and-run-it-again)
  - [5. Use the script, Luke](#5-use-the-script-luke)
    - [5.1. Instead of raw SQL, write Rust for migrations](#51-instead-of-raw-sql-write-rust-for-migrations)
    - [5.2. Include migrations in the final binary](#52-include-migrations-in-the-final-binary)
  - [6. Add a new migration that changes existing tables by adding a new column and preserve data](#6-add-a-new-migration-that-changes-existing-tables-by-adding-a-new-column-and-preserve-data)
  - [7. Diesel and Rust](#7-diesel-and-rust)
    - [Create a connection](#create-a-connection)
    - [CRUD operations](#crud-operations)
    - [Timestamps](#timestamps)
    - [Automatically run migrations](#automatically-run-migrations)
- [VSCode and SQLite extension](#vscode-and-sqlite-extension)
- [History](#history)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Introduction

This crate is an exploration of SQL in Rust. All the examples use SQLite as the database.
However, the driver / ORM used is different in each example.

1. [`rusqlite` and SQLite example](#rusqlite-and-sqlite-example)
2. [`diesel` and SQLite example](#diesel-and-sqlite-example)

## rusqlite and SQLite example

The `rusqlite` library is a low-level SQLite driver for Rust.

- It is a thin wrapper around the SQLite C API.
- And it bundles the SQLite C library, so there is no need to install `sqlite3` on the
  system.

The primary use case that this example addresses is storing an application's settings that
are a mix of binary and JSON formatted text data. Using the filesystem naively where we
have a separate file for each, can cause problems in scenarios where multiple processes of
this binary run concurrently. Instead we will use a SQLite database to store this data.

This example works with 2 tables:

1. One contains JSON Text formatted data, and the
2. The other contains binary data that's read for a file.

This is meant to demonstrate how to work with JSON encoded data and binary data in SQLite
using Rust. The example
[here](https://github.com/nazmulidris/rust-scratch/blob/main/sql/src/bin/rusqlite_ex.rs)
does the following:

- This will create a `rusqlite.db` file in the current directory.
- It will use the `rusqlite` Rust crate to interact with it to perform some simple CRUD
  operations on the database.
- The code is very simple, there is no ORM, or SQL syntax checking, or migrations. The SQL
  is just written as Rust strings.
- There are 2 tables, one containing JSON text data, and the other containing binary data.

To run this example, use:

```sh
cargo run --bin rusqlite
```

## diesel and SQLite example

The `diesel` library is a high-level ORM for Rust.

> The main instructions are from the
> [`diesel` official getting started guide](https://diesel.rs/guides/getting-started.html)
> for use with SQLite.

In this example we will work with Rust, Diesel, and SQLite to setup databases, using
migrations, and do CRUD operations in Rust code. Here are the details:

1. We will create a migration to setup the database.
2. Then create the models.
3. And write some code to do CRUD operations in Rust.
4. Then we will add another migration to alter the database, then migrate any existing
   data, and then update the models to reflect the changes.
5. Finally, we will automate these migrations so that they don't have to be run manually.
   And they will be done in your binary target when it starts.

The example
[here](https://github.com/nazmulidris/rust-scratch/blob/main/sql/src/bin/diesel_sqlite_ex.rs)
does the following:

- This will create a `diesel.db` file in the current directory. It runs migrations as well
  programmatically when the binary runs, at the very start.
- It will use the `diesel` Rust crate (and ORM) to interact with it to perform some simple
  CRUD operations on the database.
- There are 2 tables, one containing JSON text data, and the other containing binary data.

To run this example, use:

```sh
cargo run --bin diesel
```

### Why Diesel and not SQLx?

[Here](https://users.rust-lang.org/t/which-one-to-use-postgres-vs-sqlx/63680/6) are some
reasons to use Diesel over SQLx.

### 1. Add the Cargo dependencies

Here are the commands to add the required dependencies in `Cargo.toml`:

```toml
diesel = { version = "2.2.0", features = ["sqlite", "returning_clauses_for_sqlite_3_35"] }
dotenvy = "0.15"
```

### 2. Add Linux packages (sqlite-dev) and Diesel CLI

> All the steps between 2 and 5 can be automated by running the `diesel_setup.sh` script.

Here are the commands to setup the Diesel CLI for SQLite for Linux:

```sh
sudo apt install libsqlite3-dev
cargo install diesel_cli --no-default-features --features sqlite
```

### 3. Use the Diesel CLI to create database file and migrations

There are a few ways to run `diesel setup`. The `path/to/your/database.db` is the value
for `DATABASE_URL`.

1. Use the `.env` file to store the `DATABASE_URL` environment variable. Run
   `echo DATABASE_URL=diesel.db > .env`. Then you can run `diesel setup`.

2. If you don't want to set this environment variable, you can just pass it inline to the
   shell `DATABASE_URL=diesel.db diesel setup`.

3. You can use the `--database-url` flag to specify the path to the database directly. For
   example: `diesel --database-url=diesel.db setup`.

We are going to use the 3rd option. Here are the commands to setup the database file.

```sh
diesel --database-url=diesel.db setup
```

This command actually creates the database file. The `diesel.db` file is created in the
current directory. If the migrations are already present (as can be gleaned from the
`diesel.toml` file), then the `schema.rs` file is generated and the `diesel.db` file is
generated.

### 4. Write SQL migrations, then run them to create tables and generate schema.rs

Now that the database file is created, we have to define our migration, that will actually
run some SQL that we provide and generate the `schema.rs` file. This process can also
happen in reverse, where we can write the `schema.rs` file first and ask the Diesel CLI to
generate the SQL migrations folder, and the `up.sql` and `down.sql` files.

The following commands will create a folder called `migrations` at the top level of the
project. Inside this folder, there will be one folder, for the migration called
`create_tables`. The folder will look like `<timestamp>_create_tables` and will contain an
`up.sql` and `down.sql` file.

```sh
diesel --database-url=diesel.db migration generate create_tables
```

Migrations allow us to evolve the database schema over time. Each migration consists of an
`up.sql` file to apply the changes and a `down.sql` file to revert them. Applying and
immediately reverting a migration should leave your database schema unchanged.

> If you have multiple migrations, they will be applied in the order they were created.
> They are additive. In this example, we create a single migration, and the `up.sql` in it
> creates two tables. However, we could have split this into two migrations, one for each
> table. The `down.sql` does not get run when there are multiple migrations. It only gets
> run when you run `diesel migration redo` or `diesel migration revert`.

Once migrations are created they can be checked into version control. The folder structure
for each table has a timestamp in it, and contains a `up.sql` and `down.sql` file.

Here's the `up.sql`:

```sql
create table if not exists data_table (
  id text primary key not null,
  name text not null,
  data text not null
);

create table if not exists file_table (
  id text primary key not null,
  name text not null,
  data text not null
);
```

Here's the `down.sql`:

```sql
drop table if exists data_table;

drop table if exists file_table;
```

Then execute the migrations:

```sh
# This executes the `up.sql` file.
diesel --database-url=diesel.db migration run
# This executes the `down.sql`, then `up.sql`.
diesel --database-url=diesel.db migration redo
```

The commands above will create a `diesel.db` file in the current directory if it does not
exist.

#### 4.1 What is the difference between redo and run?

- `redo` will run the `down.sql` and then the `up.sql` file.
- `run` will only run the `up.sql` file.

> Both commands will preserve any existing data in the `diesel.db` file. Migrations will
> not destroy the data in the tables, unless you explicitly write SQL to do so.

Let's say you have `run` the migration and then you make a change to `up.sql` above, and
add a new column. If you run `run` again you will **not** see these changes in your
`schema.rs` file!

You could run `redo`, which will run `down.sql` and then `up.sql`, and this should drop
the table from the `diesel.db` file and then recreate it with the new column.

However, in this scenario it might be best to create a new migration and not modify the
existing one. This way you can keep track of the changes you made to the database schema
over time. Once you create the
[new migration](#6-add-a-new-migration-that-changes-existing-tables-by-adding-a-new-column-and-preserve-data),
you can run `diesel --database-url=diesel.db migration run` to apply the changes.

#### 4.2. Location of the generated schema.rs file

This will also generate the `schema.rs` file in the `src` directory. This file will have
the Rust representation of the tables in the database. You can change the location of this
file by changing the `diesel.toml` file and setting the path for the `print_schema:file`
key. Here's an example:

```toml
[print_schema]
file = "src/diesel_sqlite_ex/schema.rs"
```

#### 4.3. For the current migration, change the up.sql file and run it again

If you want to change the current migration, you can edit the `up.sql` file and then run
the migration again. You can do this as many times as you want, without having to create a
new migration. This will simply regenerate the `schema.rs` file.

Here's how you can do that:

```sh
# Edit the up.sql file.
# Run the migration again.
diesel --database-url=diesel.db migration run
```

### 5. Use the script, Luke

In steps 1 through 5, there are a lot of manual steps. Use the script as follows:

1. Remove the following files & folders: `diesel.toml`, `diesel.db`, `migrations`,
   `src/schema.rs`.
2. Run `./diesel_setup.sh` and it will create the database file, create a migration, which
   will generate the `up.sql` and `down.sql` files. However, the migration will not be run
   and no `.db` file will be created.
3. Edit the `up.sql` and `down.sql` files to add the SQL for creating and dropping tables.
4. Run `./diesel_setup.sh` again, and tell it to run the migrations and generate the
   `diesel.db` and `schema.rs` file.
   1. This runs the `diesel migration run` command to exercise the `up.sql` file.
   2. And then runs the `diesel migration redo` command to exercise the `down.sql` file,
      and then the `up.sql` file.
   3. If you want to manually generate the `schema.rs` file, you can run
      `diesel print-schema > src/schema.rs`.

#### 5.1. Instead of raw SQL, write Rust for migrations

Alternatively, if you don't want to write raw SQL to do the migrations, you can just start
with writing the `src/schema.rs` file instead and then run
`diesel migration generate --diff-schema create_tables` to have it generate the `up.sql`
and `down.sql` files for you. The script does not currently support this.

#### 5.2. Include migrations in the final binary

When preparing your app for use in production, you may want to run your migrations during
the application's initialization phase. You may also want to include the migration scripts
as a part of your code, to avoid having to copy them to your deployment location/image
etc.

You can also include the migrations in the final binary of the application you're building
by using the
[`diesel_migration` crate's `embed_migrations!` macro](https://docs.rs/diesel_migrations/2.2.0/diesel_migrations/macro.embed_migrations.html).
This way there is no manual setup required to run the migrations and can be handled by the
binary itself.

### 6. Add a new migration that changes existing tables by adding a new column and preserve data

Let's say you have everything working so far, and you want to alter the existing tables by
adding a new column, there are few things to keep in mind:

- There's might be data in the tables, which are in the `diesel.db` file.
- You want to preserve this data when you add a new column.
- When you add a new column, you have to backfill the data in the existing rows which were
  created when this column didn't exist.

Here are the steps to create a new migration to alter existing tables by adding a new
column `created_at`:

1. Create a new migration using:

   ```sh
   diesel --database-url=diesel.db migration generate add_new_column_to_both_tables
   ```

2. Populate `up.sql` file in the new migration with the following SQL:

   ```sql
   -- Add a new column created_at to data_table. This can't be current_timestamp because
   -- SQLite doesn't support that. The default value must be a constant.
   alter table
      data_table
   add
      column created_at timestamp not null default '1900-01-01 12:12:12';

   -- Add a new column created_at to file_table. This can't be current_timestamp because
   -- SQLite doesn't support that. The default value must be a constant.
   alter table
      file_table
   add
      column created_at timestamp not null default '1900-01-01 12:12:12';

   -- Update the created_at column in data_table if needed (it is needed if the row's date is
   -- hard coded to '1900-01-01 12:12:12'.
   update
      data_table
   set
      created_at = current_timestamp
   where
      created_at is '1900-01-01 12:12:12';
   ```

3. Populate the `down.sql` file in the new migration with the following SQL:

   ```sql
   -- Drop the created_at column from data_table.
   alter table
      data_table
   drop
      column created_at;

   -- Drop the created_at column from file_table.
   alter table
      file_table
   drop
      column created_at;
   ```

4. Finally run:

   ```sh1
   diesel --database-url=diesel.db migration run
   ```

Once all this is done, your `diesel.db` file will have the new column `created_at` in both
tables. However, the `models` are still not updated to reflect this change. You can update
the structs in the `models` module manually to accommodate these changes. Since the change
is SQL column type `TIMESTAMP` related, you can add the following field:

```rust
pub struct DataTableRecord<'a> {
   pub created_at: chrono::NaiveDateTime,
   ...
}

pub struct FileTableRecord<'a> {
   pub created_at: chrono::NaiveDateTime,
   ...
}
```

See the [timestamps](#timestamps) section for more information on handling timestamps in
Diesel, SQLite and Rust.

### 7. Diesel and Rust

#### Create a connection

We can just specify the path to the database directly when needed, instead of using the
`DATABASE_URL` environment variable (and using `.env` and and `dotenvy` crate). There are
a few ways in which you can specify the database URL:

- `path/to/your/file.db` - Save the database file in the given path.
- `file://file.db` - Save the database file in given path.
- `:memory:` - Create an in-memory database.

Here's an example of this in Rust:

```rust
use diesel::prelude::*;
use miette::*;

/// Specify your database URL, eg:
/// - `path/to/your/file.db` - Save the database file in the given path.
/// - `file://file.db` - Save the database file in given path.
/// - `:memory:` - Create an in-memory database.
///
/// See [SqliteConnection] for more details.
pub fn create_connection(database_url: &str) -> Result<SqliteConnection> {
   SqliteConnection::establish(database_url).into_diagnostic()
}
```

#### CRUD operations

This [example](https://diesel.rs/guides/getting-started.html) demonstrates how to do CRUD
operations with Diesel and Sqlite. The
[example](https://github.com/nazmulidris/rust-scratch/blob/main/sql/src/diesel_sqlite_ex/diesel_impl.rs)
provides examples of implementing CRUD on two different tables, one that holds structured
JSON text data, and another that holds binary data.

#### Timestamps

To handle SQLite `TIMESTAMP` column type in Rust, add the `chrono` feature in Diesel (in
your `Cargo.toml`). Also add a dependency on the `chrono` crate. Here's a full listing of
the required dependencies for your `Cargo.toml`:

```toml
diesel = { version = "2.2.4", features = [
  # For SQLite support.
  "sqlite",
  # The enables returning clauses for SQLite 3.35 and later.
  "returning_clauses_for_sqlite_3_35",
  # For timestamp support.
  "chrono",
] }

chrono = "0.4"
```

In the code, you can handle timestamps as follows:

- **Save** - Get the timestamp for current time in UTC. Do this in Rust, to create a new
  timestamp that will be inserted or updated into the database.
  - [chrono::Utc::now()](https://docs.rs/chrono/latest/chrono/offset/struct.Utc.html)
    returns a
    [chrono::DateTime::naive_utc()](https://docs.rs/chrono/latest/chrono/struct.DateTime.html)s
    which is a
    [chrono::NaiveDateTime](https://docs.rs/chrono/latest/chrono/naive/struct.NaiveDateTime.html).
- **Load** - Convert the timestamp in the database to a
  [chrono::DateTime](https://docs.rs/chrono/latest/chrono/struct.DateTime.html#). Do this
  in Rust, to read the timestamp from the database.
  - Use
    [chrono::DateTime::from_naive_utc_and_offset](https://docs.rs/chrono/latest/chrono/struct.DateTime.html#method.from_naive_utc_and_offset)
    with the following args:
    1. [chrono::NaiveDateTime](https://docs.rs/chrono/latest/chrono/naive/struct.NaiveDateTime.html)
       from the previous step.
    2. `0` offset for the timezone.
- **Human readable format** - Convert a
  [chrono::NaiveDateTime](https://docs.rs/chrono/latest/chrono/naive/struct.NaiveDateTime.html)
  to a human readable string.
  - Use the format options in
    [chrome::NaiveDateTime::format()](https://docs.rs/chrono/latest/chrono/format/strftime/index.html)
    to format the timestamp. To get the output `around 10:44pm UTC on Nov 11`, you can
    use:
    ```rust
    record.created_at.format("around %I:%M%P UTC on %b %-d")
    ```

#### Automatically run migrations

Let's say that the `diesel.db` file is not present, since you have **NOT** done any of the
following:

- Run the `diesel_setup.fish` script file.
- Run the `diesel setup` command.
- Run the `diesel migration run` command.

Or a `diesel.db` file is present, and you just added a new migration **BUT** you didn't
run it yet.

In this case your application will not work, since the database file is not present, or it
is out of date 😮.

Thankfully, you can have the migrations run automatically when the application starts, if
the database file is not present, it will be created. If the database file is old, it will
be updated to the latest version 🎉.

In order to make this happen you have to do the following things.

1. Add the `diesel_migrations` crate to your `Cargo.toml` file:

   ```toml
   # For automatic migrations.
   diesel_migrations = { version = "2.2.0", features = ["sqlite"] }
   ```

2. Add a procedural macro and this function:

   ```rust
   use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
   pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
   pub fn try_run_migrations(
       connection: &mut SqliteConnection,
   ) -> std::result::Result<
       Vec<diesel::migration::MigrationVersion<'_>>,
       Box<dyn std::error::Error + Send + Sync>,
   > {
       connection.run_pending_migrations(MIGRATIONS)
   }
   ```

3. Finally, to your `main.rs` file, or whatever file and function you want to run before
   any database operations are run in your binary, call the function above. For example:

   ```rust
   let connection = &mut general_ops::create_connection(DATABASE_URL)?;
   if migration_ops::try_run_migrations(connection).is_err() {
       println!("Error running migrations");
       miette::bail!("Error running migrations");
   }
   ```

4. Optionally, you can add a `build.rs` file at the root of your project to get around
   current limitations in Rust's `proc-macro` API. There is currently no way to signal
   that a specific proc macro should be rerun if some external file changes or is added.
   Which means that `embed_migrations!` cannot regenerate the list of embedded migrations
   if **ONLY** the migrations are changed. To get around this you can add the following to
   your `build.rs` file:

   ```rust
   fn main() {
      println!("cargo:rerun-if-changed=migrations");
   }
   ```

That's it! Now your application will automatically run migrations when it starts 🚀.

## VSCode and SQLite extension

1. You can install
   [`qwtel.sqlite-viewer`](https://marketplace.visualstudio.com/items?itemName=qwtel.sqlite-viewer)
   to view SQLite databases in VSCode. Alternatively you can use RustRover as db explorer
   is built in.

2. You can install
   [`adpyke.vscode-sql-formatter`](https://marketplace.visualstudio.com/items?itemName=adpyke.vscode-sql-formatter)
   to format SQL queries in VSCode.

## History

This [tracking bug](https://github.com/r3bl-org/r3bl-private-planning/issues/16) has lots
of background information regarding the exploration of SQL, Rust, the best database to
use, and the best driver & ORM combo.
