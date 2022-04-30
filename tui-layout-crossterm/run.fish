#!/usr/bin/env fish
clear
cargo update
cargo build
RUST_BACKTRACE=1 cargo run
