# sql

**Table of contents**

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Introduction](#introduction)
- [rusqlite and SQLite example](#rusqlite-and-sqlite-example)
- [diesel and SQLite example](#diesel-and-sqlite-example)
  - [1. Add the Cargo dependencies](#1-add-the-cargo-dependencies)
  - [2. Add Linux packages (sqlite-dev) and Diesel CLI](#2-add-linux-packages-sqlite-dev-and-diesel-cli)
  - [3. Use the Diesel CLI to create database file and migrations](#3-use-the-diesel-cli-to-create-database-file-and-migrations)
  - [4. Write SQL migrations, then run them to create tables and generate schema.rs](#4-write-sql-migrations-then-run-them-to-create-tables-and-generate-schemars)
    - [4.1. Location of the generated schema.rs file](#41-location-of-the-generated-schemars-file)
    - [4.2. For the current migration, change the up.sql file and run it again](#42-for-the-current-migration-change-the-upsql-file-and-run-it-again)
  - [5. Use the script, Luke](#5-use-the-script-luke)
    - [5.1. Instead of raw SQL, write Rust for migrations](#51-instead-of-raw-sql-write-rust-for-migrations)
    - [5.2. Include migrations in the final binary](#52-include-migrations-in-the-final-binary)
  - [6. Diesel and Rust](#6-diesel-and-rust)
- [VSCode and SQLite extension](#vscode-and-sqlite-extension)
- [History](#history)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Introduction

This crate is an exploration of SQL in Rust. All the examples use SQLite as the database. However,
the driver / ORM used is different in each example.

1. [rusqlite and SQLite example](#rusqlite-and-sqlite-example)
2. [diesel and SQLite example](#diesel-and-sqlite-example)
3. [sqlx and SQLite example](#sqlx-and-sqlite-example)

## rusqlite and SQLite example

The `rusqlite` library is a low-level SQLite driver for Rust.

- It is a thin wrapper around the SQLite C API.
- And it bundles the SQLite C library, so there is no need to install `sqlite3` on the system.

To run this example, use:

```sh
cargo run --bin rusqlite_ex
```

It does the following:

- This will create a `rusqlite.db` file in the current directory.
- It will use the `rusqlite` Rust crate to interact with it.

## diesel and SQLite example

The main instructions are from the
[official getting started guide](https://diesel.rs/guides/getting-started.html) for use with SQLite.

### Why Diesel?

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

There are a few ways to run `diesel setup`. The `path/to/your/database.db` is the value for
`DATABASE_URL`.

1. Use the `.env` file to store the `DATABASE_URL` environment variable. Run
   `echo DATABASE_URL=diesel.db > .env`. Then you can run `diesel setup`.

2. If you don't want to set this environment variable, you can just pass it inline to the shell
   `DATABASE_URL=diesel.db diesel setup`.

3. You can use the `--database-url` flag to specify the path to the database directly. For example:
   `diesel setup --database-url=diesel.db`.

We are going to use the 3rd option. Here are the commands to setup the database file.

```sh
diesel setup --database-url=diesel.db
```

This command actually creates the database file. The `diesel.db` file is created in the
current directory. If the migrations are already present (as can be gleaned from the
`diesel.toml` file), then the `schema.rs` file is generated and the `diesel.db` file is
generated.

### 4. Write SQL migrations, then run them to create tables and generate schema.rs

Now that the database file is created, we have to define our migration, that will actually run some
SQL that we provide and generate the `schema.rs` file. This process can also happen in reverse,
where we can write the `schema.rs` file first and ask the Diesel CLI to generate the SQL migrations
folder, and the `up.sql` and `down.sql` files.

The following commands will create a folder called `migrations` at the top level of the project.
Inside this folder, there will be one folder, for the migration called `create_tables`. The folder
will look like `<timestamp>_create_tables` and will contain an `up.sql` and `down.sql` file.

```sh
diesel migration generate --database-url=diesel.db create_tables
```

Migrations allow us to evolve the database schema over time. Each migration consists of an `up.sql`
file to apply the changes and a `down.sql` file to revert them. Applying and immediately reverting a
migration should leave your database schema unchanged.

> If you have multiple migrations, they will be applied in the order they were created. They are
> additive. In this example, we create a single migration, and the `up.sql` in it creates two
> tables. However, we could have split this into two migrations, one for each table. The `down.sql`
> does not get run when there are multiple migrations. It only gets run when you run
> `diesel migration redo` or `diesel migration revert`.

Once migrations are created they can be checked into version control. The folder structure for each
table has a timestamp in it, and contains a `up.sql` and `down.sql` file.

Here's the `up.sql`:

```sql
create table if not exists data_table (
  id text primary key,
  name text not null,
  data text not null
);
create table if not exists file_table (
  id text primary key,
  name text not null,
  data blob not null
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
diesel migration run --database-url=diesel.db
# This executes the `down.sql`, then `up.sql`.
diesel migration redo --database-url=diesel.db
```

The commands above will create a `diesel.db` file in the current directory if it does not
exist.

#### 4.1. Location of the generated schema.rs file

This will also generate the `schema.rs` file in the `src` directory. This file will have the Rust
representation of the tables in the database. You can change the location of this file by changing
the `diesel.toml` file and setting the path for the `print_schema:file` key. Here's an example:

```toml
[print_schema]
file = "src/diesel_sqlite_ex/schema.rs"
```

#### 4.2. For the current migration, change the up.sql file and run it again

If you want to change the current migration, you can edit the `up.sql` file and then run the
migration again. You can do this as many times as you want, without having to create a new
migration. This will simply regenerate the `schema.rs` file.

Here's how you can do that:

```sh
# Edit the up.sql file.
# Run the migration again.
diesel migration run --database-url=diesel.db
```

### 5. Use the script, Luke

In steps 1 through 5, there are a lot of manual steps. Use the script as follows:

1. Remove the following files & folders: `diesel.toml`, `diesel.db`, `migrations`, `src/schema.rs`.
2. Run `./diesel_setup.sh` and it will create the database file, create a migration, which will
   generate the `up.sql` and `down.sql` files. However, the migration will not be run and no `.db`
   file will be created.
3. Edit the `up.sql` and `down.sql` files to add the SQL for creating and dropping tables.
4. Run `./diesel_setup.sh` again, and tell it to run the migrations and generate the `diesel.db` and
   `schema.rs` file.
   1. This runs the `diesel migration run` command to exercise the `up.sql` file.
   2. And then runs the `diesel migration redo` command to exercise the `down.sql` file, and then
      the `up.sql` file.
   3. If you want to manually generate the `schema.rs` file, you can run
      `diesel print-schema > src/schema.rs`.

#### 5.1. Instead of raw SQL, write Rust for migrations

Alternatively, if you don't want to write raw SQL to do the migrations, you can just start with
writing the `src/schema.rs` file instead and then run
`diesel migration generate --diff-schema create_tables` to have it generate the `up.sql` and
`down.sql` files for you. The script does not currently support this.

#### 5.2. Include migrations in the final binary

When preparing your app for use in production, you may want to run your migrations during the
application's initialization phase. You may also want to include the migration scripts as a part of
your code, to avoid having to copy them to your deployment location/image etc.

You can also include the migrations in the final binary of the application you're building
by using the [`diesel_migration` crate's `embed_migrations!`
macro](https://docs.rs/diesel_migrations/2.2.0/diesel_migrations/macro.embed_migrations.html).
This way there is no manual setup required to run the migrations and can be handled by the
binary itself.

### 6. Diesel and Rust

#### The connection

We can just specify the path to the database directly when needed, instead of using the
`DATABASE_URL` environment variable (and using `.env` and and `dotenvy` crate). Here's an example of
this in Rust:

```rust
use diesel::prelude::*;
use miette::*;

/// Specify your database URL, eg: "path/to/your/database.db".
pub fn create_connection(database_url: &str) -> Result<SqliteConnection> {
    SqliteConnection::establish(database_url).into_diagnostic()
}
```

#### Automatically run migrations

Let's say that the `diesel.db` file is not present, since you haven't done any of the following:
- Run the `diesel_setup.fish` script file.
- Run the `diesel setup` command.
- Run the `diesel migration run` command.

In this case your application will not work, since the database file is not present. You
can automatically run the migrations when the application starts, if the database file is
not present, it will be created.

// TODO: Add code to automatically run migrations <https://gemini.google.com/app/5f1b885c0db4e4f4>

#### CRUD operations

This [example](https://diesel.rs/guides/getting-started.html) demonstrates how to do CRUD
operations with Diesel and Sqlite. The
[example](https://github.com/nazmulidris/rust-scratch/blob/main/sql/src/diesel_sqlite_ex/diesel_impl.rs)
provides examples of implementing CRUD on two different tables, one that holds structured
JSON text data, and another that holds binary data.

## VSCode and SQLite extension

1. You can install
   [`qwtel.sqlite-viewer`](https://marketplace.visualstudio.com/items?itemName=qwtel.sqlite-viewer)
   to view SQLite databases in VSCode. Alternatively you can use RustRover as db explorer is built
   in.

2. You can install
   [`adpyke.vscode-sql-formatter`](https://marketplace.visualstudio.com/items?itemName=adpyke.vscode-sql-formatter)
   to format SQL queries in VSCode.

## History

This [tracking bug](https://github.com/r3bl-org/r3bl-private-planning/issues/16) has lots of
background information regarding the exploration of SQL, Rust, the best database to use, and the
best driver & ORM combo.
