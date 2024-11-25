use clap::Parser;
use env_logger::{Builder, Env};
use log::{error, info, warn};
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
    let cloned_args = Arc::new(args.clone()); // Clone to pass to the server thread
    let input_folder = Arc::new(args.input_folder);
    let serve = args.serve;
    let watch = args.watch;
    let bind_address: &str = args.bind.as_str();
    let mut verbose = args.verbose; // -v info, -vv debug

    if verbose == 0 && (args.watch || args.serve) {
        verbose = 1; // force info level when watching or serving
    }
    if args.debug {
        verbose = 2; // backward compatibility with --debug flag
    }

    let config_path = if args.config.starts_with('.') || args.config.starts_with('/') {
        Arc::new(PathBuf::new().join(args.config))
    } else {
        Arc::new(input_folder.join(args.config))
    };

    let env = Env::default().default_filter_or(match verbose {
        0 => "marmite=warn",
        1 => "marmite=info",
        2 => "marmite=debug",
        3 => "marmite=trace",
        4..=u8::MAX => "trace",
    });
    if let Err(e) = Builder::from_env(env).try_init() {
        error!("Logger already initialized: {}", e);
    }
    if args.debug {
        warn!("--debug flag is deprecated, use -vv for debug messages");
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
        config::generate(&input_folder, &cloned_args);
        return; // Exit early if only generating config
    }

    // Handle `init_site` flag
    if args.init_site {
        site::initialize(&input_folder, &cloned_args);
        return; // Exit early if only initializing site
    }

    // Generate site content
    let output_folder = Arc::new(args.output_folder.unwrap_or(input_folder.join("site")));
    site::generate(
        &config_path,
        &input_folder,
        &output_folder,
        watch,
        serve,
        bind_address,
        &cloned_args,
    );
    // Serve the site if the flag was provided
    if serve && !watch {
        info!("Starting built-in HTTP server...");
        server::start(bind_address, &Arc::clone(&output_folder));
    }
}
