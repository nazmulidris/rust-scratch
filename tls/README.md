# TLS with Tokio, Rust, and rustls

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Introduction](#introduction)
- [TLS primer](#tls-primer)
- [First, configure the certificates](#first-configure-the-certificates)
  - [Scripts](#scripts)
  - [Tools used by the scripts (CFSSL)](#tools-used-by-the-scripts-cfssl)
  - [Configuration files](#configuration-files)
  - [Run the scripts and generate the certificates](#run-the-scripts-and-generate-the-certificates)
    - [Examine the generated certificates](#examine-the-generated-certificates)
- [Second, write and run the code](#second-write-and-run-the-code)
- [References](#references)

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

It consists of both symmetric and asymmetric encryption algorithms. Here's a
brief overview of both:

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
- **Handshake**: The client and server perform a handshake to establish a secure connection.
  During this process:
  - The client and server agree on the TLS version and cipher suites to use.
  - The server presents its digital certificate, which contains its public key.
  - The client verifies the server's certificate against trusted Certificate Authorities
    (CAs).
  - The client generates a random session key, encrypts it with the server's public key,
    and sends it to the server.
- **Session Key**: Once the server receives the encrypted session key, it decrypts it using
  its private key. Both parties now have the same session key, which is used for symmetric
  encryption of the data transmitted during the session.
- **Data Transmission**: All data sent between the client and server is encrypted using the
  session key, ensuring confidentiality and integrity.

Rust has 2 main implementations for TLS:
1. `rustls`: A modern, safe, and fast TLS library written in Rust. This does not have any
   dependencies on OpenSSL, or any C code, or any OS specific code. It is a pure Rust
   implementation.
2. `native-tls`: A thin wrapper around the platform's native TLS implementation. It uses
   OpenSSL on Unix-like systems and SChannel on Windows.

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

### Configuration files

There are 3 JSON files that are used to generate the certificates:

- `ca-config.json`: The configuration for the CA.
- `ca-csr.json`: The Certificate Signing Request (CSR) for the CA.
- `server.json`: The configuration for the server certificate.

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

1. Generate root certificate (CA) and sign it. The `ca` string in the filenames
    comes from the `cfssl gencert ... | cfssljson -bare ca` command. If you change the
    string `ca` in the command, it will change the filenames that are produced.

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
| `DNS:localhost, IP Address:127.0.0.1`          | This is from `server.json`                   |

3. Finally verify the server certificate against the CA certificate:

```sh
openssl verify -CAfile generated/ca.pem generated/server.pem
```

If the certificate is valid, you will see the following output: `generated/server.pem: OK`

## Second, write and run the code

TK: Write this section.

## References

All of these links (video & repo) are related to the same project:

- This [video](https://www.youtube.com/watch?v=iqBXe80QaGw&list=WL&index=2&t=13s) goes
  over the process of setting up TLS with CFSSL.
- This [video](https://www.youtube.com/watch?v=eVuKCu6BMBQ&list=WL&index=6) goes over the
  process of writing Rust code using `tokio` and `rustls`.
- This
  [repo](https://github.com/dionysus-oss/netrusting/blob/c5364a2e31ef3871b8e968364c575f6f0d7cd8b8/rcat/README.md)
  has a good example of how to use `tokio` and `rustls` together.
