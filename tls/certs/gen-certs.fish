#!/usr/bin/env fish

# Add current directory to PATH. This prevents the need to use `./cfssl` or `./cfssljson`.
set PATH (realpath .):$PATH

# If cfssl or cfssljson files do not exist, download them.
get-cfssl-binaries.fish

# If `generated` directory exists, delete it. And create a new one.
if test -d generated
    rm -rf generated
end
mkdir generated

# Generate CA and server certificates in the `generated` directory.
pushd generated

# Generate root certificate (CA) and sign it.
#
# Creates the following files:
# - ca.csr: certificate signing request
# - ca-key.pem: private key
# - ca.pem: public key; used in the Rust client code
cfssl gencert \
    -initca ../ca-csr.json \
    | cfssljson -bare ca

# Generate server certificate (and private key) and sign it with the CA.
#
# Arguments:
# - `-config ../ca-config.json` is the configuration file, which contains lifetimes for
#   the certificates.
# - `-profile server` is from `ca-config.json`
#
# Generates the following files:
# - server.csr: certificate signing request
# - server-key.pem: private key; used in the Rust server code
# - server.pem: public key; used in the Rust server code
cfssl gencert \
    -ca ca.pem \
    -ca-key ca-key.pem \
    -config ../ca-config.json \
    -profile server ../server.json \
    | cfssljson -bare server

popd

echo (set_color green)"✅ Generated certificates in the 'generated' directory."(set_color normal)
echo (set_color blue)"🔍 Verifying certificates..."(set_color normal)
openssl x509 -noout -text -in generated/server.pem
openssl verify -CAfile generated/ca.pem generated/server.pem

if test $status -eq 0
    echo (set_color green)"✅ Certificates are valid"(set_color normal)
else
    echo (set_color red)"❗ Certificates are invalid"(set_color normal)
end
