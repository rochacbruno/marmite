use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Marmite {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_tagline")]
    pub tagline: String,
    #[serde(default = "default_url")]
    pub url: String,
    #[serde(default = "default_footer")]
    pub footer: String,
    #[serde(default = "default_pagination")]
    pub pagination: u32,

    #[serde(default = "default_list_title")]
    pub list_title: String,
    #[serde(default = "default_pages_title")]
    pub pages_title: String,
    #[serde(default = "default_tags_title")]
    pub tags_title: String,
    #[serde(default = "default_tags_content_title")]
    pub tags_content_title: String,
    #[serde(default = "default_archives_title")]
    pub archives_title: String,

    #[serde(default = "default_content_path")]
    pub content_path: String,
    #[serde(default = "default_site_path")]
    pub site_path: String,
    #[serde(default = "default_templates_path")]
    pub templates_path: String,
    #[serde(default = "default_static_path")]
    pub static_path: String,
    #[serde(default = "default_media_path")]
    pub media_path: String,

    #[serde(default = "default_card_image")]
    pub card_image: String,
    #[serde(default = "default_logo_image")]
    pub logo_image: String,

    #[serde(default = "default_menu")]
    pub menu: Option<Vec<(String, String)>>,

    #[serde(default = "default_data")]
    pub data: Option<HashMap<String, Value>>,
}

fn default_name() -> String {
    "Home".to_string()
}

fn default_tagline() -> String {
    "Site generated from markdown content".to_string()
}

fn default_url() -> String {
    String::new()
}

fn default_footer() -> String {
    r#"<a href="https://creativecommons.org/licenses/by-nc-sa/4.0/">CC-BY_NC-SA</a> | Site generated with <a href="https://github.com/rochacbruno/marmite">Marmite</a>"#.to_string()
}

fn default_pagination() -> u32 {
    10
}

fn default_list_title() -> String {
    "Posts".to_string()
}

fn default_tags_title() -> String {
    "Tags".to_string()
}

fn default_tags_content_title() -> String {
    "Posts tagged with '$tag'".to_string()
}

fn default_pages_title() -> String {
    "Pages".to_string()
}

fn default_archives_title() -> String {
    "Archive".to_string()
}

fn default_site_path() -> String {
    String::new()
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

fn default_card_image() -> String {
    String::new()
}

fn default_logo_image() -> String {
    String::new()
}

fn default_menu() -> Option<Vec<(String, String)>> {
    vec![
        ("Pages".to_string(), "pages.html".to_string()),
        ("Tags".to_string(), "tags.html".to_string()),
    ]
    .into()
}

fn default_data() -> Option<HashMap<String, Value>> {
    None
}
