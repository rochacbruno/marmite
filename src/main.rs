use clap::Parser;
use env_logger::{Builder, Env};
use log::{error, info, warn};
use std::{path::PathBuf, sync::Arc};

mod cli;
mod config;
mod content;
mod embedded;
mod feed;
mod image_provider;
mod linkcheck;
mod parser;
mod server;
mod site;
mod templates;
mod tera_filter;
mod tera_functions;

fn main() {
    let args = cli::Cli::parse();
    let cloned_args = Arc::new(args.clone()); // Clone to pass to the server thread
    let input_folder = Arc::new(args.input_folder);
    let serve = args.serve || args.check_links; // check_links implies serve
    let watch = args.watch;
    let bind_address: &str = args.bind.as_str();
    let check_links = args.check_links;
    let mut verbose = args.verbose; // -v info, -vv debug

    if verbose == 0
        && (args.watch
            || args.serve
            || args.check_links
            || args.start_theme
            || args.init_templates
            || args.generate_config
            || args.init_site)
    {
        verbose = 1;
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
        error!("Logger already initialized: {e:?}");
    }
    if args.debug {
        warn!("--debug flag is deprecated, use -vv for debug messages");
    }

    if args.init_site {
        site::initialize(&input_folder, &cloned_args);
        return;
    }

    if !input_folder.exists() {
        error!("Input folder does not exist: {input_folder:?}");
        return;
    }

    if let Some(title) = args.create.new {
        content::new(&input_folder, &title, &cloned_args, &config_path);
        return;
    }

    if args.init_templates {
        templates::initialize_templates(&input_folder);
        return;
    }

    if args.start_theme {
        templates::initialize_templates(&input_folder);
        templates::initialize_theme_assets(&input_folder);
        return;
    }

    if args.generate_config {
        config::generate(&input_folder, &cloned_args);
        return;
    }

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

    // If check_links is enabled, start the server first, then check links
    if check_links {
        info!("Starting temporary HTTP server for link checking...");
        let server_handle = std::thread::spawn({
            let output_folder = Arc::clone(&output_folder);
            let bind_address = bind_address.to_string();
            move || {
                server::start(&bind_address, &output_folder);
            }
        });

        // Give the server a moment to start
        std::thread::sleep(std::time::Duration::from_millis(1000));

        // Construct base URL from bind address
        let base_url = if bind_address.starts_with("0.0.0.0:") {
            format!(
                "http://127.0.0.1:{}",
                bind_address.split(':').nth(1).unwrap_or("8000")
            )
        } else {
            format!("http://{bind_address}")
        };

        // Check links
        if let Err(e) = linkcheck::check_links(&output_folder, &base_url) {
            error!("Link checking failed: {e}");
            std::process::exit(1);
        }

        // If we're not in watch mode, we can exit after checking
        if !watch {
            info!("Link checking complete. Shutting down.");
            std::process::exit(0);
        }

        // In watch mode, let the server continue running
        server_handle.join().unwrap();
    } else if serve && !watch {
        info!("Starting built-in HTTP server...");
        server::start(bind_address, &Arc::clone(&output_folder));
    }
}
