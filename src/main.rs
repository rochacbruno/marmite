use clap::Parser;
use env_logger::{Builder, Env};
use log::{error, info, warn, SetLoggerError};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

mod cli;
mod config;
mod content;
mod embedded;
mod feed;
mod gallery;
mod image_provider;
mod parser;
mod server;
mod shortcodes;
mod site;
mod templates;
mod tera_filter;
mod tera_functions;
mod theme_manager;

fn setup_logging(verbose: u8, debug: bool) -> Result<(), SetLoggerError> {
    let env = Env::default().default_filter_or(match verbose {
        0 => "marmite=warn",
        1 => "marmite=info",
        2 => "marmite=debug",
        3 => "marmite=trace",
        4..=u8::MAX => "trace",
    });
    Builder::from_env(env).try_init()?;

    if debug {
        warn!("--debug flag is deprecated, use -vv for debug messages");
    }
    Ok(())
}

fn determine_verbosity(args: &cli::Cli) -> u8 {
    let mut verbose = args.verbose;

    if verbose == 0
        && (args.watch
            || args.serve
            || args.start_theme.is_some()
            || args.set_theme.is_some()
            || args.init_templates
            || args.generate_config
            || args.init_site)
    {
        verbose = 1;
    }
    if args.debug {
        verbose = 2; // backward compatibility with --debug flag
    }

    verbose
}

fn get_config_path(input_folder: &Path, config: &str) -> PathBuf {
    if config.starts_with('.') || config.starts_with('/') {
        PathBuf::new().join(config)
    } else {
        input_folder.join(config)
    }
}

#[allow(clippy::too_many_lines)]
fn run_cli(args: cli::Cli) -> Result<(), Box<dyn std::error::Error>> {
    let cloned_args = Arc::new(args.clone()); // Clone to pass to the server thread
    let input_folder = Arc::new(args.input_folder.clone());
    let serve = args.serve;
    let watch = args.watch;
    let bind_address: &str = args.bind.as_str();
    let verbose = determine_verbosity(&args);

    let config_path = Arc::new(get_config_path(args.input_folder.as_path(), &args.config));

    if let Err(e) = setup_logging(verbose, args.debug) {
        error!("Logger already initialized: {e:?}");
    }

    if args.init_site {
        site::initialize(&input_folder, &cloned_args);
        return Ok(());
    }

    if !input_folder.exists() {
        return Err(format!("Input folder does not exist: {input_folder:?}").into());
    }

    if let Some(title) = args.create.new {
        content::new(&input_folder, &title, &cloned_args, &config_path);
        return Ok(());
    }

    if args.init_templates {
        templates::initialize_templates(&input_folder);
        return Ok(());
    }

    if let Some(theme_name) = args.start_theme {
        templates::initialize_theme(&input_folder, &theme_name);
        return Ok(());
    }

    if let Some(theme_source) = args.set_theme {
        theme_manager::set_theme(
            &input_folder,
            &theme_source,
            args.configuration.theme.clone(),
        );
        return Ok(());
    }

    if args.generate_config {
        config::generate(&input_folder, &cloned_args);
        return Ok(());
    }

    if args.shortcodes {
        let mut processor = shortcodes::ShortcodeProcessor::new(None);
        if let Err(e) = processor.collect_shortcodes(&input_folder) {
            return Err(format!("Failed to collect shortcodes: {e}").into());
        }
        println!("Shortcodes:");
        println!("Reusable blocks of content that can be used in your markdown files.");
        println!("They are defined in the shortcodes/ directory and are rendered using the Tera template engine.");
        println!("Check the documentation for details on how to use and create shortcodes.");
        println!("================");
        println!("Examples:");
        println!("<!-- .youtube id=dQw4w9WgXcQ -->");
        println!("<!-- .youtube id=dQw4w9WgXcQ width=800 height=600 -->");
        println!("<!-- .toc -->");
        println!("<!-- .authors -->");
        println!("<!-- .streams ord=desc items=5 -->");
        println!("--------------------------------");
        println!("Available shortcodes:");
        for (name, description) in processor.list_shortcodes_with_descriptions() {
            match description {
                Some(desc) => println!("  - {name}: {desc}"),
                None => println!("  - {name}"),
            }
        }
        return Ok(());
    }

    if args.show_urls {
        site::show_urls(&config_path, &input_folder, &cloned_args);
        return Ok(());
    }

    let output_folder = Arc::new(args.output_folder.unwrap_or(input_folder.join("site")));
    if let Err(e) = site::generate(
        &config_path,
        &input_folder,
        &output_folder,
        watch,
        serve,
        bind_address,
        &cloned_args,
    ) {
        error!("Failed to generate site: {e}");
        std::process::exit(1);
    }

    if serve && !watch {
        info!("Starting built-in HTTP server...");
        server::start(bind_address, &Arc::clone(&output_folder));
    }

    Ok(())
}

fn main() {
    let args = cli::Cli::parse();
    if let Err(e) = run_cli(args) {
        error!("{e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
#[path = "tests/main.rs"]
mod tests;
