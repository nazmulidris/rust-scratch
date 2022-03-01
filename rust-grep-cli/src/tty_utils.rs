#![allow(dead_code)]

/// If you run `echo "test" | cargo run` the following will return true.
pub fn is_stdin_piped() -> bool {
  atty::isnt(atty::Stream::Stdin)
}

/// If you run `cargo run | grep foo` the following will return true.
pub fn is_stdout_piped() -> bool {
  atty::isnt(atty::Stream::Stdout)
}

/// If you run `cargo run` the following will return true.
pub fn is_tty() -> bool {
  atty::is(atty::Stream::Stdin)
    && atty::is(atty::Stream::Stdout)
    && atty::is(atty::Stream::Stderr)
}
