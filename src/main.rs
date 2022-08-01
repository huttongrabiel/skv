use clap::Parser;
use skv::cli::{Cli, Commands};
use skv::connection::KeyValueStore;
use skv::thread::ThreadPool;
use std::{
    error::Error,
    net::TcpListener,
    sync::{Arc, Mutex},
};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    // TODO: Figure out how to handle HTTP requests done through the CLI.
    match &args.command {
        Commands::Start { port } => start_server(port.to_string())?,
        Commands::PUT { key, value } => {}
        Commands::GET {
            key,
            encryption_key,
        } => {}
        Commands::DELETE {
            key,
            encryption_key,
        } => {}
        Commands::ListKeys { encryption_key } => {}
    }

    Ok(())
}

fn start_server(port: String) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(format!("localhost:{}", port)).expect(
        format!("Failed to bind to localhost (127.0.0.1) on port {}", port,)
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
