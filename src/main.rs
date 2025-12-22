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
mod image_resize;
mod parser;
mod re;
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
        return handle_shortcodes_command(&input_folder, &cloned_args);
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
        server::start(bind_address, &Arc::clone(&output_folder), None);
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

/// Handle the --shortcodes command to display available shortcodes
fn handle_shortcodes_command(
    input_folder: &Path,
    cli_args: &Arc<cli::Cli>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration to check if shortcodes are enabled
    let config_path = input_folder.join(cli_args.config.as_str());
    let mut site_data = site::Data::from_file(&config_path);
    site_data.site.override_from_cli_args(cli_args);

    let mut processor =
        shortcodes::ShortcodeProcessor::new(site_data.site.shortcode_pattern.as_deref());
    if let Err(e) = processor.collect_shortcodes(input_folder) {
        return Err(format!("Failed to collect shortcodes: {e}").into());
    }

    println!("Shortcodes:");
    println!("Enabled: {}", site_data.site.enable_shortcodes);

    // Display the actual pattern being used
    let pattern = site_data
        .site
        .shortcode_pattern
        .as_deref()
        .unwrap_or(re::SHORTCODE_HTML_COMMENT);
    println!("Pattern: {pattern}");

    println!("\nReusable blocks of content that can be used in your markdown files.");
    println!("They are defined in the shortcodes/ directory and are rendered using the Tera template engine.");
    println!("Check the documentation for details on how to use and create shortcodes.");
    println!("================");

    // Generate examples based on available shortcodes and the actual pattern
    if processor.shortcodes.is_empty() {
        println!("No shortcodes available.");
    } else {
        println!("Examples based on your configuration:");

        // Get a few shortcode names for examples
        let shortcode_names: Vec<&str> = processor
            .shortcodes
            .keys()
            .take(3)
            .map(std::string::String::as_str)
            .collect();

        // Try to generate pattern-based examples
        for name in shortcode_names {
            // Build example based on the pattern
            let example = if pattern.contains(r"<!--") {
                // Default HTML comment pattern
                format!("  <!-- .{name} -->")
            } else if pattern.contains(r"\{\{<") || pattern.contains("{{<") {
                // Hugo-style shortcode
                format!("  {{{{< {name} >}}}}")
            } else if pattern.contains(r"\{\{%") || pattern.contains("{%") {
                // Liquid/Jekyll style
                format!("  {{{{% {name} %}}}}")
            } else if pattern.contains(r"\[") || pattern.contains('[') {
                // Markdown-style shortcode
                format!("  [{name}]")
            } else {
                // Unknown pattern, show generic
                format!("  [Use '{name}' with your pattern]")
            };
            println!("{example}");

            // Add parameter example for known shortcodes
            if matches!(
                name,
                "youtube"
                    | "spotify"
                    | "posts"
                    | "pages"
                    | "tags"
                    | "streams"
                    | "authors"
                    | "series"
                    | "card"
                    | "gallery"
            ) {
                let param_example = if pattern.contains(r"<!--") {
                    format!("  <!-- .{name} param=value -->")
                } else if pattern.contains(r"\{\{<") || pattern.contains("{{<") {
                    format!("  {{{{< {name} param=\"value\" >}}}}")
                } else if pattern.contains(r"\{\{%") || pattern.contains("{%") {
                    format!("  {{{{% {name} param=\"value\" %}}}}")
                } else {
                    String::new()
                };
                if !param_example.is_empty() {
                    println!("{param_example}");
                }
            }
        }

        println!("\nNote: Replace 'param' and 'value' with actual parameter names and values.");
        if !pattern.contains(r"<!--") {
            println!("Custom pattern in use: {pattern}");
        }
    }

    println!("--------------------------------");
    println!("Available shortcodes:");
    for (name, description) in processor.list_shortcodes_with_descriptions() {
        match description {
            Some(desc) => println!("  - {name}: {desc}"),
            None => println!("  - {name}"),
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/main.rs"]
mod tests;
