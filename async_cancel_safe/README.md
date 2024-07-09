# Rust async in practice tokio::select!, actor pattern & cancel safety

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

Async cancellation safety:
- https://docs.rs/tokio/latest/tokio/macro.select.html#cancellation-safety
- https://www.youtube.com/watch?v=o2ob8zkeq2s&t=8353s
- https://stackoverflow.com/questions/74547855/how-to-assess-cancel-safety-of-select
- https://users.rust-lang.org/t/cancel-safety-in-async-and-tokio-select/92381/3

Async stream (for testing):
- https://github.com/r3bl-org/r3bl-open-core/blob/main/terminal_async/src/readline_impl/readline.rs#L621

## YT Video

- <https://youtu.be/cQq5i8J1ELg>

Chapters:
- 00:00:00 Introduction
- 00:00:53 Cancel safety with tokio::select!
- 00:01:25 Futures are stateful, be careful when dropping them
- 00:02:20 Create a new crate for live coding session
- 00:02:56 Required deps
- 00:05:33 Example 1 - pin and sleep (cancel unsafe)
- 00:20:41 Example 2 - pin and sleep (cancel safe)
- 00:30:00 Running into problems with Example 2
- 00:36:19 Debug Example 2 by running test in terminal
- 00:38:11 Continue debugging Example 2
- 00:41:24 Read docs to debug
- 00:42:09 Insight about sleep (timeout) vs interval (repeating ticks)
- 00:42:54 Fix the examples 1 and 2 with interval
- 00:45:53 Split example 1 into two examples (3 in total so far)
- 00:48:34 It compiles, vs it works 😂
- 00:49:00 VSCode insiders vs running test in terminal
- 00:50:42 Final example to show cancel unsafe code with async stream
- 00:51:30 Required imports for async stream generation, proxy for file or network stream
- 00:53:00 Test fixtures for async stream and generator
- 00:56:30 Example 4 - create the cancel unsafe future
- 00:57:36 Set up the race between reading from stream and timeout (using sleep)
- 01:01:49 Implement the cancel unsafe function
- 01:03:32 Problem with VSCode and Clippy
- 01:04:00 Back to the cancel unsafe function
- 01:06:00 Run the final example test and interpret the output
- 01:09:00 The future drop, which contains state that is lost
- 01:09:33 Further explanation of what is happening
- 01:11:16 Final explanation of the cancel unsafe code with lots of println output
- 01:16:00 Potential fixes to the cancel unsafe code
- 01:16:13 Outro
