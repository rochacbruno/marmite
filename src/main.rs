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
mod parser;
mod server;
mod shortcodes;
mod site;
mod templates;
mod tera_filter;
mod tera_functions;
mod theme_manager;

#[allow(clippy::too_many_lines)]
fn main() {
    let args = cli::Cli::parse();
    let cloned_args = Arc::new(args.clone()); // Clone to pass to the server thread
    let input_folder = Arc::new(args.input_folder);
    let serve = args.serve;
    let watch = args.watch;
    let bind_address: &str = args.bind.as_str();
    let mut verbose = args.verbose; // -v info, -vv debug

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

    if let Some(theme_name) = args.start_theme {
        templates::initialize_theme(&input_folder, &theme_name);
        return;
    }

    if let Some(theme_source) = args.set_theme {
        theme_manager::set_theme(
            &input_folder,
            &theme_source,
            args.configuration.theme.clone(),
        );
        return;
    }

    if args.generate_config {
        config::generate(&input_folder, &cloned_args);
        return;
    }

    if args.shortcodes {
        let mut processor = shortcodes::ShortcodeProcessor::new(None);
        if let Err(e) = processor.collect_shortcodes(&input_folder) {
            error!("Failed to collect shortcodes: {e}");
            return;
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
        return;
    }

    if args.show_urls {
        site::show_urls(&config_path, &input_folder, &cloned_args);
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

    if serve && !watch {
        info!("Starting built-in HTTP server...");
        server::start(bind_address, &Arc::clone(&output_folder));
    }
}
