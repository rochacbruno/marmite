use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::{collections::HashMap, path::Path, process, sync::Arc};

use crate::cli::Cli;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ImageProvider {
    #[serde(rename = "picsum")]
    Picsum,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct RenderOptions {
    #[serde(rename = "unsafe")]
    #[serde(default = "default_render_unsafe")]
    pub unsafe_: bool,
    #[serde(default = "default_render_ignore_empty_links")]
    pub ignore_empty_links: bool,
    #[serde(default = "default_render_figure_with_caption")]
    pub figure_with_caption: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ParseOptions {
    #[serde(default = "default_parse_relaxed_tasklist_matching")]
    pub relaxed_tasklist_matching: bool,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ExtensionOptions {
    #[serde(default = "default_extension_tagfilter")]
    pub tagfilter: bool,
    #[serde(default = "default_extension_strikethrough")]
    pub strikethrough: bool,
    #[serde(default = "default_extension_table")]
    pub table: bool,
    #[serde(default = "default_extension_autolink")]
    pub autolink: bool,
    #[serde(default = "default_extension_tasklist")]
    pub tasklist: bool,
    #[serde(default = "default_extension_footnotes")]
    pub footnotes: bool,
    #[serde(default = "default_extension_description_lists")]
    pub description_lists: bool,
    #[serde(default = "default_extension_multiline_block_quotes")]
    pub multiline_block_quotes: bool,
    #[serde(default = "default_extension_underline")]
    pub underline: bool,
    #[serde(default = "default_extension_spoiler")]
    pub spoiler: bool,
    #[serde(default = "default_extension_greentext")]
    pub greentext: bool,
    #[serde(default = "default_extension_shortcodes")]
    pub shortcodes: bool,
    #[serde(default = "default_extension_wikilinks_title_before_pipe")]
    pub wikilinks_title_before_pipe: bool,
    #[serde(default = "default_extension_wikilinks_title_after_pipe")]
    pub wikilinks_title_after_pipe: bool,
    #[serde(default = "default_extension_alerts")]
    pub alerts: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct ParserOptions {
    #[serde(default)]
    pub render: RenderOptions,
    #[serde(default)]
    pub parse: ParseOptions,
    #[serde(default)]
    pub extension: ExtensionOptions,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            unsafe_: default_render_unsafe(),
            ignore_empty_links: default_render_ignore_empty_links(),
            figure_with_caption: default_render_figure_with_caption(),
        }
    }
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            relaxed_tasklist_matching: default_parse_relaxed_tasklist_matching(),
        }
    }
}

