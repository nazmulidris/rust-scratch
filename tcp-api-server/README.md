# tcp-api-server

```
░░░░░ ░░░░ ░░░░░   ░░░░░ ░░░░░ ░    ░░░░░ ░░░░ ░░░░░  ░░   ░ ░░░░ ░░░░░
  ░   ░    ░   ░   ░   ░ ░   ░ ░    ░     ░    ░   ░  ░░   ░ ░    ░   ░
  ░░  ░░   ░░░░░   ░░░░░ ░░░░░ ░░   ░░░░░ ░░░░ ░░░░░░ ░░  ░░ ░░░░ ░░░░░░
  ░░  ░░   ░░      ░░  ░ ░░    ░░      ░░ ░░   ░░   ░  ░  ░  ░░   ░░   ░
  ░░  ░░░░ ░░      ░░  ░ ░░    ░░   ░░░░░ ░░░░ ░░   ░  ░░░░  ░░░░ ░░   ░
```

<!-- Install app from: https://flathub.org/apps/io.github.nokse22.asciidraw -->
<!-- Get glyphs from: https://github.com/r3bl-org/r3bl-ts-utils/blob/main/src/tui-figures/symbols.ts -->

```
┌──────────────────┐ ┌────────────────────────────────────┐ ┌─────────────────────────────┐
│                  │ │                                    │ │                             │
│  CLIENT          │ │  TCP Protocol                      │ │  Server                     │
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

![image](./main_diagram.drawio.svg)


# Usage

## To see help for the command:

```sh
cargo run -- --help
```

## To run the server on the default port on localhost:

```sh
cargo run -- server
```

## To run the client on the default port on localhost:

```sh
cargo run -- client
```

## Automatically compile

You can also run this [`cargo-watch`](https://crates.io/crates/cargo-watch) command to
automatically recompile and run the server when the code changes:

```sh
cargo watch -x 'build'
```

Once the command above is running in a terminal, you can run the binary directly using
`target/debug/tcp-api-server` in another terminal. Simply replace the `cargo run --` part
of the commands above with `target/debug/tcp-api-server`.

> Do not use `cargo run` as it will recompile the code all over again (needlessly).