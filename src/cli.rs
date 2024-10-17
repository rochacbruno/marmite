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

    /// Path to custom configuration file (defaults to marmite.yaml)
    #[arg(long, default_value = "marmite.yaml")]
    pub config: String,
}
