#!/usr/bin/env fish

# 1. Make sure to install cargo-outdated via `cargo install --locked cargo-outdated`.
# More info about cargo-outdated: https://crates.io/crates/cargo-outdated

echo (set_color brmagenta)"≡ Running tests .. ≡"(set_color normal)
cargo test -q -- --test-threads=1
