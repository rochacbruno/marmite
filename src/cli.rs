#![allow(clippy::struct_excessive_bools)]
use clap::{Args, Parser};
use std::path::PathBuf;

/// Command Line Argument Parser for Marmite CLI
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Input folder containing markdown files
    pub input_folder: PathBuf,

    /// Output folder to generate the site
    /// [default: `input_folder/site`]
    pub output_folder: Option<PathBuf>,

    /// Verbosity level (0-4)
    /// [default: 0 warn]
    /// options: -v: info,-vv: debug,-vvv: trace,-vvvv: trace all
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Detect changes and rebuild the site automatically
    #[arg(long, short, requires = "output_folder")]
    pub watch: bool,

    /// Serve the site with a built-in HTTP server
    #[arg(long)]
    pub serve: bool,

    /// Address to bind the server
    #[arg(long, default_value = "0.0.0.0:8000", requires = "serve")]
    pub bind: String,

    /// Path to custom configuration file
    #[arg(long, short, default_value = "marmite.yaml")]
    pub config: String,

    /// Print debug messages
    /// Deprecated: Use -vv for debug messages
    #[arg(long, hide = true)]
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

    /// Init a new site with sample content and default configuration
    /// this will overwrite existing files
    /// usually you don't need to run this because  
    /// Marmite can generate a site from any folder with markdown files
    #[arg(long)]
    pub init_site: bool,

    /// Create a new markdown file in the input folder
    #[command(flatten)]
    pub create: Create,

    /// Override configuration values from CLI arguments
    #[command(flatten)]
    pub configuration: Configuration,
}

/// Create a new markdown file in the input folder
#[derive(Args, Debug, Clone)]
pub struct Create {
    /// Create a new post with the given title and open in the default editor
    #[arg(long)]
    pub new: Option<String>,
    /// Edit the file in the default editor
    #[arg(short, requires = "new")]
    pub edit: bool,
    /// Set the new content as a page
    #[arg(short, requires = "new")]
    pub page: bool,
    /// Set the tags for the new content
    /// tags are comma separated
    #[arg(short, requires = "new")]
    pub tags: Option<String>,
}

/// Gather configuration values from CLI arguments
#[derive(Args, Debug, Clone)]
pub struct Configuration {
    /// Site name [default: "Home" or value from config file]
    #[arg(long)]
    pub name: Option<String>,

    /// Site tagline [default: empty or value from config file]
    #[arg(long)]
    pub tagline: Option<String>,

    /// Site url [default: empty or value from config file]
    #[arg(long)]
    pub url: Option<String>,

    /// Site footer [default: from '_footer.md' or config file]
    #[arg(long)]
    pub footer: Option<String>,

    /// Site main language 2 letter code [default: "en" or value from config file]
    #[arg(long)]
    pub language: Option<String>,

    /// Number of posts per page [default: 10 or value from config file]
    #[arg(long)]
    pub pagination: Option<usize>,

    /// Enable search [default: false or from config file]
    #[arg(long)]
    pub enable_search: Option<bool>,

    /// Path for content subfolder [default: "content" or value from config file]
    /// this is the folder where markdown files are stored inside `input_folder`
    /// no need to change this if your markdown files are in `input_folder` directly
    #[arg(long)]
    pub content_path: Option<String>,

    /// Path for templates subfolder [default: "templates" or value from config file]
    #[arg(long)]
    pub templates_path: Option<String>,

    /// Path for static subfolder [default: "static" or value from config file]
    #[arg(long)]
    pub static_path: Option<String>,

    /// Path for media subfolder [default: "media" or value from config file]
    /// this path is relative to the folder where your content files are
    #[arg(long)]
    pub media_path: Option<String>,

    /// Default date format [default: "%b %e, %Y" or from config file]
    /// see <https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html>
    #[arg(long)]
    pub default_date_format: Option<String>,

    /// Name of the colorscheme to use [default: "default" or from config file]
    /// see <https://rochacbruno.github.io/marmite/getting-started.html#colorschemes>
    #[arg(long)]
    pub colorscheme: Option<String>,

    /// Show Table of Contents in posts [default: false or from config file]
    /// this will generate a table of contents for each post
    #[arg(long)]
    pub toc: Option<bool>,
}
