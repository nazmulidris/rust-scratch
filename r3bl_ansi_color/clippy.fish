#!/usr/bin/env fish

# More info about cargo-watch: https://lib.rs/crates/cargo-watch

cargo fix --allow-dirty --allow-staged
cargo fmt --all
RUST_BACKTRACE=1 cargo watch -x 'clippy --fix --allow-dirty --allow-staged' -c -q

# OG command:
# cargo clippy --fix --allow-dirty --allow-staged
