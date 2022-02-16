#!/usr/bin/env fish

# https://stackoverflow.com/a/47743269/2085356
if test -z "$argv"
    echo "Usage: "(set_color -o -u)"cargo-one.fish "(set_color normal)\
    (set_color yellow)"<test-name-fragment>"\
    (set_color normal)
    exit 1
end

# Make sure to install cargo-watch via `cargo install cargo-watch`.
# More info about cargo-watch: https://crates.io/crates/cargo-watch
cargo watch -x check -x "test $argv -- --show-output" -c -q
