use super::*;
use serde_json::json;

#[test]
fn test_url_for_basic_path() {
    let url_for = UrlFor {
        base_url: String::new(),
        ..Default::default()
    };
    let result = url_for.resolve("about.html", false);
    assert_eq!(result, "/about.html");
}

#[test]
fn test_url_for_absolute_path() {
    let url_for = UrlFor {
        base_url: "https://example.com".to_string(),
        ..Default::default()
    };
    let result = url_for.resolve("about.html", true);
    assert_eq!(result, "https://example.com/about.html");
}

#[test]
fn test_url_for_external_url() {
    let url_for = UrlFor {
        base_url: String::new(),
        ..Default::default()
    };
    let result = url_for.resolve("https://external.com", false);
    assert_eq!(result, "https://external.com");
}

#[test]
fn test_url_for_missing_path() {
    let mut tera = tera::Tera::default();
    tera.register_function(
        "url_for",
        UrlFor {
            base_url: String::new(),
            ..Default::default()
        },
    );
    tera.add_raw_template("test", r#"{{ url_for() }}"#).unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_err());
}

fn create_test_data() -> Data {
    use std::fs;
    use tempfile::TempDir;
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.yaml");
    fs::write(&config_path, "title: Test Site\n").unwrap();
    Data::from_file(&config_path)
}

#[test]
fn test_group_function_tag() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function(
        "group",
        Group {
            site_data,
            cross_site_data: None,
        },
    );
    tera.add_raw_template("test", r#"{{ group(kind="tag") }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_group_function_invalid_kind() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function(
        "group",
        Group {
            site_data,
            cross_site_data: None,
        },
    );
    tera.add_raw_template("test", r#"{{ group(kind="invalid") }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_err());
}

#[test]
fn test_group_function_missing_kind() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function(
        "group",
        Group {
            site_data,
            cross_site_data: None,
        },
    );
    tera.add_raw_template("test", r#"{{ group() }}"#).unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_err());
}

#[test]
fn test_source_link_empty() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function("source_link", SourceLink { site_data });
    tera.add_raw_template("test", r#"{{ source_link(content=content) }}"#)
        .unwrap();
    let content = json!({
        "source_path": "/path/to/file.md"
    });
    let mut ctx = tera::Context::new();
    ctx.insert("content", &content);
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_display_name_stream_without_config() {
    let site_data = create_test_data();
    let display_name = DisplayName {
        site_data,
        kind: "stream".to_string(),
    };
    let result = display_name.resolve("main");
    assert_eq!(result, "main");
}

#[test]
fn test_get_posts_default() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function(
        "get_posts",
        GetPosts {
            site_data,
            cross_site_data: None,
        },
    );
    tera.add_raw_template("test", r#"{{ get_posts() }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_get_posts_with_limit() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function(
        "get_posts",
        GetPosts {
            site_data,
            cross_site_data: None,
        },
    );
    tera.add_raw_template("test", r#"{{ get_posts(items=2) }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_get_posts_asc_order() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function(
        "get_posts",
        GetPosts {
            site_data,
            cross_site_data: None,
        },
    );
    tera.add_raw_template("test", r#"{{ get_posts(ord="asc") }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_get_posts_with_string_limit() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function(
        "get_posts",
        GetPosts {
            site_data,
            cross_site_data: None,
        },
    );
    tera.add_raw_template("test", r#"{{ get_posts(items="5") }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_group_function_tags_with_params() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function(
        "group",
        Group {
            site_data,
            cross_site_data: None,
        },
    );
    tera.add_raw_template("test", r#"{{ group(kind="tag", ord="asc", items=2) }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_group_function_series_with_params() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function(
        "group",
        Group {
            site_data,
            cross_site_data: None,
        },
    );
    tera.add_raw_template("test", r#"{{ group(kind="series", ord="desc") }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_group_function_streams_with_params() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function(
        "group",
        Group {
            site_data,
            cross_site_data: None,
        },
    );
    tera.add_raw_template("test", r#"{{ group(kind="stream", items="5") }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_get_data_by_slug_missing_slug() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function(
        "get_data_by_slug",
        GetDataBySlug {
            site_data,
            cross_site_data: None,
        },
    );
    tera.add_raw_template("test", r#"{{ get_data_by_slug() }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_err());
}

#[test]
fn test_get_data_by_slug_nonexistent_slug() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function(
        "get_data_by_slug",
        GetDataBySlug {
            site_data,
            cross_site_data: None,
        },
    );
    tera.add_raw_template("test", r#"{{ get_data_by_slug(slug="nonexistent-slug") }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Content not found for slug"));
}

#[test]
fn test_get_data_by_slug_tag_not_found() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function(
        "get_data_by_slug",
        GetDataBySlug {
            site_data,
            cross_site_data: None,
        },
    );
    tera.add_raw_template("test", r#"{{ get_data_by_slug(slug="tag-nonexistent") }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Tag not found"));
}

// === prefix_slug tests ===

#[test]
fn test_prefix_slug_empty_path() {
    assert_eq!(prefix_slug("my-post", ""), "my-post");
}

#[test]
fn test_prefix_slug_with_path() {
    assert_eq!(prefix_slug("my-post", "blog"), "blog/my-post");
}

#[test]
fn test_prefix_slug_nested_path() {
    assert_eq!(
        prefix_slug("post.html", "sites/blog"),
        "sites/blog/post.html"
    );
}

// === UrlFor with path_prefix tests ===

#[test]
fn test_url_for_with_path_prefix() {
    let url_for = UrlFor {
        base_url: String::new(),
        path_prefix: "blog".to_string(),
        all_site_prefixes: vec!["blog".to_string()],
    };
    let result = url_for.resolve("about.html", false);
    assert_eq!(result, "/blog/about.html");
}

#[test]
fn test_url_for_with_path_prefix_already_prefixed() {
    let url_for = UrlFor {
        base_url: String::new(),
        path_prefix: "blog".to_string(),
        all_site_prefixes: vec!["blog".to_string()],
    };
    let result = url_for.resolve("/blog/about.html", false);
    assert_eq!(result, "/blog/about.html");
}

#[test]
fn test_url_for_with_path_prefix_external() {
    let url_for = UrlFor {
        base_url: String::new(),
        path_prefix: "blog".to_string(),
        all_site_prefixes: vec!["blog".to_string()],
    };
    let result = url_for.resolve("https://example.com", false);
    assert_eq!(result, "https://example.com");
}

#[test]
fn test_url_for_with_path_prefix_other_site() {
    let url_for = UrlFor {
        base_url: String::new(),
        path_prefix: "blog".to_string(),
        all_site_prefixes: vec!["blog".to_string(), "docs".to_string()],
    };
    let result = url_for.resolve("/docs/page.html", false);
    assert_eq!(result, "/docs/page.html");
}

#[test]
fn test_url_for_has_site_prefix_true() {
    let url_for = UrlFor {
        base_url: String::new(),
        path_prefix: String::new(),
        all_site_prefixes: vec!["blog".to_string(), "docs".to_string()],
    };
    assert!(url_for.has_site_prefix("/blog/post.html"));
    assert!(url_for.has_site_prefix("/docs/page.html"));
}

#[test]
fn test_url_for_has_site_prefix_false() {
    let url_for = UrlFor {
        base_url: String::new(),
        path_prefix: String::new(),
        all_site_prefixes: vec!["blog".to_string()],
    };
    assert!(!url_for.has_site_prefix("/about.html"));
    assert!(!url_for.has_site_prefix("/other/page.html"));
}

#[test]
fn test_url_for_has_site_prefix_empty_filtered() {
    let url_for = UrlFor {
        base_url: String::new(),
        path_prefix: String::new(),
        all_site_prefixes: vec![String::new(), "blog".to_string()],
    };
    assert!(!url_for.has_site_prefix("/any/path.html"));
    assert!(url_for.has_site_prefix("/blog/post.html"));
}

// === collect_cross_site_posts/pages tests ===

fn make_cross_site_data() -> crate::workspace::CrossSiteData {
    use crate::content::Content;
    use crate::workspace::{CrossSiteData, SiteData};

    let mut blog_data = Data::new("", &std::path::PathBuf::from("test.yaml"));
    blog_data.posts.push(Content {
        title: "Blog Post 1".to_string(),
        slug: "blog-post-1".to_string(),
        ..Default::default()
    });
    blog_data.posts.push(Content {
        title: "Blog Post 2".to_string(),
        slug: "blog-post-2".to_string(),
        ..Default::default()
    });
    blog_data.pages.push(Content {
        title: "Blog About".to_string(),
        slug: "about".to_string(),
        ..Default::default()
    });

    let mut docs_data = Data::new("", &std::path::PathBuf::from("test.yaml"));
    docs_data.posts.push(Content {
        title: "Docs Post".to_string(),
        slug: "docs-post".to_string(),
        ..Default::default()
    });
    docs_data.pages.push(Content {
        title: "Docs Intro".to_string(),
        slug: "intro".to_string(),
        ..Default::default()
    });

    let mut sites = std::collections::HashMap::new();
    sites.insert(
        "blog".to_string(),
        SiteData {
            name: "blog".to_string(),
            output_path: "blog".to_string(),
            data: blog_data,
        },
    );
    sites.insert(
        "docs".to_string(),
        SiteData {
            name: "docs".to_string(),
            output_path: "docs".to_string(),
            data: docs_data,
        },
    );

    CrossSiteData {
        sites,
        separator: "::".to_string(),
    }
}

#[test]
fn test_collect_cross_site_posts_all() {
    let csd = make_cross_site_data();
    let posts = collect_cross_site_posts("all", &csd);
    assert_eq!(posts.len(), 3);
    assert!(posts.iter().all(|p| p.slug.contains('/')));
}

#[test]
fn test_collect_cross_site_posts_specific() {
    let csd = make_cross_site_data();
    let posts = collect_cross_site_posts("blog", &csd);
    assert_eq!(posts.len(), 2);
    assert!(posts.iter().all(|p| p.slug.starts_with("blog/")));
}

#[test]
fn test_collect_cross_site_posts_unknown_site() {
    let csd = make_cross_site_data();
    let posts = collect_cross_site_posts("unknown", &csd);
    assert!(posts.is_empty());
}

#[test]
fn test_collect_cross_site_pages_all() {
    let csd = make_cross_site_data();
    let pages = collect_cross_site_pages("all", &csd);
    assert_eq!(pages.len(), 2);
}

#[test]
fn test_collect_cross_site_pages_specific() {
    let csd = make_cross_site_data();
    let pages = collect_cross_site_pages("docs", &csd);
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].slug, "docs/intro");
}

