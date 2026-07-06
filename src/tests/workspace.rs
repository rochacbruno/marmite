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

// === resolve_site_url tests ===

#[test]
fn test_resolve_site_url_empty_base() {
    assert_eq!(resolve_site_url("", "blog"), None);
}

#[test]
fn test_resolve_site_url_base_only() {
    assert_eq!(
        resolve_site_url("https://example.com", ""),
        Some("https://example.com".to_string())
    );
}

#[test]
fn test_resolve_site_url_with_subpath() {
    assert_eq!(
        resolve_site_url("https://example.com", "blog"),
        Some("https://example.com/blog".to_string())
    );
}

#[test]
fn test_resolve_site_url_trailing_slash() {
    assert_eq!(
        resolve_site_url("https://example.com/", ""),
        Some("https://example.com".to_string())
    );
    assert_eq!(
        resolve_site_url("https://example.com/", "blog"),
        Some("https://example.com/blog".to_string())
    );
}

// === write_sites_json tests ===

#[test]
fn test_write_sites_json() {
    let dir = TempDir::new().unwrap();
    let ws_config = WorkspaceConfig {
        sites: vec![
            WorkspaceSiteEntry {
                name: "blog".into(),
                output_path: Some("b".into()),
            },
            WorkspaceSiteEntry {
                name: "photos".into(),
                output_path: None,
            },
        ],
        default_site: None,
        redirect: false,
        defaults: None,
        separator: "::".into(),
    };

    write_sites_json(&ws_config, dir.path()).unwrap();

    let content = fs::read_to_string(dir.path().join("sites.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    let sites = parsed.as_array().unwrap();
    assert_eq!(sites.len(), 2);
    assert_eq!(sites[0]["name"], "blog");
    assert_eq!(sites[0]["path"], "/b/");
    assert_eq!(sites[1]["name"], "photos");
    assert_eq!(sites[1]["path"], "/photos/");
}

// === write_workspace_build_info tests ===

#[test]
fn test_write_workspace_build_info() {
    let dir = TempDir::new().unwrap();
    let ws_config = WorkspaceConfig {
        sites: vec![
            WorkspaceSiteEntry {
                name: "blog".into(),
                output_path: None,
            },
            WorkspaceSiteEntry {
                name: "docs".into(),
                output_path: None,
            },
        ],
        default_site: None,
        redirect: false,
        defaults: None,
        separator: "::".into(),
    };

    let mut sites_map = std::collections::HashMap::new();
    let mut blog_data = crate::site::Data::new("", &std::path::PathBuf::from("test.yaml"));
    blog_data.posts.push(crate::content::Content::default());
    blog_data.posts.push(crate::content::Content::default());
    blog_data.pages.push(crate::content::Content::default());
    sites_map.insert(
        "blog".to_string(),
        SiteData {
            name: "blog".to_string(),
            output_path: "blog".to_string(),
            data: blog_data,
        },
    );

    let mut docs_data = crate::site::Data::new("", &std::path::PathBuf::from("test.yaml"));
    docs_data.pages.push(crate::content::Content::default());
    sites_map.insert(
        "docs".to_string(),
        SiteData {
            name: "docs".to_string(),
            output_path: "docs".to_string(),
            data: docs_data,
        },
    );

    let cross_site = CrossSiteData {
        sites: sites_map,
        separator: "::".to_string(),
    };

    write_workspace_build_info(&ws_config, &cross_site, dir.path()).unwrap();

    let content = fs::read_to_string(dir.path().join("marmite-workspace.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["workspace"], true);
    assert_eq!(parsed["total_posts"], 2);
    assert_eq!(parsed["total_pages"], 2);
    assert!(parsed["marmite_version"].is_string());
    assert!(parsed["generated_at"].is_string());
    let sites = parsed["sites"].as_array().unwrap();
    assert_eq!(sites.len(), 2);
}

// === write_workspace_urls_json tests ===

#[test]
fn test_write_workspace_urls_json() {
    let dir = TempDir::new().unwrap();

    let mut sites_map = std::collections::HashMap::new();
    let blog_data = crate::site::Data::new("", &std::path::PathBuf::from("test.yaml"));
    sites_map.insert(
        "blog".to_string(),
        SiteData {
            name: "blog".to_string(),
            output_path: "blog".to_string(),
            data: blog_data,
        },
    );

    let docs_data = crate::site::Data::new("", &std::path::PathBuf::from("test.yaml"));
    sites_map.insert(
        "docs".to_string(),
        SiteData {
            name: "docs".to_string(),
            output_path: "docs".to_string(),
            data: docs_data,
        },
    );

    let cross_site = CrossSiteData {
        sites: sites_map,
        separator: "::".to_string(),
    };

    write_workspace_urls_json(&cross_site, dir.path()).unwrap();

    let content = fs::read_to_string(dir.path().join("urls-workspace.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(parsed.is_object());
}

// === resolve_cross_site_refs edge case ===

#[test]
fn test_resolve_cross_site_refs_empty_output_path() {
    let mut sites = std::collections::HashMap::new();
    sites.insert("main".to_string(), make_site_data("main", ""));
    let cross_site = CrossSiteData {
        sites,
        separator: "::".to_string(),
    };

    let html = r#"<a href="main::my-page.html">link</a>"#;
    let result = resolve_cross_site_refs(html, &cross_site);
    assert_eq!(result, r#"<a href="/my-page.html">link</a>"#);
}

#[test]
fn test_resolve_cross_site_refs_multiple_refs() {
    let mut sites = std::collections::HashMap::new();
    sites.insert("blog".to_string(), make_site_data("blog", "blog"));
    sites.insert("docs".to_string(), make_site_data("docs", "docs"));
    let cross_site = CrossSiteData {
        sites,
        separator: "::".to_string(),
    };

    let html = r#"<a href="blog::post.html">post</a> and <img src="docs::media/img.png" />"#;
    let result = resolve_cross_site_refs(html, &cross_site);
    assert!(result.contains(r#"href="/blog/post.html""#));
    assert!(result.contains(r#"src="/docs/media/img.png""#));
}

// === merge_site_config edge cases ===

#[test]
fn test_merge_site_config_no_defaults() {
    let dir = TempDir::new().unwrap();
    let site_config = dir.path().join("marmite.yaml");
    fs::write(&site_config, "language: fr\npagination: 20\n").unwrap();

    let cli_args = std::sync::Arc::new(test_cli());
    let merged = merge_site_config(None, &site_config, &cli_args);
    assert_eq!(merged.language, "fr");
    assert_eq!(merged.pagination, 20);
}

#[test]
fn test_merge_site_config_missing_file() {
    let dir = TempDir::new().unwrap();
    let site_config = dir.path().join("nonexistent.yaml");

    let defaults = crate::config::Marmite {
        language: "pt".to_string(),
        ..Default::default()
    };
    let cli_args = std::sync::Arc::new(test_cli());
    let merged = merge_site_config(Some(&defaults), &site_config, &cli_args);
    assert_eq!(merged.language, "pt");
}

// === deep_merge_yaml edge case ===

#[test]
fn test_deep_merge_yaml_overlay_replaces_mapping() {
    let base: serde_yaml::Value = serde_yaml::from_str("top:\n  a: 1\n  b: 2").unwrap();
    let overlay: serde_yaml::Value = serde_yaml::from_str("top: replaced").unwrap();
    let merged = deep_merge_yaml(base, overlay);
    let top = merged
        .as_mapping()
        .unwrap()
        .get(serde_yaml::Value::String("top".into()))
        .unwrap();
    assert_eq!(top.as_str(), Some("replaced"));
}

#[test]
fn test_deep_merge_yaml_empty_base() {
    let base: serde_yaml::Value = serde_yaml::from_str("{}").unwrap();
    let overlay: serde_yaml::Value = serde_yaml::from_str("a: 1\nb: 2").unwrap();
    let merged = deep_merge_yaml(base, overlay);
    let map = merged.as_mapping().unwrap();
    assert_eq!(
        map.get(serde_yaml::Value::String("a".into())),
        Some(&serde_yaml::Value::Number(1.into()))
    );
    assert_eq!(
        map.get(serde_yaml::Value::String("b".into())),
        Some(&serde_yaml::Value::Number(2.into()))
    );
}
