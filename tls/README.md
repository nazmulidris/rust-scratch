# TLS with Tokio, Rust, rustls and cfssl

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Detailed tutorial & live coding video of this crate](#detailed-tutorial--live-coding-video-of-this-crate)
- [Instructions to run the code](#instructions-to-run-the-code)
  - [1. Generate the certificate authority (CA) and server certificates](#1-generate-the-certificate-authority-ca-and-server-certificates)
  - [2. Run the server and client binaries](#2-run-the-server-and-client-binaries)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Detailed tutorial & live coding video of this crate

- Tutorial: <https://developerlife.com/2024/11/28/rust-tls-rustls>
- Video: <https://youtu.be/NeTZGyc9l7E>

## Instructions to run the code

### 1. Generate the certificate authority (CA) and server certificates

First, you have to generate the self signed certificate for the server and the certificate
authority (CA) certificate.

Run the following command to generate the certificates from the root folder of the project
(where the `Cargo.toml` file resides):

```bash
cargo run --bin gen-certs
```

This will generate the following files in `certs/generated` folder:

- Certificate Authority (CA) files:

  - `ca.csr` - Certificate Signing Request (CSR) for the CA
  - `ca-key.pem` - Private key for the CA
  - `ca.pem` - Certificate for the CA. **THIS GOES IN THE CLIENT BINARY**

- Server files:
  - `server.csr` - Certificate Signing Request (CSR) for the server
  - `server-key.pem` - Private key for the server **THIS GOES IN THE SERVER BINARY**
  - `server.pem` - Certificate for the server. **THIS GOES IN THE SERVER BINARY**

### 2. Run the server and client binaries

Now, you can run the server and client binaries using the following commands.

- These binaries actually pull in the certificates generated in the previous step from the
  `certs/generated` folder. They are current not baked into the binaries.
- They could be baked in if the 3 certificate files above are copied into the `src` folder
  and the `include_bytes!` macro is used to include them in the binaries.

```bash
cargo run --bin server
```

```bash
cargo run --bin client
```

You should see the program in action. It will simply display "one", "two", "three" on both
the server and client side. These messages are encrypted between the client and sever
processes using TLS.
