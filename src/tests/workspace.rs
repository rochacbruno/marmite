use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_detect_workspace_present() {
    let dir = TempDir::new().unwrap();
    let ws_path = dir.path().join(WORKSPACE_CONFIG_FILENAME);
    fs::write(&ws_path, "sites:\n  - name: blog\n").unwrap();
    assert!(detect_workspace(dir.path()).is_some());
}

#[test]
fn test_detect_workspace_absent() {
    let dir = TempDir::new().unwrap();
    assert!(detect_workspace(dir.path()).is_none());
}

#[test]
fn test_load_workspace_config_minimal() {
    let dir = TempDir::new().unwrap();
    let ws_path = dir.path().join(WORKSPACE_CONFIG_FILENAME);
    fs::write(&ws_path, "sites:\n  - name: blog\n").unwrap();
    let config = load_workspace_config(&ws_path).unwrap();
    assert_eq!(config.sites.len(), 1);
    assert_eq!(config.sites[0].name, "blog");
    assert!(config.default_site.is_none());
    assert!(!config.redirect);
    assert_eq!(config.separator, "::");
}

#[test]
fn test_load_workspace_config_full() {
    let dir = TempDir::new().unwrap();
    let ws_path = dir.path().join(WORKSPACE_CONFIG_FILENAME);
    fs::write(
        &ws_path,
        r#"
sites:
  - name: blog
    output_path: b
  - name: photos
default_site: blog
redirect: true
separator: "::"
defaults:
  language: pt
  pagination: 5
"#,
    )
    .unwrap();
    let config = load_workspace_config(&ws_path).unwrap();
    assert_eq!(config.sites.len(), 2);
    assert_eq!(config.sites[0].resolved_output_path(), "b");
    assert_eq!(config.sites[1].resolved_output_path(), "photos");
    assert_eq!(config.default_site.as_deref(), Some("blog"));
    assert!(config.redirect);
    let defaults = config.defaults.unwrap();
    assert_eq!(defaults.language, "pt");
    assert_eq!(defaults.pagination, 5);
}

#[test]
fn test_load_workspace_config_empty_sites_errors() {
    let dir = TempDir::new().unwrap();
    let ws_path = dir.path().join(WORKSPACE_CONFIG_FILENAME);
    fs::write(&ws_path, "sites: []\n").unwrap();
    assert!(load_workspace_config(&ws_path).is_err());
}

#[test]
fn test_resolved_default_site() {
    let config = WorkspaceConfig {
        sites: vec![
            WorkspaceSiteEntry {
                name: "a".into(),
                output_path: None,
            },
            WorkspaceSiteEntry {
                name: "b".into(),
                output_path: None,
            },
        ],
        default_site: None,
        redirect: false,
        defaults: None,
        separator: "::".into(),
    };
    assert_eq!(config.resolved_default_site(), Some("a"));

    let config2 = WorkspaceConfig {
        default_site: Some("b".into()),
        ..config
    };
    assert_eq!(config2.resolved_default_site(), Some("b"));
}

#[test]
fn test_deep_merge_yaml_scalars() {
    let base: serde_yaml::Value = serde_yaml::from_str("a: 1\nb: 2").unwrap();
    let overlay: serde_yaml::Value = serde_yaml::from_str("b: 3\nc: 4").unwrap();
    let merged = deep_merge_yaml(base, overlay);
    let map = merged.as_mapping().unwrap();
    assert_eq!(
        map.get(&serde_yaml::Value::String("a".into())),
        Some(&serde_yaml::Value::Number(1.into()))
    );
    assert_eq!(
        map.get(&serde_yaml::Value::String("b".into())),
        Some(&serde_yaml::Value::Number(3.into()))
    );
    assert_eq!(
        map.get(&serde_yaml::Value::String("c".into())),
        Some(&serde_yaml::Value::Number(4.into()))
    );
}

