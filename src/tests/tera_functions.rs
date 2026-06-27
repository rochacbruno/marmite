use super::*;
use serde_json::json;

#[test]
fn test_url_for_basic_path() {
    let url_for = UrlFor {
        base_url: String::new(),
    };
    let result = url_for.resolve("about.html", false);
    assert_eq!(result, "/about.html");
}

#[test]
fn test_url_for_absolute_path() {
    let url_for = UrlFor {
        base_url: "https://example.com".to_string(),
    };
    let result = url_for.resolve("about.html", true);
    assert_eq!(result, "https://example.com/about.html");
}

#[test]
fn test_url_for_external_url() {
    let url_for = UrlFor {
        base_url: String::new(),
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
    tera.register_function("group", Group { site_data });
    tera.add_raw_template("test", r#"{{ group(kind="tag") }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_group_function_invalid_kind() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function("group", Group { site_data });
    tera.add_raw_template("test", r#"{{ group(kind="invalid") }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_err());
}

#[test]
fn test_group_function_missing_kind() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function("group", Group { site_data });
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
    tera.register_function("get_posts", GetPosts { site_data });
    tera.add_raw_template("test", r#"{{ get_posts() }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_get_posts_with_limit() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function("get_posts", GetPosts { site_data });
    tera.add_raw_template("test", r#"{{ get_posts(items=2) }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_get_posts_asc_order() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function("get_posts", GetPosts { site_data });
    tera.add_raw_template("test", r#"{{ get_posts(ord="asc") }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_get_posts_with_string_limit() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function("get_posts", GetPosts { site_data });
    tera.add_raw_template("test", r#"{{ get_posts(items="5") }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_group_function_tags_with_params() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function("group", Group { site_data });
    tera.add_raw_template("test", r#"{{ group(kind="tag", ord="asc", items=2) }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_group_function_series_with_params() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function("group", Group { site_data });
    tera.add_raw_template("test", r#"{{ group(kind="series", ord="desc") }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_group_function_streams_with_params() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function("group", Group { site_data });
    tera.add_raw_template("test", r#"{{ group(kind="stream", items="5") }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_ok());
}

#[test]
fn test_get_data_by_slug_missing_slug() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function("get_data_by_slug", GetDataBySlug { site_data });
    tera.add_raw_template("test", r#"{{ get_data_by_slug() }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_err());
}

#[test]
fn test_get_data_by_slug_nonexistent_slug() {
    let site_data = create_test_data();
    let mut tera = tera::Tera::default();
    tera.register_function("get_data_by_slug", GetDataBySlug { site_data });
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
    tera.register_function("get_data_by_slug", GetDataBySlug { site_data });
    tera.add_raw_template("test", r#"{{ get_data_by_slug(slug="tag-nonexistent") }}"#)
        .unwrap();
    let result = tera.render("test", &tera::Context::new());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Tag not found"));
}
