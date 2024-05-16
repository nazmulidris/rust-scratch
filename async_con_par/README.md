# async_con_par

### Abstract

In Rust, you can write sequential code, and concurrent code:
- Sequential code, can be run sequentially, or in parallel (using `thread::spawn()`).
- Concurrent code that can be run on a single thread or multiple threads.

Concurrency is a way to structure code into separate tasks. This does not define the
resources on a machine that will be used to run or execute tasks.

Parallelism is a way to specify what resources (CPU cores, or threads) will be used on a
machine's operating system to run tasks.

These 2 concepts are not the same. They are related but not the same.

### What it is not

You can't simply attach `async` as a prefix to a function, when you define it, and
postfix `.await` when you call it. In fact, if you don't have at least one `.await` in
your async function body, then it probably doesn't need to be async. This project is a
deep dive into what async code is, what Rust Futures are, along with what async
Runtimes are. Along with some common patterns and anti-patterns when thinking in async
Rust.

### References

This project is an exploration of async concurrent and parallel programming in Rust.
The inspiration for this exploration is this blog post: [Tasks are the wrong
abstraction[(https://blog.yoshuawuyts.com/tasks-are-the-wrong-abstraction/#tasks-are-the-wrong-abstraction-for-concurrent-async-execution).

Here are some other interesting links:
1. [How futures work in runtimes w/ Context and Waker](https://gemini.google.com/app/7cdd1930f56e4b91)
2. [`epoll` (linux), vs `io_uring` (linux), vs `iocp` (windows)](https://gemini.google.com/app/8ee99f90784bd9e8)
3. [`tokio_uring` crate](https://docs.rs/tokio-uring/latest/tokio_uring/)
4. [`io_uring` paper](https://kernel.dk/io_uring.pdf)
5. [`futures_concurrency` crate, which works with any async runtime](https://docs.rs/futures-concurrency/latest/futures_concurrency/)
