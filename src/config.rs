use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct Marmite<'a> {
    #[serde(default = "default_name")]
    pub name: &'a str,
    #[serde(default = "default_tagline")]
    pub tagline: &'a str,
    #[serde(default = "default_url")]
    pub url: &'a str,
    #[serde(default = "default_footer")]
    pub footer: &'a str,
    #[serde(default = "default_pagination")]
    pub pagination: u32,

    #[serde(default = "default_list_title")]
    pub list_title: &'a str,
    #[serde(default = "default_pages_title")]
    pub pages_title: &'a str,
    #[serde(default = "default_tags_title")]
    pub tags_title: &'a str,
    #[serde(default = "default_archives_title")]
    pub archives_title: &'a str,

    #[serde(default = "default_content_path")]
    pub content_path: &'a str,
    #[serde(default = "default_site_path")]
    pub site_path: &'a str,
    #[serde(default = "default_templates_path")]
    pub templates_path: &'a str,
    #[serde(default = "default_static_path")]
    pub static_path: &'a str,
    #[serde(default = "default_media_path")]
    pub media_path: &'a str,

    #[serde(default = "default_card_image")]
    pub card_image: &'a str,
    #[serde(default = "default_logo_image")]
    pub logo_image: &'a str,

    #[serde(default = "default_menu")]
    pub menu: Option<Vec<(String, String)>>,

    #[serde(default = "default_data")]
    pub data: Option<HashMap<String, Value>>,
}

fn default_name() -> &'static str {
    "Home"
}

fn default_tagline() -> &'static str {
    "Site generated from markdown content"
}

fn default_url() -> &'static str {
    ""
}

fn default_footer() -> &'static str {
    r#"<a href="https://creativecommons.org/licenses/by-nc-sa/4.0/">CC-BY_NC-SA</a> | Site generated with <a href="https://github.com/rochacbruno/marmite">Marmite</a>"#
}

fn default_pagination() -> u32 {
    10
}

fn default_list_title() -> &'static str {
    "Posts"
}

fn default_tags_title() -> &'static str {
    "Tags"
}

fn default_pages_title() -> &'static str {
    "Pages"
}

fn default_archives_title() -> &'static str {
    "Archive"
}

fn default_site_path() -> &'static str {
    ""
}

fn default_content_path() -> &'static str {
    "content"
}

fn default_templates_path() -> &'static str {
    "templates"
}

fn default_static_path() -> &'static str {
    "static"
}

fn default_media_path() -> &'static str {
    "media"
}

fn default_card_image() -> &'static str {
    ""
}

fn default_logo_image() -> &'static str {
    ""
}

fn default_menu() -> Option<Vec<(String, String)>> {
    vec![("Pages".to_string(), "pages.html".to_string())].into()
}

fn default_data() -> Option<HashMap<String, Value>> {
    None
}
