use clap::Parser;
use env_logger::{Builder, Env};
use log::{error, info};
use std::{path::PathBuf, sync::Arc};

mod cli;
mod config;
mod content;
mod embedded;
mod feed;
mod markdown;
mod server;
mod site;
mod templates;
mod tera_filter;
mod tera_functions;

fn main() {
    let args = cli::Cli::parse();
    let input_folder = args.input_folder;
    let output_folder = Arc::new(args.output_folder);
    let serve = args.serve;
    let watch = args.watch;
    let bind_address: &str = args.bind.as_str();
    let mut verbose = args.verbose; // -v info, -vv debug
    if args.debug {
        verbose = 2; // backward compatibility with --debug flag
    }

    let config_path = if args.config.starts_with('.') || args.config.starts_with('/') {
        PathBuf::new().join(args.config)
    } else {
        input_folder.join(args.config)
    };

    let env = Env::default().default_filter_or(match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        3..=u8::MAX => "trace",
    });
    if let Err(e) = Builder::from_env(env).try_init() {
        error!("Logger already initialized: {}", e);
    }

    // Handle `init_templates` flag
    if args.init_templates {
        templates::initialize_templates(&input_folder);
        return; // Exit early if only initializing templates
    }

    // Handle `start_theme` flag
    if args.start_theme {
        templates::initialize_templates(&input_folder);
        templates::initialize_theme_assets(&input_folder);
        return; // Exit early if only initializing theme
    }

    // Handle `generate_config` flag
    if args.generate_config {
        config::generate(&input_folder);
        return; // Exit early if only generating config
    }

    // Generate site content
    site::generate(
        &config_path,
        &input_folder,
        &output_folder,
        watch,
        serve,
        bind_address,
    );

    // Serve the site if the flag was provided
    if serve && !watch {
        info!("Starting built-in HTTP server...");
        server::start(bind_address, &output_folder);
    }
}
