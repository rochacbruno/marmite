use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub struct Site<'a> {
    pub name: &'a str,
    pub tagline: &'a str,
    pub url: &'a str,
    pub footer: &'a str,
    pub content_path: &'a str,
    pub templates_path: &'a str,
    pub static_path: &'a str,
    pub site_path: &'a str,
}

pub struct SiteData<'a> {
    pub site: &'a Site<'a>,
    pub posts: Vec<Content>,
    pub pages: Vec<Content>,
}

impl<'a> SiteData<'a> {
    pub fn new(site: &'a Site) -> Self {
        SiteData {
            site,
            posts: Vec::new(),
            pages: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Content {
    pub title: String,
    pub slug: String,
    pub html: String,
    pub tags: Vec<String>,
    pub date: Option<NaiveDateTime>,
    pub show_in_menu: bool,
}
