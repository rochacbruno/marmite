use structopt::StructOpt;
use std::path::PathBuf;

#[derive(StructOpt, Debug)]
#[structopt(name = "Marmite Site Generator", about = "Generates static websites from markdown content")]
pub struct Cli {
    /// The root folder of the site
    pub folder: Option<String>,

    /// Serve the website in editing mode
    #[structopt(long)]
    pub serve: bool,

    /// Initialize the folder structure
    #[structopt(long)]
    pub init: Option<String>,

    /// Custom configuration file path
    #[structopt(long, value_name = "FILE")]
    pub config: Option<String>,
}

impl Cli {
    pub fn get_folder(&self) -> PathBuf {
        if let Some(folder_name) = &self.init {
            PathBuf::from(folder_name)
        } else if self.serve {
            PathBuf::from(self.folder.as_ref().map(|s| s.as_str()).unwrap_or("."))
        } else {
            PathBuf::from(self.folder.as_ref().expect("Folder argument is required unless initializing a project or serving."))
        }
    }

    pub fn get_config_path(&self) -> PathBuf {
        self.config
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("marmite.yaml"))
    }
}
