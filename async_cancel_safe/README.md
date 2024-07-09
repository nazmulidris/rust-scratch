# Rust async in practice video: `tokio::select!`, actor, cancel safety, w/ `tcp-api-server`

## Notes

### Example 1 - pin and sleep

- `sleep()` in a `select!` block doesn't do what you think, use `Interval` instead.

### Example 2 - unsafe code

- Come up with an example to demonstrate cancel unsafety (unintentionally dropping some
  data stored in the future struct that is lost, to interval, when raced in a loop). It
  should have the following branches in an infinite loop:
  1. `tokio::pin!(sleep(..))`.
  2. shutdown signal handler.
  3. cancel unsafe code.

## References

- https://docs.rs/tokio/latest/tokio/macro.select.html#cancellation-safety
- https://www.youtube.com/watch?v=o2ob8zkeq2s&t=8353s
- https://stackoverflow.com/questions/74547855/how-to-assess-cancel-safety-of-select
- https://users.rust-lang.org/t/cancel-safety-in-async-and-tokio-select/92381/3