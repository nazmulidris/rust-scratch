#!/usr/bin/env fish

# Make sure to install cargo-watch via `cargo install cargo-watch`.
# More info about cargo-watch: https://crates.io/crates/cargo-watch
cargo watch -x check -x 'test -q --color always' -c -q

# cargo test -q --color always
