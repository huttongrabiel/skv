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
        .expect(
            format!(
                "Failed to bind to localhost (127.0.0.1) on port {}",
                args.port,
            )
            .as_str(),
        );

    let key_value_store = Arc::new(Mutex::new(KeyValueStore::new()));

    let thread_pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let kv_store = Arc::clone(&key_value_store);
        thread_pool.execute(move || {
            match kv_store.lock().unwrap().handle_request(stream) {
                Ok(_) => (),
                Err(e) => eprintln!("Error, try again: {}", e),
            }
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
