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

use std::{error::Error, net::TcpListener};

use skv::KeyValueStore;

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("localhost:3400")
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
