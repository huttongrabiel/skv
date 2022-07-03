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

use std::{
    error::Error,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use skv::KeyValueRequest;

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("localhost:3400")
        .expect("Failed to bind to localhost (127.0.0.1) on port 3400");

    for stream in listener.incoming() {
        handle_connection(stream?);
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    stream.read(&mut buf).unwrap();

    let kvr = KeyValueRequest::new(&buf);
    let kvr_response = kvr.handle_request();

    stream.write(kvr_response.unwrap().as_bytes()).unwrap();
    stream.flush().unwrap();
}
