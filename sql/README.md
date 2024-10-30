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

- This will create a `test.db` file in the current directory.
- It will use the `rusqlite` library to interact with it.

## diesel and SQLite example

TODO: <https://diesel.rs/guides/getting-started.html>

## vscode and SQLite extension

You can install
[`qwtel.sqlite-viewer`](https://marketplace.visualstudio.com/items?itemName=qwtel.sqlite-viewer)
to view SQLite databases in VSCode. Alternatively you can use RustRover as db explorer is
built in.

## History

This [tracking bug](https://github.com/r3bl-org/r3bl-private-planning/issues/16) has lots
of background information regarding the exploration of SQL, Rust, the best database to
use, and the best driver & ORM combo.
