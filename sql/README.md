<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Table of Contents**

- [sql](#sql)
  - [rusqlite and SQLite example](#rusqlite-and-sqlite-example)
  - [diesel and SQLite example](#diesel-and-sqlite-example)
    - [1. Add the Cargo dependencies](#1-add-the-cargo-dependencies)
    - [2. Add Linux packages (sqlite-dev) and Diesel CLI](#2-add-linux-packages-sqlite-dev-and-diesel-cli)
    - [3. Use the Diesel CLI to setup tables and migrations](#3-use-the-diesel-cli-to-setup-tables-and-migrations)
    - [4. Write SQL migrations (schema)](#4-write-sql-migrations-schema)
    - [5. Diesel and Rust](#5-diesel-and-rust)
  - [VSCode and SQLite extension](#vscode-and-sqlite-extension)
  - [History](#history)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

# sql

This crate is an exploration of SQL in Rust. All the examples use SQLite as the database.
However, the driver / ORM used is different in each example.

1. [rusqlite and SQLite example](#rusqlite-and-sqlite-example)
2. [diesel and SQLite example](#diesel-and-sqlite-example)
3. [sqlx and SQLite example](#sqlx-and-sqlite-example)

## rusqlite and SQLite example

The `rusqlite` library is a low-level SQLite driver for Rust.
- It is a thin wrapper around
  the SQLite C API.
- And it bundles the SQLite C library, so there is no need to install `sqlite3` on the
  system.

To run this example, use:

```sh
cargo run --bin rusqlite_ex
```

It does the following:

- This will create a `rusqlite.db` file in the current directory.
- It will use the `rusqlite` Rust crate to interact with it.

## diesel and SQLite example

TODO: <https://diesel.rs/guides/getting-started.html>

### 1. Add the Cargo dependencies

Here are the commands to add the required dependencies:

```sh
cargo add diesel --features sqlite
# The following is only required if you plan to use `.env` files to get the `DATABASE_URL`.
# cargo add dotenv
```

### 2. Add Linux packages (sqlite-dev) and Diesel CLI

Here are the commands to setup the Diesel CLI for SQLite:

```sh
sudo apt install libsqlite3-dev
cargo install diesel_cli --no-default-features --features sqlite
```

### 3. Use the Diesel CLI to setup tables and migrations

There are a few ways to run `diesel setup`. The `path/to/your/database.db` is the value
for `DATABASE_URL`.

1. Use the `.env` file to store the `DATABASE_URL` environment variable. Run `echo
   DATABASE_URL=diesel.db > .env`. Then you can run `diesel setup`.

2. If you don't want to set this environment variable, you can just pass it inline to the
   shell `DATABASE_URL=diesel.db diesel setup`.

3. You can use the `--database-url` flag to specify the path to the database directly. For
   example: `diesel setup --database-url=diesel.db`.

We are going to use the 3rd option. Here are the commands to setup the tables and
migrations:

```sh
diesel setup --database-url=diesel.db
diesel migration generate --database-url=diesel.db data_table
diesel migration generate --database-url=diesel.db file_table
```

These commands will create a folder called `migrations` in the current directory. Inside
this folder, there will be two folders: `data_table` and `file_table`, each containing a
`up.sql` and `down.sql` file.

### 4. Write SQL migrations (schema)

TODO: write SQL migrations for `data_table` & `file_table`, in `up.sql` & `down.sql`.

### 5. Diesel and Rust

We can just specify the path to the database directly when needed, instead of using the
`DATABASE_URL` environment variable (and using `.env` and and `dotenvy` crate). Here's an
example of this in Rust:

```rust
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

fn establish_connection() -> SqliteConnection {
    let database_url = "diesel.db"; // Specify your database URL here as `path/to/your/database.db`
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
```

## VSCode and SQLite extension

You can install
[`qwtel.sqlite-viewer`](https://marketplace.visualstudio.com/items?itemName=qwtel.sqlite-viewer)
to view SQLite databases in VSCode. Alternatively you can use RustRover as db explorer is
built in.

## History

This [tracking bug](https://github.com/r3bl-org/r3bl-private-planning/issues/16) has lots
of background information regarding the exploration of SQL, Rust, the best database to
use, and the best driver & ORM combo.
