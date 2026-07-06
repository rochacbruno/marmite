use super::*;
use std::sync::Arc;

fn test_cli() -> crate::cli::Cli {
    crate::cli::Cli {
        input_folder: Some(std::path::PathBuf::from("test")),
        output_folder: None,
        verbose: 0,
        watch: false,
        serve: false,
        bind: "0.0.0.0:8000".to_string(),
        config: "marmite.yaml".to_string(),
        debug: false,
        init_templates: false,
        start_theme: None,
        set_theme: None,
        generate_config: false,
        init_site: false,
        force: false,
        shortcodes: false,
        show_urls: false,
        skill: false,
        skill_install: false,
        skill_install_claude: false,
        create: crate::cli::Create {
            new: None,
            edit: false,
            page: false,
            tags: None,
            directory: None,
            lang: None,
            translates: None,
            site: None,
        },
        configuration: crate::cli::Configuration {
            name: None,
            tagline: None,
            url: None,
            https: None,
            footer: None,
            language: None,
            pagination: None,
            enable_search: None,
            search_show_matches: None,
            search_match_count: None,
            enable_related_content: None,
            content_path: None,
            templates_path: None,
            static_path: None,
            media_path: None,
            default_date_format: None,
            colorscheme: None,
            toc: None,
            json_feed: None,
            show_next_prev_links: None,
            publish_md: None,
            source_repository: None,
            image_provider: None,
            theme: None,
            build_sitemap: None,
            publish_urls_json: None,
            enable_shortcodes: None,
            shortcode_pattern: None,
            skip_image_resize: None,
            check_internal_links: None,
            strict_internal_links: None,
            native_mermaid_render: None,
        },
        subcommand: None,
    }
}

#[test]
fn test_marmite_new_defaults() {
    let m = Marmite::new();
    assert_eq!(m.name, "Home");
    assert_eq!(m.pagination, 10);
    assert_eq!(m.content_path, "content");
    assert_eq!(m.templates_path, "templates");
    assert_eq!(m.static_path, "static");
    assert_eq!(m.media_path, "media");
    assert_eq!(m.default_date_format, "%b %e, %Y");
    assert_eq!(m.pages_title, "Pages");
    assert_eq!(m.tags_title, "Tags");
    assert_eq!(m.archives_title, "Archive");
    assert_eq!(m.streams_title, "Streams");
    assert_eq!(m.series_title, "Series");
    assert_eq!(m.authors_title, "Authors");
    assert!(m.show_next_prev_links);
    assert!(m.enable_shortcodes);
    assert!(m.build_sitemap);
    assert!(m.publish_urls_json);
    assert!(m.gallery_create_thumbnails);
    assert_eq!(m.gallery_thumb_size, 50);
    assert_eq!(m.gallery_path, "gallery");
    assert!(!m.toc);
    assert!(!m.json_feed);
    assert!(!m.publish_md);
    assert!(!m.enable_search);
    assert!(!m.skip_image_resize);
    assert!(!m.check_internal_links);
    assert!(!m.strict_internal_links);
    assert!(m.native_mermaid_render);
}

#[test]
fn test_marmite_serde_defaults_language() {
    let m: Marmite = serde_yaml::from_str("{}").unwrap();
    assert_eq!(m.language, "en");
}

