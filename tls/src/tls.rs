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
//! | Function                                | Description                                         |
//! |-----------------------------------------|-----------------------------------------------------|
//! | [certificate_ops::client_load_ca_certs] | CA certificate.                                     |
//! | [tls_ops::client_tls_connect]           | Client code to connect to the server securely.      |
//!
//! # Server
//!
//! | Function                              | Description                                               |
//! |---------------------------------------|-----------------------------------------------------------|
//! | [certificate_ops::server_load_key]    | Private key.                                              |
//! | [certificate_ops::server_load_cert]   | Server certificate signed by CA certificate.              |
//! | [tls_ops::server_tls_accept]          | Server code to accept secure connections from the client. |

use miette::IntoDiagnostic;
use rustls::{
    pki_types::{CertificateDer, PrivatePkcs1KeyDer},
    RootCertStore,
};
use rustls_pemfile;
use rustls_pemfile::{read_one, Item};
use std::{io::BufReader, iter};

pub mod tls_ops {
    // TODO: impl this
    pub fn client_tls_connect() {
        todo!()
    }

    // TODO: impl this
    pub fn server_tls_accept() {
        todo!()
    }
}

pub mod certificate_ops {
    use super::*;

    /// This is the private key that the server uses.
    /// - It is in the `PKCS#1` format.
    /// - The [binary_data::SERVER_KEY_PEM] holds the contents of the `server-key.pem` file.
    pub fn server_load_key() -> miette::Result<PrivatePkcs1KeyDer<'static>> {
        if let Some(key) =
            certificate_ops::load_key_from_pem_data(binary_data::SERVER_KEY_PEM).pop()
        {
            Ok(key)
        } else {
            miette::bail!("No keys found in the server-key.pem file");
        }
    }

    /// - This is the server certificate that the server uses.
    /// - The [binary_data::SERVER_CERT_PEM] holds the contents of the `server.pem` file.
    pub fn server_load_cert() -> miette::Result<CertificateDer<'static>> {
        if let Some(cert) =
            certificate_ops::load_certs_from_pem_data(binary_data::SERVER_CERT_PEM).pop()
        {
            Ok(cert)
        } else {
            miette::bail!("No certificates found in the server.pem file");
        }
    }

    /// - This is the CA certificate that the client uses to verify the server certificate.
    /// - The [binary_data::CLIENT_CA_CERT_PEM] holds the contents of the `ca.pem` file.
    pub fn client_load_ca_certs() -> miette::Result<Vec<CertificateDer<'static>>> {
        let certs = certificate_ops::load_certs_from_pem_data(binary_data::CLIENT_CA_CERT_PEM);
        if certs.is_empty() {
            miette::bail!("No CA certificates found in the ca.pem file");
        }
        Ok(certs)
    }

    /// This function creates a [RootCertStore] that contains the CA certificates
    pub fn create_root_cert_store() -> miette::Result<RootCertStore> {
        let mut root_store = RootCertStore::empty();
        for cert in client_load_ca_certs()? {
            root_store.add(cert).into_diagnostic()?;
        }
        Ok(root_store)
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
    pub fn load_key_from_pem_data(key_data: &[u8]) -> Vec<PrivatePkcs1KeyDer<'static>> {
        let mut reader = BufReader::new(key_data);
        let mut return_value = vec![];
        for item in iter::from_fn(|| read_one(&mut reader).transpose()) {
            match item {
                Ok(Item::Pkcs1Key(key)) => {
                    return_value.push(key);
                }
                _ => continue,
            }
        }
        return_value
    }

    /// It is in the `DER-encoded X.509` format for certificates.
    pub fn load_certs_from_pem_data(pem_data: &[u8]) -> Vec<CertificateDer<'static>> {
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

/// These are generated by running the `gen_certs.fish` script in the `certs` directory.
pub mod binary_data {
    /// - Embed the `server-key.pem` file into the binary.
    /// - This is the private key that the server uses.
    /// - It is generated using the self-signed CA certificate.
    pub const SERVER_KEY_PEM: &[u8] = include_bytes!("../certs/generated/server-key.pem");

    /// - Embed the `server.pem` file into the binary.
    /// - This is the server certificate that the server uses.
    /// - It is generated using the self-signed CA certificate.
    pub const SERVER_CERT_PEM: &[u8] = include_bytes!("../certs/generated/server.pem");

    /// - Embed the `ca.pem` file into the binary.
    /// - This is the CA certificate that the client uses to verify the server
    ///   certificate.
    /// - It is generated using the self-signed CA certificate.
    pub const CLIENT_CA_CERT_PEM: &[u8] = include_bytes!("../certs/generated/ca.pem");
}
