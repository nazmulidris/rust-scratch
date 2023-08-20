#!/usr/bin/env fish

# 1. Make sure to install cargo-outdated via `cargo install --locked cargo-outdated`.
# More info about cargo-outdated: https://crates.io/crates/cargo-outdated

echo (set_color brmagenta)"≡ Upgrading deps .. ≡"(set_color normal)
sh -c "cargo outdated --workspace --verbose"
sh -c "cargo upgrade --to-lockfile --verbose"
sh -c "cargo update"
