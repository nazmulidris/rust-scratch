# Generating the CA Certificate and Key (for the client):

```sh
# Generate a new private key for the CA
openssl genrsa -out ca-key.pem 2048

# Create a self-signed certificate for the CA
openssl req -x509 -new -nodes -key ca-key.pem -out ca-cert.pem -days 365

# Convert the CA certificate to DER format
openssl x509 -in ca-cert.pem -outform DER -out ca.der
```

# Generating the Server Certificate and Key (for the server):

```sh
# Generate a private key for the server
openssl genrsa -out server-key.pem 2048

# Create a Certificate Signing Request (CSR) for the server
openssl req -new -key server-key.pem -out server.csr

# Sign the CSR using the CA to create the server certificate
openssl x509 -req -in server.csr -signkey ca-key.pem -out server-cert.pem -CA ca-cert.pem -CAkey ca-key.pem -days 365

# Convert the server certificate and key to DER format
openssl x509 -in server-cert.pem -outform DER -out cert.der
openssl pkcs8 -in server-key.pem -outform DER -out key.der
```

# Now you have the following files:

- CA / Client files:
    - `ca-key.pem`: Private key of the CA (KEEP THIS SECURE)
    - `ca-cert.pem`: Public certificate of the CA
    - `ca.der`: DER-encoded CA certificate (for client)
- Server files:
    - `server-key.pem`: Private key of the server
    - `server-cert.pem`: Public certificate of the server
    - `cert.der`: DER-encoded server certificate (for client)
    - `key.der`: DER-encoded server private key (for server)

Remember to keep the CA private key (`ca-key.pem`) secure. This key is used to sign
certificates and should not be compromised.

# Client code:

```rust
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use rustls::{ClientConfig, RootCertStore};

fn main() {
    // Load the CA certificate
    let mut root_store = RootCertStore::empty();
    let mut cert_file = File::open("ca.der").unwrap();
    let mut cert_der = Vec::new();
    cert_file.read_to_end(&mut cert_der).unwrap();
    root_store.add(&rustls::Certificate(cert_der)).unwrap();

    // Create a TLS configuration
    let mut config = ClientConfig::new();
    config.root_store = root_store;

    // Connect to the server
    let tcp_stream = TcpStream::connect("127.0.0.1:8443").unwrap();
    let mut stream = rustls::ClientConnection::new(config, tcp_stream).unwrap();

    // Perform TLS handshake
    stream.handshake().unwrap();

    // Send and receive data
    stream.write_all(b"Hello, server!\n").unwrap();
    let mut buf = [0u8; 1024];
    let len = stream.read(&mut buf).unwrap();
    println!("Received: {}", String::from_utf8_lossy(&buf[..len]));
}
```

# Server code:

```rust
use std::fs::File;
use std::io::Read;
use rustls::{ServerConfig, NoClientAuth, Certificate, PrivateKey};

fn main() {
    // Load the certificate and private key
    let cert_file = File::open("cert.der").unwrap();
    let mut cert_der = Vec::new();
    cert_file.read_to_end(&mut cert_der).unwrap();

    let key_file = File::open("key.der").unwrap();
    let mut key_der = Vec::new();
    key_file.read_to_end(&mut key_der).unwrap();

    let cert = Certificate(cert_der);
    let key = PrivateKey(key_der);

    // Create a TLS configuration
    let mut config = ServerConfig::new(NoClientAuth::new());
    config.set_single_cert(vec![cert], key).unwrap();

    // ... rest of your server code, including creating a TCP listener and accepting connections
}
````