# async_con_par

## async_con_par

This project is an exploration of async concurrent and parallel programming in Rust. The inspiration
for this exploration is this blog post: [Tasks are the wrong
abstraction[(https://blog.yoshuawuyts.com/tasks-are-the-wrong-abstraction/#tasks-are-the-wrong-abstraction-for-concurrent-async-execution).

Here are some other interesting links:
1. [How futures work in runtimes w/ Context and Waker](https://gemini.google.com/app/7cdd1930f56e4b91)
2. [`epoll` (linux), vs `io_uring` (linux), vs `iocp` (windows)](https://gemini.google.com/app/8ee99f90784bd9e8)
3. [`tokio_uring` crate](https://docs.rs/tokio-uring/latest/tokio_uring/)
4. [`io_uring` paper](https://kernel.dk/io_uring.pdf)
5. [`futures_concurrency` crate, which works with any async runtime](https://docs.rs/futures-concurrency/latest/futures_concurrency/)

### Abstract

You can write sequential code, that can be run sequentially, or in parallel (using
`thread::spawn()`). And there is concurrent code that can be run on a single thread or
multiple threads.

Concurrency is a way to structure code into separate tasks. This does not define the
resources on a machine that will be used to run or execute tasks.

Parallelism is a way to specify what resources (CPU cores, or threads) will be used on a
machine's operating system to run tasks.

These 2 concepts are not the same. They are related but not the same.

### Concurrency on a single thread

If you have async code, you can use a `LocalSet` to run the async code, in different
tasks, on a single thread. This ensures that any data that you have to pass between these
tasks can be `!Send`. Instead of wrapping the shared data in a `Arc` or `Arc<Mutex>`, you
can just wrap it in an `Rc`.

- Look at the [local_set_tests] for more info.
- Read the docs [here](https://docs.rs/tokio/latest/tokio/task/struct.LocalSet.html).

### Concurrency on multiple threads

TK: write more on this
