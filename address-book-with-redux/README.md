# address-book-with-redux

This is a CLI app that implements an immutable data structure that represents an address book.
Mutations to this address book must happen via a Redux store, and actions that are dispatched to it.
Along w/ a reducer function.

# Todo

Basic

- [x] Create simple CLI event loop that allows the user to interact with the address book
  - [x] Use `readline` to get user input (from `r3bl_rs_utils`)
- [x] Create action enum
- [x] Create state & address book data model
  - [ ] Add search term support?
- [x] Create reducer function
- [x] Create Redux store
  - [x] Move to `lib.rs`
  - [ ] Add support for middleware
- [x] Create a simple TUI that takes input to mutate this store and display the state to stdout

Advanced

- [ ] Sync this address book data w/ Google sheet (use the one tutorial for Rust)
- [ ] Publish Redux store to `r3bl_rs_utils`
- [ ] Replace the use of `readline` with `rustyline`
- [ ] Create a better `readline` with my own implementation using `crossterm` (like `reedline`)

# References

- [Tutorial on implementing Redux in Rust](https://betterprogramming.pub/redux-in-rust-d622822085fe)
