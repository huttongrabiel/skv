use clap::Parser;
use skv::connection::{self, KeyValueStore, RequestType};
use skv::thread::ThreadPool;
use std::{
    error::Error,
    net::TcpListener,
    sync::{Arc, RwLock},
};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let listener = TcpListener::bind(format!("localhost:{}", args.port))
        .unwrap_or_else(|_| {
            panic!(
                "Failed to bind to localhost (127.0.0.1) on port {}",
                args.port,
            )
        });

    let key_value_store = Arc::new(RwLock::new(KeyValueStore::new()));

    let thread_pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let buf = connection::buf_from_stream(&stream)?;
        connection::verify_request(&buf)?;
        let request_type = connection::request_type(&buf);

        let kv_store = Arc::clone(&key_value_store);
        thread_pool.execute(move || {
            let (status_line, body) = match request_type {
                RequestType::Get => kv_store
                    .read()
                    .expect("Failed to acquire read lock for get request.")
                    .handle_get_request(&buf),
                RequestType::Put => kv_store
                    .write()
                    .expect("Failed to acquire write lock for PUT request.")
                    .handle_put_request(&buf),
                RequestType::Delete => kv_store
                    .write()
                    .expect("Failed to acquire write lock for DELETE request.")
                    .handle_put_request(&buf),
                RequestType::Unknown(mesg) => mesg,
            };
            match connection::write_stream(&stream, status_line, body) {
                Ok(_) => (),
                Err(e) => eprintln!("Error encountered: {}", e),
            };
        });
    }

    Ok(())
}

/// A simple key-value (skv) store.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Specify port on localhost to run skv server.
    #[clap(short, long, value_parser, default_value = "3400")]
    pub port: String,
}
