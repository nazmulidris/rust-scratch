# rust-grep-cli

The app is an very basic implementation of grep.

> It is inspired by this simple CLI app from the
> [Rust book, ch 12](https://doc.rust-lang.org/book/ch12-00-an-io-project.html).

# Todo

- [x] Create a `grep` like command that works in both:
  - [x] detect `stdin` redirected mode vs "normal" mode (using `atty` crate)
  - [x] `stdin` redirected mode
    - [x] builder for args for search string and case-insensitive search
    - [x] colorize output for matches
  - [x] file reading mode
    - [x] builder for args for search string, file path, and case-insensitive search
    - [x] colorize output for matches

# Usage

To run the program, you can execute the `run.fish` script.
