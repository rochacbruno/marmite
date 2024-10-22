use clap::Parser;
use env_logger::{Builder, Env};

use log::{error, info};
use std::sync::Arc;

mod cli;
mod config;
mod content;
mod embedded;
mod markdown;
mod robots;
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

        println!("Site generated at: {}/", output_folder.display());
    };

    // Build the site initially
    build_site();

    // Clone output_folder for later use
    let output_folder_clone = Arc::clone(&output_folder);

    // Watch for changes if the --watch flag is provided
    if watch {
        let input_folder_clone = Arc::clone(&input_folder);
        let build_site = Arc::new(Mutex::new(build_site));

        // Initialize Hotwatch
        let mut hotwatch = Hotwatch::new().expect("Hotwatch failed to initialize!");
        println!("Watching for changes in {:?}", input_folder_clone);

        // Watch the content directory for changes
        hotwatch.watch(&*input_folder_clone, move |_event| {
            println!("Change detected, rebuilding the site...");
            let build_site = Arc::clone(&build_site);
            let build_site = build_site.lock().unwrap();
            (build_site)(); // Trigger site rebuild
        }).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    }

        server::start_server(&bind_address, output_folder_clone.into());
    if serve {
        info!("Starting built-in HTTP server...");
        server::start(bind_address, &output_folder);
    }
}
