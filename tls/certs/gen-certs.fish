#!/usr/bin/env fish

# These are all the config files that drive the creation of the certificates (CA and server).
set config_file_ca ../config/ca-config.json
set config_file_ca_csr ../config/ca-csr.json
set config_file_server ../config/server-csr.json

# These are the values (which are defined inside the config files above).
set config_value_ca_cn ca # Eg: `ca`
set config_value_server_cn server # Eg: `server`

# These are values derived from the config values above.
set ca_pem_file "$config_value_ca_cn".pem # Eg `ca.pem`
set ca_key_pem_file "$config_value_ca_cn"-key.pem # Eg `ca-key.pem`
set server_pem_file "$config_value_server_cn".pem # Eg: `server.pem`; only used in the `display_status` function.

set bin_folder bin
set generated_folder generated

function generate_certs
    # Generate root certificate (CA) and sign it.
    #
    # Creates the following files:
    # - ca.csr: certificate signing request
    # - ca-key.pem: private key
    # - ca.pem: public key; used in the Rust client code
    cfssl gencert \
        -initca $config_file_ca_csr \
        | cfssljson -bare $config_value_ca_cn

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
        -ca $ca_pem_file \
        -ca-key $ca_key_pem_file \
        -config $config_file_ca \
        -profile $config_value_server_cn $config_file_server \
        | cfssljson -bare $config_value_server_cn
end

function display_status
    echo (set_color green)"🎉 Generated certificates in the "(set_color yellow)(realpath .)" directory."(set_color normal)
    echo (set_color blue)"🔍 Verifying certificates..."(set_color normal)

    openssl x509 -noout -text -in $ca_pem_file
    openssl x509 -noout -text -in $server_pem_file
    openssl verify -CAfile $ca_pem_file $server_pem_file

    if test $status -eq 0
        echo (set_color green)"🎉 Certificates are valid"(set_color normal)
    else
        echo (set_color red)"❗ Certificates are invalid"(set_color normal)
    end
end

function main
    # Add to PATH:
    # - `realpath .`: current directory
    # - `realpath $bin_folder`: path to the `bin` folder
    # This prevents the need to use `./cfssl` or `./cfssljson`.
    set PATH (realpath .):(realpath $bin_folder):$PATH

    # If cfssl or cfssljson files do not exist, download them.
    get-cfssl-binaries.fish

    # If `generated` directory exists, delete it. And create a new one.
    if test -d $generated_folder
        rm -rf $generated_folder
    end
    mkdir $generated_folder

    # Generate CA and server certificates in the `generated` directory.
    pushd $generated_folder
    generate_certs
    display_status
    popd
end

main
