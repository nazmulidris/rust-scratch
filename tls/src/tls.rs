/*
 *   Copyright (c) 2024 Nazmul Idris
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

//! This module contains the TLS operations that are used by the client and server.
//!
//! # Client
//!
//! | Function                                             | Description                                         |
//! |------------------------------------------------------|-----------------------------------------------------|
//! | [root_cert_store_ops::client_create_root_cert_store] | CA certificate and root store.                      |
//! | [tls_ops::try_create_client_tls_connector]           | Client code to connect to the server securely.      |
//!
//! # Server
//!
//! | Function                                         | Description                                               |
//! |--------------------------------------------------|-----------------------------------------------------------|
//! | [key_ops::server_load_single_private_key]        | Private key.                                              |
//! | [certificate_ops::server_load_server_cert_chain] | Server certificate signed by CA certificate.              |
//! | [tls_ops::try_create_server_tls_acceptor]        | Server code to accept secure connections from the client. |

use crate::tls;
use miette::IntoDiagnostic as _;
use r3bl_core::ok;
use rustls::{
    pki_types::{CertificateDer, PrivateKeyDer},
    ClientConfig, RootCertStore, ServerConfig,
};
use rustls_pemfile::{self, read_one, Item};
use std::fs;
use std::sync::OnceLock;
use std::{io::BufReader, iter, sync::Arc};
use tokio_rustls::{TlsAcceptor /* server */, TlsConnector /* client */};

pub mod tls_ops {
    use super::*;

    /// Try to create a [tokio_rustls::TlsConnector] that can be used by your client to
    /// connect to the server securely.
    ///
    /// 1. Typically you might use [tokio::net::TcpStream::connect] to connect to the
    ///    server and get a "insecure" [tokio::net::TcpStream].
    /// 2. Instead use the [tokio_rustls::TlsConnector] (created by this function) to
    ///    convert that "insecure" stream into a "secure" stream. And then use that
    ///    "secure" stream to communicate with the server.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crate::tls::tls_ops::try_create_client_tls_connector;
    /// async fn client() {
    ///     // Typical code to connect to the server insecurely.
    ///     let (host, port) = ("localhost", 8080);
    ///     let addr = format!("{}:{}", host, port);
    ///     let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    ///     // Use the TlsConnector to connect to the server securely.
    ///     let tls_connector = try_create_client_tls_connector().unwrap();
    ///     let server_cert_cn = "server";
    ///     let server_name = rustls::pki_types::ServerName::try_from(server_cert_cn).unwrap();
    ///     let secure_stream = tls_connector.connect(server_name, stream).await.unwrap();
    ///     unimplemented!("Use the secure_stream to communicate with the server");
    /// }
    /// ```
    pub fn try_create_client_tls_connector() -> miette::Result<TlsConnector> {
        let root_cert_store = root_cert_store_ops::client_create_root_cert_store()?;
        let client_config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        let client_config = Arc::new(client_config);
        let tls_connector = TlsConnector::from(client_config);
        ok!(tls_connector)
    }

    /// Try to create a [tokio_rustls::TlsAcceptor] that can be used by your server to
    /// accept secure connections from the client.
    ///
    /// 1. Typically you might use [tokio::net::TcpListener::bind] to accept connections
    ///    and get a "insecure" [tokio::net::TcpStream].
    /// 2. Instead use the [tokio_rustls::TlsAcceptor] (created by this function) to
    ///    convert that "insecure" stream into a "secure" stream. And then use that
    ///    "secure" stream to communicate with your clients.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crate::tls::tls_ops::try_create_server_tls_acceptor;
    /// async fn server() {
    ///     // Typical code to accept connections insecurely.
    ///     let (host, port) = ("localhost", 8080);
    ///     let addr = format!("{}:{}", host, port);
    ///     let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    ///     let (stream, _) = listener.accept().await.unwrap(); // Blocking call.
    ///     // Use the TlsAcceptor to accept secure connections.
    ///     let tls_acceptor = try_create_server_tls_acceptor().unwrap();
    ///     let secure_stream = tls_acceptor.accept(stream).await.unwrap();
    ///     unimplemented!("Use the secure_stream to communicate with the client");
    /// }
    /// ```
    pub fn try_create_server_tls_acceptor() -> miette::Result<TlsAcceptor> {
        let server_cert_chain = tls::certificate_ops::server_load_server_cert_chain()?;
        let server_private_key = tls::key_ops::server_load_single_private_key()?;
        let server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(server_cert_chain, server_private_key)
            .into_diagnostic()?;
        let server_config = Arc::new(server_config);
        let tls_acceptor = TlsAcceptor::from(server_config);
        ok!(tls_acceptor)
    }
}

