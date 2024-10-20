use crate::config::Marmite;
use crate::site::SiteData;
use clap::Parser;
use content::check_for_duplicate_slugs;
use env_logger::{Builder, Env};
use fs_extra::dir::{copy, CopyOptions};
use log::{debug, error, info};
use markdown::process_file;
use site::render_templates;
use std::fs;
use std::io;
use std::process;
use std::sync::Arc;
use tera::Tera;
use walkdir::WalkDir;
mod cli;
mod config;
mod content;
mod markdown;
mod robots;
mod server;
mod site;
mod tera_functions;

fn main() -> io::Result<()> {
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

    // Initialize site data
    let marmite = fs::read_to_string(&config_path).unwrap_or_else(|e| {
        debug!(
            "Unable to read '{}', assuming defaults.: {}",
            &config_path.display(),
            e
        );
        String::new()
    });
    let site: Marmite = match serde_yaml::from_str(&marmite) {
        Ok(site) => site,
        Err(e) => {
            error!("Failed to parse '{}' YAML: {}", &config_path.display(), e);
            process::exit(1);
        }
    };
    let mut site_data = SiteData::new(&site);

    // Define the content directory
    let content_dir = Some(input_folder.join(&site_data.site.content_path))
        .filter(|path| path.is_dir()) // Take if exists
        .unwrap_or_else(|| input_folder.clone()); // Fallback to input_folder if not

    // Walk through the content directory
    WalkDir::new(&content_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path().is_file() && e.path().extension().and_then(|ext| ext.to_str()) == Some("md")
        })
        .for_each(|entry| {
            if let Err(e) = process_file(entry.path(), &mut site_data) {
                error!("Failed to process file {}: {}", entry.path().display(), e);
            }
        });

    // Detect slug collision
    if let Err(duplicate) = check_for_duplicate_slugs(
        &site_data
            .posts
            .iter()
            .chain(&site_data.pages)
            .collect::<Vec<_>>(),
    ) {
        error!(
            "Error: Duplicate slug found: '{}' \
            - try setting any of `title`, `slug` as a unique text, \
            or leave both empty so filename will be assumed.",
            duplicate
        );
        process::exit(1);
    }

    // Sort posts by date (newest first)
    site_data.posts.sort_by(|a, b| b.date.cmp(&a.date));
    // Sort pages on title
    site_data.pages.sort_by(|a, b| b.title.cmp(&a.title));

    // Create the output directory
    let output_path = output_folder.join(&site_data.site.site_path);
    if let Err(e) = fs::create_dir_all(&output_path) {
        error!("Unable to create output directory: {}", e);
        process::exit(1);
    }

    robots::handle_robots(&content_dir, &output_path);

    // Initialize Tera templates
    let templates_path = input_folder.join(&site_data.site.templates_path);
    let mut tera = match Tera::new(&format!("{}/**/*.html", templates_path.display())) {
        Ok(t) => t,
        Err(e) => {
            error!("Error loading templates: {}", e);
            process::exit(1);
        }
    };
    tera.autoescape_on(vec![]); // the person writing a static site knows what is doing!
    tera.register_function(
        "url_for",
        tera_functions::UrlFor {
            base_url: site_data.site.url.to_string(),
        },
    );
    tera.register_filter("slugify", tera_functions::slugify_filter);

    // Render templates
    if let Err(e) = render_templates(&site_data, &tera, &output_path) {
        error!("Failed to render templates: {}", e);
        process::exit(1);
    }

    // Copy static folder if present
    let static_source = input_folder.join(site_data.site.static_path);
    if static_source.is_dir() {
        let mut options = CopyOptions::new();
        options.overwrite = true; // Overwrite files if they already exist

        if let Err(e) = copy(&static_source, &*output_folder, &options) {
            error!("Failed to copy static directory: {}", e);
            process::exit(1);
        }

        info!(
            "Copied '{}' to '{}/'",
            &static_source.display(),
            &output_folder.display()
        );
    }

    // Copy content/media folder if present
    let media_source = content_dir.join(site_data.site.media_path);
    if media_source.is_dir() {
        let mut options = CopyOptions::new();
        options.overwrite = true; // Overwrite files if they already exist

        if let Err(e) = copy(&media_source, &*output_folder, &options) {
            error!("Failed to copy media directory: {}", e);
            process::exit(1);
        }

        info!(
            "Copied '{}' to '{}/'",
            &media_source.display(),
            &output_folder.display()
        );
    }

    // Copy or create favicon.ico
    let favicon_dst = output_folder.join("favicon.ico");

    // Possible paths where favicon.ico might exist
    let favicon_src_paths = [
        input_folder.join("static").join("favicon.ico"), // User's favicon.ico
                                                         // on #20 we may have embedded statics
    ];

    for favicon_src in &favicon_src_paths {
        if favicon_src.exists() {
            match fs::copy(&favicon_src, &favicon_dst) {
                Ok(_) => {
                    info!(
                        "Copied favicon.ico from '{}' to output folder",
                        favicon_src.display()
                    );
                    break;
                }
                Err(e) => error!(
                    "Failed to copy favicon.ico from '{}': {}",
                    favicon_src.display(),
                    e
                ),
            }
        }
    }

    // Serve the site if the flag was provided
    if serve {
        info!("Starting built-in HTTP server...");
        server::start_server(&bind_address, output_folder.clone().into());
    }

    info!("Site generated at: {}/", output_folder.display());

    Ok(())
}
