# tokio-uring

## References
- https://tokio.rs/tokio/tutorial
- https://users.rust-lang.org/t/tokio-copy-slower-than-std-io-copy/111242
- https://tokio.rs/blog/2021-07-tokio-uring
- https://unixism.net/loti/what_is_io_uring.html#the-mental-model
- https://github.com/tokio-rs/tokio-uring
- https://docs.rs/tokio-uring/latest/tokio_uring/net/struct.TcpStream.html
- https://www.scylladb.com/2020/05/05/how-io_uring-and-ebpf-will-revolutionize-programming-in-linux/0/
- https://www.datadoghq.com/blog/engineering/introducing-glommio/
- https://lore.kernel.org/io-uring/4af91b50-4a9c-8a16-9470-a51430bd7733@kernel.dk/T/#u

## Video

Timecodes:
00:01 Don't use Tokio for file IO
00:02 What is tokio-uring and Linux io-uring?
00:03 What is Linux io_uring? What is the mental model?
00:06 Brief history of Linux IO syscalls
00:08 Overview of what we will build in this video
00:10 Create a new crate and add deps
00:13 Sub out the 2 binaries (readfile.rs, socketserver.rs)
00:14 Read a file using tokio_uring (readfile.rs)
00:19 Formatting stdout output with color
00:21 Using tokio_uring::fs::File and read_at
00:26 Shared ring buffers between the kernel and user space, io_uring can be a zero-copy system
00:29 Format the file bytes as a string and colorize the output
00:31 Can't mix #[tokio::main] with tokio_uring::start
00:32 Create an echo TCP server using tokio_uring (socketserver.rs)
00:33
00:36 Run TcpListener::bind(addr) and check for port availability
00:40 Test check port availability
00:44 Main loop to accept connections and shutdown the server using CancellationToken
00:46 Use tokio::select! and tokio_uring::spawn
00:50 Vec of abort handles for server shutdown
00:53 Write the echo server code
01:04 CtrlC handler to shutdown the server
01:09 Intermix tokio runtime with tokio_uring::start (runtime)
01:13 Run tasks in the tokio runtime using tokio::spawn


## Command to stitch videos into one

```bash
ffmpeg -f concat -i inputs.txt -c copy rust_tokio_uring_exploration.mp4
```

Here's what `inputs.txt` looks like:

```
file 'tokio-uring-1.mp4'
file 'tokio-uring-2.mp4'
file 'tokio-uring-3.mp4'
```

[More on ffmpeg concat](https://trac.ffmpeg.org/wiki/Concatenate)