pub mod root_cert_store_ops {
    use super::*;

    /// This function creates a [RootCertStore] that contains the CA certificates
    pub fn client_create_root_cert_store() -> miette::Result<RootCertStore> {
        let mut root_cert_store = RootCertStore::empty();
        for cert in certificate_ops::client_load_ca_cert_chain()? {
            root_cert_store.add(cert).into_diagnostic()?;
        }
        ok!(root_cert_store)
    }
}

pub mod key_ops {
    use super::*;

    /// `server-key.pem` -> `PKCS#1` format for private keys. Private key for server.
    ///
    /// This is the private key that the server uses.
    /// - It is in the `PKCS#1` format.
    /// - The [binary_data::get_server_key_pem_for_server_binary] holds the contents of
    ///   the `server-key.pem` file.
    pub fn server_load_single_private_key() -> miette::Result<PrivateKeyDer<'static>> {
        if let Some(key) =
            load_private_key_from_pem_data(binary_data::get_server_key_pem_for_server_binary())
                .pop()
        {
            ok!(key)
        } else {
            miette::bail!(
                "No keys found in the {} file",
                binary_data::SERVER_KEY_PEM_FILENAME
            );
        }
    }

    /// Here are a few ways to determine what the PEM file contains:
    ///
    /// 1. Look inside the `PEM` file to see what the header is, eg:
    ///    ```text
    ///    -----BEGIN RSA PRIVATE KEY-----
    ///    ```
    /// 2. You can also use the following command get the type:
    ///    ```sh
    ///    openssl rsa -in generated/server-key.pem -check
    ///    ```
    /// 3. You can also use the following command to get the type:
    ///    ```sh
    ///    openssl rsa -in generated/server-key.pem -text -noout
    ///    ```
    ///
    /// API Docs: <https://docs.rs/rustls-pemfile/latest/rustls_pemfile/>
    fn load_private_key_from_pem_data(key_data: &[u8]) -> Vec<PrivateKeyDer<'static>> {
        let mut reader = BufReader::new(key_data);
        let mut return_keys: Vec<PrivateKeyDer> = vec![];
        for item in iter::from_fn(|| read_one(&mut reader).transpose()) {
            match item {
                Ok(Item::Pkcs1Key(key)) => {
                    return_keys.push(PrivateKeyDer::Pkcs1(key));
                }
                _ => continue,
            }
        }
        return_keys
    }
}

pub mod certificate_ops {
    use super::*;