#[test]
fn test_deep_merge_yaml_nested() {
    let base: serde_yaml::Value = serde_yaml::from_str("top:\n  a: 1\n  b: 2\nother: x").unwrap();
    let overlay: serde_yaml::Value = serde_yaml::from_str("top:\n  b: 3\n  c: 4").unwrap();
    let merged = deep_merge_yaml(base, overlay);
    let top = merged
        .as_mapping()
        .unwrap()
        .get(&serde_yaml::Value::String("top".into()))
        .unwrap()
        .as_mapping()
        .unwrap();
    assert_eq!(
        top.get(&serde_yaml::Value::String("a".into())),
        Some(&serde_yaml::Value::Number(1.into()))
    );
    assert_eq!(
        top.get(&serde_yaml::Value::String("b".into())),
        Some(&serde_yaml::Value::Number(3.into()))
    );
    assert_eq!(
        top.get(&serde_yaml::Value::String("c".into())),
        Some(&serde_yaml::Value::Number(4.into()))
    );
    assert!(merged
        .as_mapping()
        .unwrap()
        .get(&serde_yaml::Value::String("other".into()))
        .is_some());
}

fn make_site_data(name: &str, output_path: &str) -> SiteData {
    SiteData {
        name: name.to_string(),
        output_path: output_path.to_string(),
        data: crate::site::Data::new("", &std::path::PathBuf::from("test.yaml")),
    }
}

#[test]
fn test_resolve_cross_site_refs_href() {
    let mut sites = std::collections::HashMap::new();
    sites.insert("blog".to_string(), make_site_data("blog", "blog"));
    let cross_site = CrossSiteData {
        sites,
        separator: "::".to_string(),
    };

    let html = r#"<a href="blog::my-post.html">link</a>"#;
    let result = resolve_cross_site_refs(html, &cross_site);
    assert_eq!(result, r#"<a href="/blog/my-post.html">link</a>"#);
}

#[test]
fn test_resolve_cross_site_refs_src() {
    let mut sites = std::collections::HashMap::new();
    sites.insert("photos".to_string(), make_site_data("photos", "photos"));
    let cross_site = CrossSiteData {
        sites,
        separator: "::".to_string(),
    };

    let html = r#"<img src="photos::media/pic.jpg" />"#;
    let result = resolve_cross_site_refs(html, &cross_site);
    assert_eq!(result, r#"<img src="/photos/media/pic.jpg" />"#);
}

#[test]
fn test_resolve_cross_site_refs_unknown_site() {
    let cross_site = CrossSiteData {
        sites: std::collections::HashMap::new(),
        separator: "::".to_string(),
    };

    let html = r#"<a href="unknown::page.html">link</a>"#;
    let result = resolve_cross_site_refs(html, &cross_site);
    assert_eq!(result, html);
}

#[test]
fn test_resolve_cross_site_refs_no_match() {
    let cross_site = CrossSiteData {
        sites: std::collections::HashMap::new(),
        separator: "::".to_string(),
    };

    let html = r#"<a href="https://example.com">link</a>"#;
    let result = resolve_cross_site_refs(html, &cross_site);
    assert_eq!(result, html);
}

#[test]
fn test_merge_site_config_empty_site() {
    let dir = TempDir::new().unwrap();
    let site_config = dir.path().join("marmite.yaml");
    fs::write(&site_config, "").unwrap();

    let defaults = crate::config::Marmite {
        language: "pt".to_string(),
        pagination: 5,
        ..Default::default()
    };
    let cli_args = std::sync::Arc::new(test_cli());
    let merged = merge_site_config(Some(&defaults), &site_config, &cli_args);
    assert_eq!(merged.language, "pt");
    assert_eq!(merged.pagination, 5);
}

#[test]
fn test_merge_site_config_override() {
    let dir = TempDir::new().unwrap();
    let site_config = dir.path().join("marmite.yaml");
    fs::write(&site_config, "language: fr\n").unwrap();

    let defaults = crate::config::Marmite {
        language: "pt".to_string(),
        pagination: 5,
        ..Default::default()
    };
    let cli_args = std::sync::Arc::new(test_cli());
    let merged = merge_site_config(Some(&defaults), &site_config, &cli_args);
    assert_eq!(merged.language, "fr");
    assert_eq!(merged.pagination, 5);
}

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
        },
        subcommand: None,
    }
}
