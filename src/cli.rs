#![allow(clippy::struct_excessive_bools)]
use clap::Parser;
use std::path::PathBuf;

/// Command Line Argument Parser for Marmite CLI
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Input folder containing markdown files
    pub input_folder: PathBuf,

    /// Output folder to generate the site
    pub output_folder: PathBuf,

    /// Serve the site with a built-in HTTP server
    #[arg(long)]
    pub serve: bool,

    /// Detect changes and rebuild the site automatically
    #[arg(long)]
    pub watch: bool,

    /// Address to bind the server
    #[arg(long, default_value = "localhost:8000")]
    pub bind: String,

    /// Path to custom configuration file
    #[arg(long, default_value = "marmite.yaml")]
    pub config: String,

    /// Print debug messages
    /// Deprecated: Use -vv for debug messages
    #[arg(long)]
    pub debug: bool,

    /// Initialize templates in the project
    #[arg(long)]
    pub init_templates: bool,

    /// Initialize a theme with templates and static assets
    #[arg(long)]
    pub start_theme: bool,

    /// Generate the configuration file
    #[arg(long)]
    pub generate_config: bool,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}
