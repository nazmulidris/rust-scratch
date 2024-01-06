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

#[macro_use]
extern crate clap;

use std::io;
use std::io::prelude::*;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};

fn main() {
    let matches = clap_app!(rcat =>
        (version: "1.0")
        (author: "Eliott Teissonniere <eliott.teissonniere.org>")
        (about: "Tiny netcat like program in rust")
        (@arg PORT: -p --port +takes_value "Port to connect to or listen on")
        (@arg ADDR: -a --address +takes_value "Address to connect to or listen on")
        (@subcommand listen =>
            (about: "Listen for incoming connections on specified address and port")
        )
        (@subcommand connect =>
            (about: "Connect to specified address and port")
        )
    )
    .get_matches();

    let addr = matches.value_of("ADDR").unwrap_or("127.0.0.1");
    let port: u16 = matches.value_of("PORT").unwrap_or("4242").parse().unwrap();

    if matches.subcommand_matches("listen").is_some() {
        println!("Listening on {}:{}", addr, port);
        listen(addr, port);
    } else if matches.subcommand_matches("connect").is_some() {
        println!("Connecting to {}:{}", addr, port);
        connect(addr, port);
    } else {
        println!("Unknown command");
    }
}

fn connect(addr: &str, port: u16) {
    let socket = SocketAddr::new(IpAddr::V4(addr.parse().unwrap()), port);
    if let Ok(mut stream) = TcpStream::connect(&socket) {
        loop {
            print!("> ");
            let _ = io::stdout().flush();

            let mut input = String::new();
            let _ = io::stdin().read_line(&mut input);

            println!(">>> {}", input.trim());
            let _ = stream.write(input.as_bytes());
        }
    } else {
        println!("Failure to connect to the target");
    }
}

fn listen(addr: &str, port: u16) {
    match TcpListener::bind((addr, port)) {
        Ok(listener) => {
            for stream in listener.incoming() {
                let stream = stream.unwrap();
                handle_connection(stream);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}

fn handle_connection(mut stream: TcpStream) {
    loop {
        let mut buffer = [0; 512];
        match stream.read(&mut buffer) {
            Ok(s) => {
                if s == 0 {
                    return;
                }

                let received = String::from_utf8_lossy(&buffer[..]);
                print!("<<< {}", received);
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}
