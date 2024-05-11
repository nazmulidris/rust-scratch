# tcp-api-server
<a id="markdown-tcp-api-server" name="tcp-api-server"></a>


```
░░░░░ ░░░░ ░░░░░   ░░░░░ ░░░░░ ░    ░░░░░ ░░░░ ░░░░░  ░░   ░ ░░░░ ░░░░░
  ░   ░    ░   ░   ░   ░ ░   ░ ░    ░     ░    ░   ░  ░░   ░ ░    ░   ░
  ░░  ░░   ░░░░░   ░░░░░ ░░░░░ ░░   ░░░░░ ░░░░ ░░░░░░ ░░  ░░ ░░░░ ░░░░░░
  ░░  ░░   ░░      ░░  ░ ░░    ░░      ░░ ░░   ░░   ░  ░  ░  ░░   ░░   ░
  ░░  ░░░░ ░░      ░░  ░ ░░    ░░   ░░░░░ ░░░░ ░░   ░  ░░░░  ░░░░ ░░   ░
```

<!-- Install app from: https://flathub.org/apps/io.github.nokse22.asciidraw -->
<!-- Get glyphs from: https://github.com/r3bl-org/r3bl-ts-utils/blob/main/src/tui-figures/symbols.ts -->

<!-- TOC -->

