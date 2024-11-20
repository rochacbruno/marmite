use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Marmite {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default)]
    pub tagline: String,
    #[serde(default)]
    pub url: String,
    #[serde(default = "default_footer")]
    pub footer: String,
    #[serde(default = "default_pagination")]
    pub pagination: usize,
    #[serde(default = "default_pages_title")]
    pub pages_title: String,
    #[serde(default = "default_tags_title")]
    pub tags_title: String,
    #[serde(default = "default_tags_content_title")]
    pub tags_content_title: String,
    #[serde(default = "default_archives_title")]
    pub archives_title: String,
    #[serde(default = "default_archives_content_title")]
    pub archives_content_title: String,
    #[serde(default = "default_authors_title")]
    pub authors_title: String,
    #[serde(default = "default_streams_title")]
    pub streams_title: String,
    #[serde(default = "default_search_title")]
    pub search_title: String,
    #[serde(default = "default_streams_content_title")]
    pub streams_content_title: String,
    #[serde(default = "default_content_path")]
    pub content_path: String,
    #[serde(default)]
    pub site_path: String,
    #[serde(default = "default_templates_path")]
    pub templates_path: String,
    #[serde(default = "default_static_path")]
    pub static_path: String,
    #[serde(default = "default_media_path")]
    pub media_path: String,
    #[serde(default)]
    pub card_image: String,
    #[serde(default)]
    pub banner_image: String,
    #[serde(default)]
    pub logo_image: String,
    #[serde(default)]
    pub enable_search: bool,
    #[serde(default = "default_date_format")]
    pub default_date_format: String,
    #[serde(default = "default_menu")]
    pub menu: Option<Vec<(String, String)>>,
    #[serde(default)]
    pub extra: Option<HashMap<String, Value>>,
    #[serde(default)]
    pub authors: HashMap<String, Author>,
    #[serde(default)]
    pub default_author: String,
}

impl Marmite {
    fn new() -> Self {
        Marmite {
            name: default_name(),
            footer: default_footer(),
            pagination: default_pagination(),
            pages_title: default_pages_title(),
            tags_title: default_tags_title(),
            tags_content_title: default_tags_content_title(),
            archives_title: default_archives_title(),
            archives_content_title: default_archives_content_title(),
            authors_title: default_authors_title(),
            streams_title: default_streams_title(),
            content_path: default_content_path(),
            templates_path: default_templates_path(),
            static_path: default_static_path(),
            media_path: default_media_path(),
            default_date_format: default_date_format(),
            menu: default_menu(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Author {
    pub name: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub links: Option<Vec<(String, String)>>,
}

/// Generates a default configuration file
/// this function writes to `marmite.yaml` in the input folder
/// the YAML file will contain the default configuration
/// default configuration is taken from serde default values
pub fn generate(input_folder: &Path) {
    let config_path = input_folder.join("marmite.yaml");
    // If the file already exists, do not overwrite
    if config_path.exists() {
        eprintln!("Config file already exists: {config_path:?}");
        return;
    }
    let config = Marmite::new();
    let config_str = serde_yaml::to_string(&config).unwrap();
    std::fs::write(&config_path, config_str).unwrap();
    println!("Config file generated: {:?}", &config_path.display());
}

// Defaults

fn default_name() -> String {
    "Home".to_string()
}

fn default_footer() -> String {
    r#"<div>Powered by <a href="https://github.com/rochacbruno/marmite">Marmite</a> | <small><a href="https://creativecommons.org/licenses/by-nc-sa/4.0/">CC-BY_NC-SA</a></small></div>"#.to_string()
}

fn default_pagination() -> usize {
    10usize
}

fn default_authors_title() -> String {
    "Authors".to_string()
}

fn default_tags_title() -> String {
    "Tags".to_string()
}

fn default_tags_content_title() -> String {
    "Posts tagged with '$tag'".to_string()
}

fn default_streams_content_title() -> String {
    "Posts from '$stream'".to_string()
}

fn default_pages_title() -> String {
    "Pages".to_string()
}

fn default_archives_title() -> String {
    "Archive".to_string()
}

fn default_streams_title() -> String {
    "Streams".to_string()
}

fn default_archives_content_title() -> String {
    "Posts from '$year'".to_string()
}

fn default_content_path() -> String {
    "content".to_string()
}

fn default_templates_path() -> String {
    "templates".to_string()
}

fn default_static_path() -> String {
    "static".to_string()
}

fn default_media_path() -> String {
    "media".to_string()
}

fn default_date_format() -> String {
    "%b %e, %Y".to_string()
}

fn default_menu() -> Option<Vec<(String, String)>> {
    vec![
        ("Tags".to_string(), "tags.html".to_string()),
        ("Archive".to_string(), "archive.html".to_string()),
        ("Authors".to_string(), "authors.html".to_string()),
        ("Streams".to_string(), "streams.html".to_string()),
    ]
    .into()
}

fn default_search_title() -> String {
    "Search".to_string()
}
