#!/usr/bin/env fish

# Make sure to install cargo-watch via `cargo install cargo-watch`.
cargo watch -x check -x test -c -q