// === resolve_slug_in_data tests ===

#[test]
fn test_resolve_slug_in_data_page() {
    let mut data = Data::new("", &std::path::PathBuf::from("test.yaml"));
    data.pages.push(crate::content::Content {
        title: "About Page".to_string(),
        slug: "about".to_string(),
        description: Some("About us".to_string()),
        ..Default::default()
    });

    let result = resolve_slug_in_data("about", &data);
    assert!(result.is_some());
    let slug_data = result.unwrap();
    assert_eq!(slug_data.slug, "about");
    assert_eq!(slug_data.title, "About Page");
    assert_eq!(slug_data.content_type, "page");
    assert_eq!(slug_data.text, "About us");
}

#[test]
fn test_resolve_slug_in_data_post() {
    let mut data = Data::new("", &std::path::PathBuf::from("test.yaml"));
    data.posts.push(crate::content::Content {
        title: "My Post".to_string(),
        slug: "my-post".to_string(),
        date: Some(
            chrono::NaiveDate::from_ymd_opt(2024, 6, 15)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        ),
        ..Default::default()
    });

    let result = resolve_slug_in_data("my-post", &data);
    assert!(result.is_some());
    let slug_data = result.unwrap();
    assert_eq!(slug_data.slug, "my-post");
    assert_eq!(slug_data.title, "My Post");
    assert_eq!(slug_data.content_type, "post");
    assert_eq!(slug_data.text, "2024-06-15");
}

