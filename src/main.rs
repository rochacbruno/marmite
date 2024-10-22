use clap::Parser;
use env_logger::{Builder, Env};

use log::{error, info};
use std::sync::Arc;

mod cli;
mod config;
mod content;
mod embedded;
mod markdown;
mod server;
mod site;
mod tera_functions;

fn main() {
    let args = cli::Cli::parse();
    let input_folder = args.input_folder;
    let output_folder = Arc::new(args.output_folder);
    let serve = args.serve;
    let config_path = input_folder.join(args.config);
    let bind_address: &str = args.bind.as_str();

    let env = Env::default().default_filter_or(if args.debug { "debug" } else { "info" });
    if let Err(e) = Builder::from_env(env).try_init() {
        error!("Logger already initialized: {}", e);
    }

    site::generate(&config_path, &input_folder, &output_folder);

    // Serve the site if the flag was provided
    if serve {
        info!("Starting built-in HTTP server...");
        server::start(bind_address, &output_folder);
    }
}