impl Default for ExtensionOptions {
    fn default() -> Self {
        Self {
            tagfilter: default_extension_tagfilter(),
            strikethrough: default_extension_strikethrough(),
            table: default_extension_table(),
            autolink: default_extension_autolink(),
            tasklist: default_extension_tasklist(),
            footnotes: default_extension_footnotes(),
            description_lists: default_extension_description_lists(),
            multiline_block_quotes: default_extension_multiline_block_quotes(),
            underline: default_extension_underline(),
            spoiler: default_extension_spoiler(),
            greentext: default_extension_greentext(),
            shortcodes: default_extension_shortcodes(),
            wikilinks_title_before_pipe: default_extension_wikilinks_title_before_pipe(),
            wikilinks_title_after_pipe: default_extension_wikilinks_title_after_pipe(),
            alerts: default_extension_alerts(),
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct Marmite {
    #[serde(default = "default_name")]
    pub name: String,

    #[serde(default)]
    pub tagline: String,

    #[serde(default)]
    pub url: String,

    #[serde(default)]
    pub https: Option<bool>,

    #[serde(default = "default_footer")]
    pub footer: String,

    #[serde(default = "default_language")]
    pub language: String,

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

    #[serde(default = "default_streams_title")]
    pub streams_title: String,

    #[serde(default = "default_streams_content_title")]
    pub streams_content_title: String,

    #[serde(default = "default_series_title")]
    pub series_title: String,

    #[serde(default = "default_series_content_title")]
    pub series_content_title: String,

    #[serde(default)]
    pub default_author: String,

    #[serde(default = "default_authors_title")]
    pub authors_title: String,

    #[serde(default)]
    pub enable_search: bool,

    #[serde(default = "default_enable_related_content")]
    pub enable_related_content: bool,

    #[serde(default = "default_search_title")]
    pub search_title: String,

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

    #[serde(default = "default_date_format")]
    pub default_date_format: String,

    #[serde(default = "default_menu")]
    pub menu: Option<Vec<(String, String)>>,

    #[serde(default)]
    pub extra: Option<HashMap<String, Value>>,

    #[serde(default)]
    pub authors: HashMap<String, Author>,

    #[serde(default)]
    pub streams: HashMap<String, StreamConfig>,

    #[serde(default)]
    pub series: HashMap<String, SeriesConfig>,

    #[serde(default)]
    pub toc: bool,

    #[serde(default)]
    pub json_feed: bool,

    #[serde(default = "default_true")]
    pub show_next_prev_links: bool,

    #[serde(default)]
    pub publish_md: bool,

    #[serde(default)]
    pub source_repository: Option<String>,

    #[serde(default)]
    pub image_provider: Option<ImageProvider>,

    #[serde(default)]
    pub markdown_parser: Option<ParserOptions>,

    #[serde(default)]
    pub theme: Option<String>,

    #[serde(default)]
    pub file_mapping: Vec<FileMapping>,

    #[serde(default = "default_true")]
    pub enable_shortcodes: bool,

    #[serde(default)]
    pub shortcode_pattern: Option<String>,

    #[serde(default = "default_true")]
    pub build_sitemap: bool,

    #[serde(default = "default_true")]
    pub publish_urls_json: bool,

    #[serde(default = "default_gallery_path")]
    pub gallery_path: String,

    #[serde(default = "default_true")]
    pub gallery_create_thumbnails: bool,

    #[serde(default = "default_gallery_thumb_size")]
    pub gallery_thumb_size: u32,

    /// Skip image resizing during build (faster development builds)
    #[serde(default)]
    pub skip_image_resize: bool,
}

fn default_true() -> bool {
    true
}

impl Marmite {
    pub fn new() -> Self {
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
            streams_content_title: default_streams_content_title(),
            series_title: default_series_title(),
            series_content_title: default_series_content_title(),
            content_path: default_content_path(),
            templates_path: default_templates_path(),
            static_path: default_static_path(),
            media_path: default_media_path(),
            default_date_format: default_date_format(),
            menu: default_menu(),
            show_next_prev_links: default_true(),
            enable_shortcodes: default_true(),
            build_sitemap: default_true(),
            publish_urls_json: default_true(),
            gallery_path: default_gallery_path(),
            gallery_create_thumbnails: default_true(),
            gallery_thumb_size: default_gallery_thumb_size(),
            ..Default::default()
        }
    }

    /// Get the resolved templates path based on theme configuration
    pub fn get_templates_path(&self, input_folder: &Path) -> std::path::PathBuf {
        if let Some(theme) = &self.theme {
            let theme_path = input_folder.join(theme);
            if !theme_path.exists() {
                error!("Theme folder '{}' does not exist", theme_path.display());
                process::exit(1);
            }
            theme_path.join(&self.templates_path)
        } else {
            input_folder.join(&self.templates_path)
        }
    }

    /// Get the resolved static path based on theme configuration
    pub fn get_static_path(&self, input_folder: &Path) -> std::path::PathBuf {
        if let Some(theme) = &self.theme {
            let theme_path = input_folder.join(theme);
            if !theme_path.exists() {
                error!("Theme folder '{}' does not exist", theme_path.display());
                process::exit(1);
            }
            theme_path.join(&self.static_path)
        } else {
            input_folder.join(&self.static_path)
        }
    }

    pub fn override_from_cli_args(&mut self, cli_args: &Arc<Cli>) {
        if let Some(name) = &cli_args.configuration.name {
            self.name.clone_from(name);
        }
        if let Some(tagline) = &cli_args.configuration.tagline {
            self.tagline.clone_from(tagline);
        }
        if let Some(url) = &cli_args.configuration.url {
            self.url.clone_from(url);
        }
        if let Some(https) = &cli_args.configuration.https {
            self.https = Some(*https);
        }
        if let Some(footer) = &cli_args.configuration.footer {
            self.footer.clone_from(footer);
        }
        if let Some(language) = &cli_args.configuration.language {
            self.language.clone_from(language);
        }
        if let Some(pagination) = cli_args.configuration.pagination {
            self.pagination = pagination;
        }
        if let Some(enable_search) = cli_args.configuration.enable_search {
            self.enable_search = enable_search;
        }
        if let Some(enable_related_content) = cli_args.configuration.enable_related_content {
            self.enable_related_content = enable_related_content;
        }
        if let Some(toc) = cli_args.configuration.toc {
            self.toc = toc;
        }
        if let Some(content_path) = &cli_args.configuration.content_path {
            self.content_path.clone_from(content_path);
        }
        if let Some(templates_path) = &cli_args.configuration.templates_path {
            self.templates_path.clone_from(templates_path);
        }
        if let Some(static_path) = &cli_args.configuration.static_path {
            self.static_path.clone_from(static_path);
        }
        if let Some(media_path) = &cli_args.configuration.media_path {
            self.media_path.clone_from(media_path);
        }
        if let Some(default_date_format) = &cli_args.configuration.default_date_format {
            self.default_date_format.clone_from(default_date_format);
        }
        if let Some(colorscheme) = &cli_args.configuration.colorscheme {
            self.extra = Some(
                [(
                    "colorscheme".to_string(),
                    Value::String(colorscheme.clone()),
                )]
                .iter()
                .cloned()
                .collect(),
            );
        }
        if let Some(json_feed) = cli_args.configuration.json_feed {
            self.json_feed = json_feed;
        }
        if let Some(show_next_prev_links) = cli_args.configuration.show_next_prev_links {
            self.show_next_prev_links = show_next_prev_links;
        }
        if let Some(publish_md) = cli_args.configuration.publish_md {
            self.publish_md = publish_md;
        }
        if let Some(source_repository) = &cli_args.configuration.source_repository {
            self.source_repository = Some(source_repository.clone());
        }
        if let Some(image_provider_str) = &cli_args.configuration.image_provider {
            match image_provider_str.to_lowercase().as_str() {
                "picsum" => self.image_provider = Some(ImageProvider::Picsum),
                _ => {
                    eprintln!("Warning: Unknown image provider '{image_provider_str}'. Available providers: picsum");
                }
            }
        }
        if let Some(theme) = &cli_args.configuration.theme {
            self.theme = Some(theme.clone());
        }
        if let Some(build_sitemap) = cli_args.configuration.build_sitemap {
            self.build_sitemap = build_sitemap;
        }
        if let Some(publish_urls_json) = cli_args.configuration.publish_urls_json {
            self.publish_urls_json = publish_urls_json;
        }
        if let Some(enable_shortcodes) = cli_args.configuration.enable_shortcodes {
            self.enable_shortcodes = enable_shortcodes;
        }
        if let Some(shortcode_pattern) = &cli_args.configuration.shortcode_pattern {
            self.shortcode_pattern = Some(shortcode_pattern.clone());
        }
        if let Some(skip_image_resize) = cli_args.configuration.skip_image_resize {
            self.skip_image_resize = skip_image_resize;
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Author {
    pub name: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub links: Option<Vec<(String, String)>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct StreamConfig {
    pub display_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SeriesConfig {
    pub display_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct FileMapping {
    pub source: String,
    pub dest: String,
}

/// Generates a default configuration file
/// this function writes to `marmite.yaml` in the input folder
/// the YAML file will contain the default configuration
/// default configuration is taken from serde default values
pub fn generate(input_folder: &Path, cli_args: &Arc<Cli>) {
    let config_path = input_folder.join(cli_args.config.as_str());
    // If the file already exists, do not overwrite
    if config_path.exists() {
        error!("Config file already exists: {}", config_path.display());
        return;
    }
    let mut config = Marmite::new();
    config.override_from_cli_args(cli_args);
    let config_str = match serde_yaml::to_string(&config) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to serialize config: {e}");
            return;
        }
    };
    if let Err(e) = std::fs::write(&config_path, config_str) {
        error!("Failed to write config file: {e}");
        return;
    }
    info!("Config file generated: {}", config_path.display());
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

fn default_series_title() -> String {
    "Series".to_string()
}

fn default_series_content_title() -> String {
    "Posts from '$series' series".to_string()
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
        // ("Streams".to_string(), "streams.html".to_string()),
    ]
    .into()
}

fn default_search_title() -> String {
    "Search".to_string()
}

fn default_language() -> String {
    "en".to_string()
}

fn default_enable_related_content() -> bool {
    true
}

// Parser options defaults - matching parser.rs current values

fn default_render_unsafe() -> bool {
    true
}

fn default_render_ignore_empty_links() -> bool {
    true
}

fn default_render_figure_with_caption() -> bool {
    true
}

fn default_parse_relaxed_tasklist_matching() -> bool {
    true
}

fn default_extension_tagfilter() -> bool {
    false
}

fn default_extension_strikethrough() -> bool {
    true
}

fn default_extension_table() -> bool {
    true
}

fn default_extension_autolink() -> bool {
    true
}

fn default_extension_tasklist() -> bool {
    true
}

fn default_extension_footnotes() -> bool {
    true
}

fn default_extension_description_lists() -> bool {
    true
}

fn default_extension_multiline_block_quotes() -> bool {
    true
}

fn default_extension_underline() -> bool {
    true
}

fn default_extension_spoiler() -> bool {
    true
}

fn default_extension_greentext() -> bool {
    false
}

fn default_extension_shortcodes() -> bool {
    true
}

fn default_extension_wikilinks_title_before_pipe() -> bool {
    true
}

fn default_extension_wikilinks_title_after_pipe() -> bool {
    false
}

fn default_extension_alerts() -> bool {
    true
}

fn default_gallery_path() -> String {
    "gallery".to_string()
}

fn default_gallery_thumb_size() -> u32 {
    50
}
