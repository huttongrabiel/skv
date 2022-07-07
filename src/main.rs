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
use skv::connection::KeyValueStore;
use skv::thread::ThreadPool;
use std::{
    error::Error,
    net::TcpListener,
    sync::{Arc, Mutex},
};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let listener = TcpListener::bind(format!("localhost:{}", args.port))
        .expect("Failed to bind to localhost (127.0.0.1) on port 3400");

    let key_value_store = Arc::new(Mutex::new(KeyValueStore::new()));

    let thread_pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let kv_store = Arc::clone(&key_value_store);
        thread_pool.execute(move || {
            kv_store.lock().unwrap().handle_request(stream).unwrap();
        });
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
