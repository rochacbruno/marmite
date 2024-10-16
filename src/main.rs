mod cli;
mod init;
mod server;
mod site_data;
mod file_processing;
mod render;

use cli::Cli;
use init::init_project;
use server::serve_website;
use site_data::{Site, SiteData};
use file_processing::process_files;
use render::render_templates;

use structopt::StructOpt;
use std::fs;
use std::process;

fn main() {
    let cli = Cli::from_args();

    let folder = cli.get_folder();
    let config_path = cli.get_config_path();

    if cli.init.is_some() {
        if let Err(e) = init_project(&folder) {
            eprintln!("Failed to initialize project: {}", e);
            process::exit(1);
        }
        println!("Project initialized successfully.");
        return;
    }

    // Initialize site data
    let marmite = match fs::read_to_string(&config_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Unable to read {}: {}", config_path.display(), e);
            process::exit(1);
        }
    };
    let site: Site = match serde_yaml::from_str(&marmite) {
        Ok(site) => site,
        Err(e) => {
            eprintln!("Failed to parse YAML: {}", e);
            process::exit(1);
        }
    };
    let mut site_data = SiteData::new(&site);

    if let Err(e) = process_files(&folder, &mut site_data) {
        eprintln!("Failed to process files: {}", e);
        process::exit(1);
    }

    if let Err(e) = render_templates(&site_data, &folder.join(&site_data.site.site_path)) {
        eprintln!("Failed to render templates: {}", e);
        process::exit(1);
    }

    println!("Site generated at: {}/", site_data.site.site_path);

    if cli.serve {
        if let Err(e) = serve_website(&folder.join(&site_data.site.site_path)) {
            eprintln!("Failed to serve website: {}", e);
            process::exit(1);
        }
    }
}