#[test]
fn test_resolve_slug_in_data_not_found() {
    let data = Data::new("", &std::path::PathBuf::from("test.yaml"));
    assert!(resolve_slug_in_data("nonexistent", &data).is_none());
}

// === GetPages tests ===

#[test]
fn test_get_pages_default() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function(
        "get_pages",
        GetPages {
            site_data,
            cross_site_data: None,
        },
    );
    tera.add_raw_template("test", r#"{{ get_pages() }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_get_pages_with_limit() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function(
        "get_pages",
        GetPages {
            site_data,
            cross_site_data: None,
        },
    );
    tera.add_raw_template("test", r#"{{ get_pages(items=2) }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

// === sort_group_list tests ===

#[test]
fn test_sort_group_list_archive_asc() {
    let mut groups = vec![
        ("2024".to_string(), vec![crate::content::Content::default()]),
        ("2023".to_string(), vec![crate::content::Content::default()]),
        ("2025".to_string(), vec![crate::content::Content::default()]),
    ];
    sort_group_list(&mut groups, "archive", "asc");
    assert_eq!(groups[0].0, "2025");
    assert_eq!(groups[2].0, "2024");
}

#[test]
fn test_sort_group_list_archive_desc_no_change() {
    let mut groups = vec![
        ("2024".to_string(), vec![crate::content::Content::default()]),
        ("2023".to_string(), vec![crate::content::Content::default()]),
        ("2025".to_string(), vec![crate::content::Content::default()]),
    ];
    sort_group_list(&mut groups, "archive", "desc");
    assert_eq!(groups[0].0, "2024");
    assert_eq!(groups[1].0, "2023");
    assert_eq!(groups[2].0, "2025");
}

#[test]
fn test_sort_group_list_tag_asc() {
    let mut groups = vec![
        ("rust".to_string(), vec![crate::content::Content::default()]),
        (
            "alpha".to_string(),
            vec![crate::content::Content::default()],
        ),
        ("beta".to_string(), vec![crate::content::Content::default()]),
    ];
    sort_group_list(&mut groups, "tag", "asc");
    assert_eq!(groups[0].0, "alpha");
    assert_eq!(groups[1].0, "beta");
    assert_eq!(groups[2].0, "rust");
}

#[test]
fn test_sort_group_list_tag_desc_sorts_by_count() {
    let mut groups = vec![
        ("rust".to_string(), vec![crate::content::Content::default()]),
        (
            "python".to_string(),
            vec![
                crate::content::Content::default(),
                crate::content::Content::default(),
                crate::content::Content::default(),
            ],
        ),
        (
            "go".to_string(),
            vec![
                crate::content::Content::default(),
                crate::content::Content::default(),
            ],
        ),
    ];
    sort_group_list(&mut groups, "tag", "desc");
    assert_eq!(groups[0].0, "python");
    assert_eq!(groups[1].0, "go");
    assert_eq!(groups[2].0, "rust");
}

// === DisplayName series tests ===

#[test]
fn test_display_name_series_without_config() {
    let site_data = create_test_data();
    let display_name = DisplayName {
        site_data,
        kind: "series".to_string(),
    };
    let result = display_name.resolve("my-series");
    assert_eq!(result, "my-series");
}

#[test]
fn test_display_name_unknown_kind() {
    let site_data = create_test_data();
    let display_name = DisplayName {
        site_data,
        kind: "unknown".to_string(),
    };
    let result = display_name.resolve("test");
    assert_eq!(result, "test");
}

// === SourceLink with source_repository tests ===

#[test]
fn test_source_link_with_repository() {
    use std::fs;
    use tempfile::TempDir;
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.yaml");
    fs::write(
        &config_path,
        "source_repository: https://github.com/user/repo/blob/main/content\n",
    )
    .unwrap();
    let site_data = Data::from_file(&config_path);

    let mut tera = tera::Tera::default();
    tera.register_function("source_link", SourceLink { site_data });
    tera.add_raw_template("test", r#"{{ source_link(content=content) }}"#)
        .unwrap();
    let content = json!({
        "source_path": "/path/to/my-post.md"
    });
    let mut ctx = tera::Context::new();
    ctx.insert("content", &content);
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(
        result,
        "https://github.com/user/repo/blob/main/content/my-post.md"
    );
}

#[test]
fn test_source_link_with_publish_md() {
    use std::fs;
    use tempfile::TempDir;
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.yaml");
    fs::write(&config_path, "publish_md: true\n").unwrap();
    let site_data = Data::from_file(&config_path);

    let mut tera = tera::Tera::default();
    tera.register_function("source_link", SourceLink { site_data });
    tera.add_raw_template("test", r#"{{ source_link(content=content) }}"#)
        .unwrap();
    let content = json!({
        "source_path": "/path/to/my-post.md"
    });
    let mut ctx = tera::Context::new();
    ctx.insert("content", &content);
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result, "./my-post.md");
}

// === merge_grouped_content tests ===

#[test]
fn test_merge_grouped_content_all() {
    let csd = make_cross_site_data();
    let merged = merge_grouped_content("all", "tag", &csd);
    assert!(merged.is_empty() || merged.values().all(|v| !v.is_empty()));
}

#[test]
fn test_merge_grouped_content_unknown_site() {
    let csd = make_cross_site_data();
    let merged = merge_grouped_content("unknown", "tag", &csd);
    assert!(merged.is_empty());
}
