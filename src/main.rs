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
mod tests {
    use super::*;

    fn create_test_cli(overrides: impl FnOnce(&mut cli::Cli)) -> cli::Cli {
        let mut args = cli::Cli {
            input_folder: PathBuf::from("test"),
            output_folder: None,
            verbose: 0,
            watch: false,
            serve: false,
            bind: "0.0.0.0:8000".to_string(),
            config: "marmite.yaml".to_string(),
            debug: false,
            init_templates: false,
            start_theme: None,
            set_theme: None,
            generate_config: false,
            init_site: false,
            force: false,
            shortcodes: false,
            show_urls: false,
            create: cli::Create {
                new: None,
                edit: false,
                page: false,
                tags: None,
            },
            configuration: cli::Configuration {
                name: None,
                tagline: None,
                url: None,
                https: None,
                footer: None,
                language: None,
                pagination: None,
                enable_search: None,
                enable_related_content: None,
                content_path: None,
                templates_path: None,
                static_path: None,
                media_path: None,
                default_date_format: None,
                colorscheme: None,
                toc: None,
                json_feed: None,
                show_next_prev_links: None,
                publish_md: None,
                source_repository: None,
                image_provider: None,
                theme: None,
                build_sitemap: None,
                publish_urls_json: None,
            },
        };
        overrides(&mut args);
        args
    }

    #[test]
    fn test_determine_verbosity_default() {
        let args = create_test_cli(|_| {});
        assert_eq!(determine_verbosity(&args), 0);
    }

    #[test]
    fn test_determine_verbosity_watch_sets_verbose() {
        let args = create_test_cli(|args| {
            args.watch = true;
        });
        assert_eq!(determine_verbosity(&args), 1);

        let args = create_test_cli(|args| {
            args.serve = true;
        });
        assert_eq!(determine_verbosity(&args), 1);

        let args = create_test_cli(|args| {
            args.init_templates = true;
        });
        assert_eq!(determine_verbosity(&args), 1);
    }

    #[test]
    fn test_determine_verbosity_debug_flag() {
        let args = create_test_cli(|args| {
            args.debug = true;
        });
        assert_eq!(determine_verbosity(&args), 2);
    }

    #[test]
    fn test_determine_verbosity_explicit_verbose() {
        let args = create_test_cli(|args| {
            args.verbose = 3;
        });
        assert_eq!(determine_verbosity(&args), 3);
    }

    #[test]
    fn test_get_config_path_relative() {
        let input_folder = PathBuf::from("/home/user/project");
        let config = "marmite.yaml";
        let result = get_config_path(&input_folder, config);
        assert_eq!(result, PathBuf::from("/home/user/project/marmite.yaml"));
    }

    #[test]
    fn test_get_config_path_absolute() {
        let input_folder = PathBuf::from("/home/user/project");
        let config = "/etc/marmite.yaml";
        let result = get_config_path(&input_folder, config);
        assert_eq!(result, PathBuf::from("/etc/marmite.yaml"));
    }

    #[test]
    fn test_get_config_path_dot_relative() {
        let input_folder = PathBuf::from("/home/user/project");
        let config = "./config/marmite.yaml";
        let result = get_config_path(&input_folder, config);
        assert_eq!(result, PathBuf::from("./config/marmite.yaml"));
    }

    #[test]
    fn test_run_cli_nonexistent_folder() {
        let args = create_test_cli(|args| {
            args.input_folder = PathBuf::from("/nonexistent/folder");
        });

        let result = run_cli(args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Input folder does not exist"));
    }
}
