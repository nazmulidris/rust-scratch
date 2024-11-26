# TLS with Tokio, Rust, rustls and cfssl

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Introduction](#introduction)
- [TLS primer](#tls-primer)
- [Rust and TLS primer](#rust-and-tls-primer)
- [First, create the certificates by running gen-certs.fish](#first-create-the-certificates-by-running-gen-certsfish)
  - [Tools used by the scripts (CFSSL)](#tools-used-by-the-scripts-cfssl)
  - [Configuration files deep dive](#configuration-files-deep-dive)
  - [Run the scripts and generate the certificates](#run-the-scripts-and-generate-the-certificates)
    - [Examine the generated certificates](#examine-the-generated-certificates)
- [Second, write and run the code](#second-write-and-run-the-code)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Introduction

This repo contains code for a simple server and client program written in Rust that
communicate over TLS using the `tokio` and `rustls` libraries.

- TLS is used to secure the communication between the server and client.
- It is an added layer on top of the TCP connection.

## TLS primer

TLS is a cryptographic protocol designed to provide secure communication over a computer
network. It ensures:

- Confidentiality: Data is encrypted so that only the intended recipient can read it.
- Integrity: Data cannot be altered without detection.
- Authentication: The identities of the parties involved can be verified.

It consists of both symmetric and asymmetric encryption algorithms. Here's a brief
overview of both:

**Symmetric Encryption:**

- Definition: Uses the same key for both encryption and decryption.
- Examples: AES (Advanced Encryption Standard), DES (Data Encryption Standard).
- Benefits:
  - Faster than asymmetric encryption.
  - Suitable for encrypting large amounts of data.
- Drawbacks:
  - Key distribution can be a challenge; both parties must securely share the key. So
    sharing the key between both parties can either happen out of band, or using some
    other mechanism (like asymmetric encryption).

**Asymmetric Encryption:**

- Definition: Uses a pair of keys (public and private) for encryption and decryption.
- Examples: RSA, ECC (Elliptic Curve Cryptography).
- Benefits:
  - Solves the key distribution problem; the public key can be shared openly.
  - Provides authentication through digital signatures.
- Drawbacks:
  - Slower than symmetric encryption.
  - Not suitable for encrypting large amounts of data directly.

TLS uses a combination of both symmetric and asymmetric encryption. It uses asymmetric
encryption to establish a secure connection and symmetric encryption to encrypt the data
transferred over the connection.

Additionally the following are required to make the communication secure between the
client and server:

1. The client needs to have the CA certificate in case you are using self-signed
   certificates.
2. The server needs to have both the server certificate and the private key.

Here's an overview of how TLS works:

- **Handshake**: The client and server perform a handshake to establish a secure
  connection. During this process:
  - The client and server agree on the TLS version and cipher suites to use.
  - The server presents its digital certificate, which contains its public key.
  - The client verifies the server's certificate against trusted Certificate Authorities
    (CAs).
  - The client generates a random session key, encrypts it with the server's public key,
    and sends it to the server.
- **Session Key**: Once the server receives the encrypted session key, it decrypts it
  using its private key. Both parties now have the same session key, which is used for
  symmetric encryption of the data transmitted during the session.
- **Data Transmission**: All data sent between the client and server is encrypted using
  the session key, ensuring confidentiality and integrity.

## Rust and TLS primer

Now that we know more about TLS, how do we access it in Rust? Rust has 2 main
implementations for TLS:

1. [`rustls`](https://docs.rs/rustls/latest/rustls/): A modern, safe, and fast TLS library
   written in Rust. This does not have any dependencies on OpenSSL, or any C code, or any
   OS specific code. It is a pure Rust implementation.

   - This [video](https://www.youtube.com/watch?v=eVuKCu6BMBQ&list=WL&index=6) goes over
     the process of writing Rust code using `tokio` and `rustls`.
   - This
     [repo](https://github.com/dionysus-oss/netrusting/blob/c5364a2e31ef3871b8e968364c575f6f0d7cd8b8/rcat/README.md)
     has a good example of how to use `tokio` and `rustls` together.

2. [`native-tls`](https://docs.rs/tokio-native-tls/latest/tokio_native_tls/): A thin
   wrapper around the platform's native TLS implementation. It uses OpenSSL on Unix-like
   systems and SChannel on Windows.

## First, create the certificates by running gen-certs.fish

All the scripts and certificate related files are in the `certs` folder:

1. The main script is `gen-certs.fish`. It generates the CA and server certificates. It
   also uses the script below to get the CFSSL binaries.
2. The `get-cfssl-binaries.fish` script downloads the CFSSL binaries if needed. If they
   are already downloaded, it does nothing.

### Tools used by the scripts (CFSSL)

- The [CFSSL](https://github.com/cloudflare/cfssl) tool is used to generate the
  certificates.
- Learn more about the tool in this
  [blog post](https://blog.cloudflare.com/introducing-cfssl/).
- You can get the prebuilt binaries [here](https://github.com/cloudflare/cfssl/releases).
- This [video](https://www.youtube.com/watch?v=iqBXe80QaGw&list=WL&index=2&t=13s) goes
  over the process of setting up TLS with CFSSL.

### Configuration files deep dive

There are 3 JSON files that are used to generate the certificates:

**`ca-config.json`**: The configuration for the CA.

- The main node is `signing` which has the `profiles` node. You can have multiple
  profiles. In this case, I create a single profile named `server`, which is a name I just
  made up.
  - The node named `server`, which is a made up name of a profile, is used to generate the
    server certificate. This is a name that I created, it is not a reserved keyword, it
    has no special meaning. It is used in the
    `cfssl gencert ... -profile=server server.json` command and used to tie all the
    generated files together.
    - The `expiry` node sets the expiration date for a certificate. I changed it 10 years
      or `87600h`.
    - The `usages` node sets the key usage for the certificate. I set it to `signing`,
      `key encipherment`, `server auth`, and `client auth`.
  - Here's an example:
  ```json
  {
    "signing": {
      "default": {
        "expiry": "87600h"
      },
      "profiles": {
        "server": {
          "expiry": "87600h",
          "usages": ["signing", "key encipherment", "server auth"]
        }
      }
    }
  }
  ```

**`server.json`**: The configuration for the server certificate. This is related to the
`server` profile above. The CA will sign the server certificate using the `server`
profile.

- The `CN` node is the Common Name for this certificate. I set it to `server`. This has no
  special meaning. It is set to ensure that the `cfssl gencert -ca ca.pem ...` commands to
  generate the certificates work and can find the information related to the `server`,
  which matches the profile name.
- The `key` node sets the key size and type. I set it to `2048` bits and `rsa`. This is
  important.
- The `hosts` node sets the DNS names and IP addresses for the certificate. This is really
  important. The client will use a `ServerName` in Rust code to connect to the server.
  That name must match whatever is in the `hosts` array. You can just add another name
  there which can be parsed as a DNS name or an IP address. In my case, I have `localhost`
  and `r3bl.com` (which is just made up). However, in the Rust client code to connect to
  the server, I can create a
  [`ServerName`](https://docs.rs/rustls-pki-types/latest/rustls_pki_types/enum.ServerName.html)
  using either `"localhost"` or `"r3bl.com"`.
- Here's an example:
  ```json
  {
    "CN": "server",
    "hosts": ["localhost", "r3bl.com"],
    "key": {
      "algo": "rsa",
      "size": 2048
    },
    "names": [
      {
        "C": "US",
        "ST": "Texas",
        "L": "Austin"
      }
    ]
  }
  ```

**`ca-csr.json`**: The Certificate Signing Request (CSR) for the CA.

- The `CN` node is the Common Name for the CA. I set it to `ca`. This has no special
  meaning. It is just to make sure that the `cfssl gencert -initca ca-csr.json` commands
  to generate the certificates work and can find the information related to the CA.
- The `key` node sets the key size and type. I set it to `2048` bits and `rsa`. This is
  important.
- Here's an example:
  ```json
  {
    "CN": "ca",
    "key": {
      "algo": "rsa",
      "size": 2048
    },
    "names": [
      {
        "C": "US",
        "ST": "Texas",
        "L": "Austin"
      }
    ]
  }
  ```

Each of these files are modified from some default values to the desired values. They all
started life using the following commands:

- `./cfssl print-defaults config > ca-config.json`
- `./cfssl print-defaults csr > ca-csr.json`
- `./cfssl print-defaults csr > server.json`

### Run the scripts and generate the certificates

Run the following commands to generate the certificates in the `certs/generated` folder:

```sh
cd certs
./gen-certs.fish
```

Running this script will generate the following files:

1. Generate root certificate (CA) and sign it. The `ca` string in the filenames comes from
   the `cfssl gencert ... | cfssljson -bare ca` command. If you change the string `ca` in
   the command, it will change the filenames that are produced.

| File         | Description                              |
| ------------ | ---------------------------------------- |
| `ca.csr`     | Certificate signing request              |
| `ca-key.pem` | Private key                              |
| `ca.pem`     | Public key; used in the Rust client code |

2. Generate server certificate (and private key) and sign it with the CA. The `server`
   string in the filenames comes from the `cfssl gencert ... | cfssljson -bare server`
   command. If you change the string `server` in the command, it will change the filenames
   that are produced.

| File             | Description                               |
| ---------------- | ----------------------------------------- |
| `server.csr`     | Certificate signing request               |
| `server-key.pem` | Private key; used in the Rust server code |
| `server.pem`     | Public key; used in the Rust server code  |

#### Examine the generated certificates

1. Look in the `certs/generated/` folder to see the generated certificates. You can
   examine them using the `openssl` command:

```sh
openssl x509 -noout -text -in generated/ca.pem
```

Look for the following lines which confirm that this is a CA certificate, and some other
configuration properties provided in the `ca-config.json` file:

| Field                                  | Description                                 |
| -------------------------------------- | ------------------------------------------- |
| `Issuer: C=US, ST=TX, L=Austin, CN=ca` | The CA's own details, from `ca-config.json` |
| `Not After: ...`                       | Expiration date                             |
| `Public-Key: (2048 bit)`               | Key size and type from `ca-csr.json`        |
| `CA:TRUE`                              | This is a CA (root certificate)             |

2. Look in the `certs/generated` folder to see the server certificates. You can examine
   them using the `openssl` command:

```sh
openssl x509 -noout -text -in generated/server.pem
```

Look for the following lines which confirm that this is a server certificate, and some
other configuration properties provided in the `server.json` file:

| Field                                          | Description                                  |
| ---------------------------------------------- | -------------------------------------------- |
| `Issuer: C=US, ST=Texas, L=Austin, CN=ca`      | Issued by the CA above                       |
| `Subject: C=US, ST=Texas, L=Austin, CN=server` | The server's own details                     |
| `Not After : ...`                              | Expiration date                              |
| `CA:FALSE`                                     | Not a root certificate                       |
| `TLS Web Server Authentication`                | Extended Key Usage for server authentication |
| `DNS:localhost, IP Address:127.0.0.1`          | This is from `server.json`. The Rust client code uses this in `ServerName` to make a TLS connection |

3. Finally verify the server certificate against the CA certificate:

```sh
openssl verify -CAfile generated/ca.pem generated/server.pem
```

If the certificate is valid, you will see the following output: `generated/server.pem: OK`

## Second, write and run the code

Once the certificates are generated, the next step is to write the server and client code.
Here's the mental model for doing this.

- **Client code**

  - Certificate concerns:

    - The client code will need to load the root certificate store, inside of which will
      reside the CA (certificate authority) certificate chain, that we have generated (the
      `ca.pem` file).
    - The client will also need to know the server's hostname, which is used to verify the
      server's certificate. This has to match the `hosts` entry in the `server.json`
      config file. This entry has to be in the form of a `ServerName` in the Rust code,
      which is a DNS or IP address parsable format.

  - Code concerns:
    - The certificate and key files above is used to generate a `ClientConfig` struct,
      from the `rustls` crate. It is then used to create a `TlsConnector` struct.
    - The unsecure connection of type `TcpStream` will be created as per usual using
      `TcpStream::connect()`. However, this will then be wrapped in a `TlsConnector` which
      will make it a secure connection. The reader and writer halves are split from this
      `TlsStream` struct. And the reader and writer halves are used as per usual.

- **Server code**

  - Certificate concerns:

    - The server code will need to load the server's certificate and private key, which we
      have generated (the `server.pem` and `server-key.pem` files).
      - This server certificate is signed by the CA certificate. Since we are using
        self-signed certificates, only the client will need to load the CA certificate to
        verify the server certificate. And not the server.
        - This is because the server is self-signed and doesn't need to verify any
          incoming certificates.
        - If we weren't using self-signed certificates, the client would just have to load
          the root certificate store that's available publicly (like Mozilla root
          certificates).
      - The server will not need to load the root certificate store, inside of which will
        reside the CA certificate chain, that we have generated (the `ca.pem` file).

  - Code concerns:
    - The certificate and key files above are used to generate a `ServerConfig` struct,
      from the `rustls` crate. It is then used to create a `TlsAcceptor` struct.
    - The server will create a `TcpListener` and accept incoming connections. Each
      connection will be wrapped in a `TlsAcceptor` which will make it a secure
      connection. The reader and writer halves are split from this `TlsStream` struct. And
      the reader and writer halves are used as per usual.

Here's some more information about mapping the Rust code to the TLS files:

- [Rust code using `rustls` and TLS certificate & key files](https://gemini.google.com/app/6f8efc1d6a468cbf)

For details on the actual, code, here are some files from the `tls` repo:

- [`client.rs`](https://github.com/nazmulidris/rust-scratch/blob/main/tls/src/bin/client.rs)
- [`server.rs`](https://github.com/nazmulidris/rust-scratch/blob/main/tls/src/bin/server.rs)
- [`tls.rs`](https://github.com/nazmulidris/rust-scratch/blob/main/tls/src/tls.rs)

Here are the files for the TLS configuration and certificate generation:

- [`certs/config`](https://github.com/nazmulidris/rust-scratch/tree/main/tls/certs/config)
- [`fish` scripts to generate the certificates](https://github.com/nazmulidris/rust-scratch/tree/main/tls/certs)