- [Mental models](#mental-models)
- [Tokio tracing usage](#tokio-tracing-usage)
- [Run the server and client](#run-the-server-and-client)
  - [Run Jaeger in Docker](#run-jaeger-in-docker)
  - [To see help for the command](#to-see-help-for-the-command)
  - [To run the server on the default port on localhost:](#to-run-the-server-on-the-default-port-on-localhost)
  - [To run the client on the default port on localhost](#to-run-the-client-on-the-default-port-on-localhost)
  - [Automatically compile](#automatically-compile)

<!-- /TOC -->

## Mental models
<a id="markdown-mental-models" name="mental-models"></a>


```
┌──────────────────┐ ┌────────────────────────────────────┐ ┌─────────────────────────────┐
│                  │ │                                    │ │                             │
│  Client          │ │  TCP Protocol                      │ │  Server                     │
│  - API           │ │  - bincode to handle enum          │ │  - expose API over TCP      │
│  - CLI (tuify)   │ │  - length first prefix (bigendian) │ │  - use kv for persistence   │
│                  │ │                                    │ │                             │
└──────────────────┘ └────────────────────────────────────┘ └─────────────────────────────┘
```

<!-- Source diagram:
https://asciiflow.com/#/share/eJzFk8FqwkAQhl9lmJOCuRQkNDeRHoQigXrcS0wmunSdDZuNJIggPkEPHvowPo1P0qT0oGQhEAld%2FsMsMzvf%2FgxzQI52hAEXSk1QRRUZDPAgsBQYvE79icCqjl78JrJU2voi8Ha59hY887iXvoZFCsGNrdb589pOuErPnaV3pPn74m25cpBW8xBCo62OteogfZDZk%2BkieTALFy5PHqwlxzohsBq2ESeKgLjYOUgeUJnpnH5b6QbafLNNql3ByBYyrcaPJEW8sVtIpcktZIZSWcJoLTfEiYx4fE8qasznHlJtICOTy9wSx%2FRIctgdZE63y6mX%2FmFDvgfeEDzi8QduiDC9)
-->

The following diagram illustrates what the "threading" mental model for the client and
server programs (in the same binary) look like:

![image](./main_diagram.drawio_1.svg)

Here's a further breakdown of the interactions between the client and the server. Unlike
HTTP, this protocol is not purely request and response. The client can send a message and
the server can send another message as a reply, but the server can also send messages to
the client at any time. Comparing this to the web, it would be akin to a websocket.

![image](./main_diagram.drawio_2.svg)

## Tokio tracing usage
<a id="markdown-tokio-tracing-usage" name="tokio-tracing-usage"></a>

Code:
- [`r3bl_terminal_async` tracing setup](https://github.com/r3bl-org/r3bl-open-core/blob/nazmulidris/otel/terminal_async/src/public_api/tracing_setup.rs)
- [`r3bl_terminal_async` jaeger & otel setup](https://github.com/r3bl-org/r3bl-open-core/blob/nazmulidris/otel/terminal_async/src/public_api/jaeger_setup.rs#L1)
- [use tracing setup in this project](https://github.com/nazmulidris/rust-scratch/tree/main/tcp-api-server)

Here's an example.

```rust
mod client_task {
    #[instrument(name = "caller", skip_all, fields(?client_id))]
    pub async fn entry_point(client_id: usize) {
        info!("entry point");
        handle_message(client_id, "foo").await;
    }

    #[instrument(name = "callee", skip_all, fields(%message))]
    pub async fn handle_message(client_id: usize, message: String) {
        info!("handling message");
    }
}
```

- You have to be careful about recording the same field multiple times, in an async call
  chain. In the example above, `client_task::entry_point()` is the entry point, and is the
  only function that should log the `?client_id`; `?` means debug. And not any other
  functions that it calls, like `handle_message()`.

  - When you call `entry_point()`, it will call `handle_message()`, and the span that is
    generated by `handle_message()` will have the `client_id` field added to it, because
    of the call chain. So the output of `info!("handling message")` will have the
    `client_id` included in it (for free). It will also have the `%message` field in it;
    `%` means display. You don't have to explicitly add either of these fields to the
    `info!()` call 🎉.
  - If you use the `client_id` field in multiple `#[instrument..]` attributes in functions
    (that are in the call chain), then this will show up multiple times in the log output
    (when using `info!`, `debug!`, etc) of the leaf function in the call chain. So when you
    see the same fields showing up multiple times in the output from `info!`, `debug!`, etc,
    then you know that you have to remove that field from the `#[instrument..]` attribute
    somewhere in the call chain (that the span covers).

- You have to be careful about how to use
  [`[#instrument]`](https://docs.rs/tracing/latest/tracing/attr.instrument.html) attribute
  with `tracing::Span::record`. You have to call
  `tracing::Span::current().record("foo","bar")` in the same function where the
  `#[instrument(fields(foo))]` attribute is used.

## Run the server and client
<a id="markdown-run-the-server-and-client" name="run-the-server-and-client"></a>

### Run Jaeger in Docker
<a id="markdown-run-jaeger-in-docker" name="run-jaeger-in-docker"></a>

Do the following before running the server or client.

1. Run `docker compose up -d` to start the Jaeger backend. Alternatively you can run
   `docker run -d -p16686:16686 -p4317:4317 -e COLLECTOR_OTLP_ENABLED=true
   jaegertracing/all-in-one:latest`
2. Open: <http://localhost:16686/search>
3. Run the server or client.
4. When you're done running the server or client, run `docker compose down` to stop the
   Jaeger backend.

> 1) You can use `docker stats` to see the currently running containers.
> 2) Here's a [full Rust
>    example](https://github.com/open-telemetry/opentelemetry-rust/blob/main/examples/tracing-jaeger/src/main.rs)
>    of using Jaeger tracing.

### To see help for the command
<a id="markdown-to-see-help-for-the-command" name="to-see-help-for-the-command"></a>

```sh
cargo run -- --help
```

### To run the server on the default port on localhost:
<a id="markdown-to-run-the-server-on-the-default-port-on-localhost%3A" name="to-run-the-server-on-the-default-port-on-localhost%3A"></a>

```sh
cargo run -- server
```

### To run the client on the default port on localhost
<a id="markdown-to-run-the-client-on-the-default-port-on-localhost" name="to-run-the-client-on-the-default-port-on-localhost"></a>

```sh
cargo run -- client
```

### Automatically compile
<a id="markdown-automatically-compile" name="automatically-compile"></a>

You can also run this [`cargo-watch`](https://crates.io/crates/cargo-watch) command to
automatically recompile and run the server when the code changes:

```sh
cargo watch -x 'build'
```

Once the command above is running in a terminal, you can run the binary directly using
`target/debug/tcp-api-server` in another terminal. Simply replace the `cargo run --` part
of the commands above with `target/debug/tcp-api-server`.

> There is no need to use `cargo run` as it will recompile the code all over again
> (needlessly).