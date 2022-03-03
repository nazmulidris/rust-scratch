# address-book-with-redux

This is a CLI app that implements an immutable data structure that represents an address book.
Mutations to this address book must happen via a Redux store, and actions that are dispatched to it.
Along w/ a reducer function.

# Todo

- [ ] Create a simple CLI event loop that allows the user to interact with the address book
  - Use `readline` to get user input (from `r3bl_rs_utils`)
- [ ] Create a Redux store
- [ ] Create a Redux reducer
- [ ] Create an action framework
- [ ] Create an AddressBook strut and impl
- [ ] Create a CLI app that takes input to mutate this store and display the state to stdout
- [ ] Replace the use of `readline` with `rustyline`
- [ ] Create a better `readline` with my own implementation using `crossterm` (like `reedline`)
