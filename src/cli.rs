use clap::Parser;

/// Marmite Site Generator
/// Generates static websites from markdown content
#[derive(Parser, Debug)]
#[command(name = "Marmite Site Generator", about = "Generates static websites from markdown content")]
pub struct Cli {
    /// The root folder of the site (input folder)
    pub input_folder: Option<String>,

    /// Build the website (render markdown to HTML)
    #[arg(long)]
    pub build: bool,

    /// Serve the website in editing mode
    #[arg(long)]
    pub serve: bool,

    /// Custom configuration file path
    #[arg(long, value_name = "FILE")]
    pub config: Option<String>,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
