use clap::{Parser, Subcommand};

/// A simple key-value (skv) store.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Command for server to execute.
    ///
    /// Supported commands are: start, PUT, GET, DELETE, and ls.
    #[clap(subcommand)]
    pub command: Commands,
    /// Specify port on localhost to run skv server.
    #[clap(short, long, value_parser, default_value = "3400")]
    pub port: String,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Start {
        /// Specify port on localhost to run skv server.
        #[clap(value_parser, default_value = "3400")]
        port: String,
    },
    PUT {
        /// Key used for look up in the key-value store.
        #[clap(value_parser)]
        key: String,
        /// Information associated with key.
        #[clap(short, long, value_parser)]
        value: String,
    },
    GET {
        /// Key used for look up in the key-value store.
        #[clap(value_parser)]
        key: String,
        /// Information associated with key.
        #[clap(value_parser)]
        encryption_key: String,
    },
    DELETE {
        /// Key used for look up in the key-value store.
        #[clap(value_parser)]
        key: String,
        /// Encryption key provided to user at time of server start.
        #[clap(value_parser)]
        encryption_key: String,
    },
    ListKeys {
        /// Encryption key provided to user at time of server start.
        #[clap(value_parser)]
        encryption_key: String,
    },
}