#[test]
fn test_marmite_serde_roundtrip() {
    let original = Marmite::new();
    let yaml = serde_yaml::to_string(&original).unwrap();
    let deserialized: Marmite = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_marmite_deserialize_empty_yaml() {
    let m: Marmite = serde_yaml::from_str("{}").unwrap();
    assert_eq!(m.name, "Home");
    assert_eq!(m.pagination, 10);
    assert_eq!(m.language, "en");
    assert_eq!(m.content_path, "content");
}

#[test]
fn test_marmite_deserialize_partial_yaml() {
    let m: Marmite = serde_yaml::from_str("name: MySite\npagination: 20\n").unwrap();
    assert_eq!(m.name, "MySite");
    assert_eq!(m.pagination, 20);
    assert_eq!(m.language, "en");
    assert_eq!(m.content_path, "content");
}

#[test]
fn test_override_from_cli_args_name() {
    let mut m = Marmite::new();
    let mut args = test_cli();
    args.configuration.name = Some("CLI Name".to_string());
    m.override_from_cli_args(&Arc::new(args));
    assert_eq!(m.name, "CLI Name");
}

#[test]
fn test_override_from_cli_args_pagination() {
    let mut m = Marmite::new();
    let mut args = test_cli();
    args.configuration.pagination = Some(25);
    m.override_from_cli_args(&Arc::new(args));
    assert_eq!(m.pagination, 25);
}

#[test]
fn test_override_from_cli_args_multiple() {
    let mut m = Marmite::new();
    let mut args = test_cli();
    args.configuration.name = Some("My Blog".to_string());
    args.configuration.language = Some("pt".to_string());
    args.configuration.pagination = Some(5);
    args.configuration.url = Some("https://example.com".to_string());
    args.configuration.enable_search = Some(true);
    args.configuration.toc = Some(true);
    m.override_from_cli_args(&Arc::new(args));
    assert_eq!(m.name, "My Blog");
    assert_eq!(m.language, "pt");
    assert_eq!(m.pagination, 5);
    assert_eq!(m.url, "https://example.com");
    assert!(m.enable_search);
    assert!(m.toc);
}

#[test]
fn test_override_from_cli_args_none_leaves_defaults() {
    let original = Marmite::new();
    let mut m = Marmite::new();
    let args = test_cli();
    m.override_from_cli_args(&Arc::new(args));
    assert_eq!(m, original);
}

#[test]
fn test_code_highlight_config_default() {
    let c = CodeHighlightConfig::default();
    assert!(c.enabled);
    assert_eq!(c.light_theme, "github-light");
    assert_eq!(c.dark_theme, "github-dark");
}

#[test]
fn test_render_options_default() {
    let r = RenderOptions::default();
    assert!(r.unsafe_);
    assert!(r.ignore_empty_links);
    assert!(r.figure_with_caption);
}

#[test]
fn test_extension_options_default() {
    let e = ExtensionOptions::default();
    assert!(!e.tagfilter);
    assert!(e.strikethrough);
    assert!(e.table);
    assert!(e.autolink);
    assert!(e.tasklist);
    assert!(e.footnotes);
    assert!(e.description_lists);
    assert!(e.multiline_block_quotes);
    assert!(e.underline);
    assert!(e.spoiler);
    assert!(!e.greentext);
    assert!(e.shortcodes);
    assert!(e.wikilinks_title_before_pipe);
    assert!(!e.wikilinks_title_after_pipe);
    assert!(e.alerts);
}

#[test]
fn test_parse_options_default() {
    let p = ParseOptions::default();
    assert!(p.relaxed_tasklist_matching);
}

#[test]
fn test_override_from_cli_args_image_provider() {
    let mut m = Marmite::new();
    let mut args = test_cli();
    args.configuration.image_provider = Some("picsum".to_string());
    m.override_from_cli_args(&Arc::new(args));
    assert_eq!(m.image_provider, Some(ImageProvider::Picsum));
}

#[test]
fn test_override_from_cli_args_theme() {
    let mut m = Marmite::new();
    let mut args = test_cli();
    args.configuration.theme = Some("my-theme".to_string());
    m.override_from_cli_args(&Arc::new(args));
    assert_eq!(m.theme, Some("my-theme".to_string()));
}

#[test]
fn test_override_from_cli_args_colorscheme() {
    let mut m = Marmite::new();
    let mut args = test_cli();
    args.configuration.colorscheme = Some("dracula".to_string());
    m.override_from_cli_args(&Arc::new(args));
    let extra = m.extra.unwrap();
    assert_eq!(
        extra.get("colorscheme"),
        Some(&serde_yaml::Value::String("dracula".to_string()))
    );
}

#[test]
fn test_native_mermaid_render_defaults_to_true() {
    let config: Marmite = serde_yaml::from_str("name: Test").unwrap();
    assert!(config.native_mermaid_render);
}

#[test]
fn test_native_mermaid_render_disabled_from_yaml() {
    let config: Marmite = serde_yaml::from_str("native_mermaid_render: false").unwrap();
    assert!(!config.native_mermaid_render);
}

#[test]
fn test_override_from_cli_args_native_mermaid_render() {
    let mut m = Marmite::new();
    let mut args = test_cli();
    args.configuration.native_mermaid_render = Some(false);
    m.override_from_cli_args(&Arc::new(args));
    assert!(!m.native_mermaid_render);
}
