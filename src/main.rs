// HTTP Request
//
// Method Request-URI HTTP-Version CRLF
// headers CRLF
// message-body

// HTTP Response
//
// HTTP-Version Status-Code Reason-Phrase CRLF
// headers CRLF
// message-body

use clap::Parser;
use skv::KeyValueStore;
use std::{error::Error, net::TcpListener};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let listener = TcpListener::bind(format!("localhost:{}", args.port))
        .expect("Failed to bind to localhost (127.0.0.1) on port 3400");

    let mut key_value_store = KeyValueStore::new();

    for stream in listener.incoming() {
        match key_value_store.handle_request(stream?) {
            Ok(_) => (),
            Err(e) => panic!("Failed to serve request: {}", e),
        }
    }

    Ok(())
}

/// A simple key-value (skv) store.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Specify port on localhost to run skv server.
    #[clap(short, long, value_parser, default_value = "3400")]
    port: String,
}