    /// `server.pem` -> `X.509` format for certificates. Certificate for server, issued by
    /// CA.
    ///
    /// - This is the server certificate that the server uses.
    /// - The [binary_data::get_server_cert_pem_for_server_binary] holds the contents of
    ///   the `server.pem` file.
    pub fn server_load_server_cert_chain() -> miette::Result<Vec<CertificateDer<'static>>> {
        let return_certs = certificate_ops::load_certs_from_pem_data(
            binary_data::get_server_cert_pem_for_server_binary(),
        );
        if return_certs.is_empty() {
            miette::bail!(
                "No certificates found in the {} file",
                binary_data::SERVER_CERT_PEM_FILENAME
            );
        }
        ok!(return_certs)
    }

    /// `ca.pem` -> `X.509` format for certificates. Certificate for CA, issued by itself.
    ///
    /// - This is the CA certificate that the client uses to verify the server
    ///   certificate.
    /// - The [binary_data::get_ca_cert_pem_for_client_binary] holds the contents of the
    ///   `ca.pem` file.
    pub fn client_load_ca_cert_chain() -> miette::Result<Vec<CertificateDer<'static>>> {
        let return_certs = certificate_ops::load_certs_from_pem_data(
            binary_data::get_ca_cert_pem_for_client_binary(),
        );
        if return_certs.is_empty() {
            miette::bail!(
                "No CA certificates found in the {} file",
                binary_data::CA_CERT_PEM_FILENAME
            );
        }
        ok!(return_certs)
    }

    /// It is in the `PEM-encoded X.509` format for certificates. While the data is from a
    /// PEM encoded file, `rustls` loads this into a the [CertificateDer] struct. PEM file
    /// format is human readable and Base64 encoded. DER format is binary.
    fn load_certs_from_pem_data(pem_data: &[u8]) -> Vec<CertificateDer<'static>> {
        let mut reader = BufReader::new(pem_data);
        let mut return_value = vec![];
        for item in iter::from_fn(|| read_one(&mut reader).transpose()) {
            match item {
                Ok(Item::X509Certificate(cert)) => {
                    return_value.push(cert);
                }
                _ => continue,
            }
        }
        return_value
    }
}

/// These are generated by running either:
/// 1. `gen_certs.fish` script in the `certs` directory.
/// 2. `cargo run --bin gen_certs` in the project root directory.
///
/// These files are not loaded statically at compile time (through the use of
/// [include_bytes!]). Rather they are loaded at runtime. This is due to the fact that
/// when using the `gen_certs` binary:
/// 1. The files don't exist until the binary is run.
/// 2. And the binary can't be built until the files exist. So it's a chicken and egg
///    problem.
pub mod binary_data {
    use super::*;

    /// - Embed the `server-key.pem` file into the binary.
    /// - This is the private key that the **SERVER** uses.
    /// - It is generated using the self-signed CA certificate.
    static SERVER_KEY_PEM: OnceLock<Vec<u8>> = OnceLock::new();
    /// For error messages and file loading.
    pub const SERVER_KEY_PEM_FILENAME: &str = "certs/generated/server-key.pem";
    /// Actually load the bytes at runtime (once).
    pub fn get_server_key_pem_for_server_binary() -> &'static Vec<u8> {
        SERVER_KEY_PEM.get_or_init(|| {
            fs::read(SERVER_KEY_PEM_FILENAME).expect("Failed to read server-key.pem")
        })
    }

    /// - Embed the `server.pem` file into the binary.
    /// - This is the server certificate that the **SERVER** uses.
    /// - It is generated using the self-signed CA certificate.
    static SERVER_CERT_PEM: OnceLock<Vec<u8>> = OnceLock::new();
    /// For error messages and file loading.
    pub const SERVER_CERT_PEM_FILENAME: &str = "certs/generated/server.pem";
    /// Actually load the bytes at runtime (once).
    pub fn get_server_cert_pem_for_server_binary() -> &'static Vec<u8> {
        SERVER_CERT_PEM
            .get_or_init(|| fs::read(SERVER_CERT_PEM_FILENAME).expect("Failed to read server.pem"))
    }

    /// - Embed the `ca.pem` file into the binary.
    /// - This is the CA certificate that the **CLIENT** uses to verify the server
    ///   certificate.
    /// - It is generated using the self-signed CA certificate.
    static CA_CERT_PEM: OnceLock<Vec<u8>> = OnceLock::new();
    /// For error messages and file loading.
    pub const CA_CERT_PEM_FILENAME: &str = "certs/generated/ca.pem";
    /// Actually load the bytes at runtime (once).
    pub fn get_ca_cert_pem_for_client_binary() -> &'static Vec<u8> {
        CA_CERT_PEM.get_or_init(|| fs::read(CA_CERT_PEM_FILENAME).expect("Failed to read ca.pem"))
    }
}
