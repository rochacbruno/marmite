use super::*;
use serde_json::json;
use std::collections::HashMap;
use tera::Value;

#[test]
fn test_url_for_basic_path() {
    let url_for = UrlFor {
        base_url: String::new(),
    };
    let mut args = HashMap::new();
    args.insert("path".to_string(), Value::String("about.html".to_string()));

    let result = url_for.call(&args).unwrap();
    assert_eq!(result, Value::String("/about.html".to_string()));
}

#[test]
fn test_url_for_absolute_path() {
    let url_for = UrlFor {
        base_url: "https://example.com".to_string(),
    };
    let mut args = HashMap::new();
    args.insert("path".to_string(), Value::String("about.html".to_string()));
    args.insert("abs".to_string(), Value::Bool(true));

    let result = url_for.call(&args).unwrap();
    assert_eq!(
        result,
        Value::String("https://example.com/about.html".to_string())
    );
}

#[test]
fn test_url_for_external_url() {
    let url_for = UrlFor {
        base_url: String::new(),
    };
    let mut args = HashMap::new();
    args.insert(
        "path".to_string(),
        Value::String("https://external.com".to_string()),
    );

    let result = url_for.call(&args).unwrap();
    assert_eq!(result, Value::String("https://external.com".to_string()));
}

#[test]
fn test_url_for_missing_path() {
    let url_for = UrlFor {
        base_url: String::new(),
    };
    let args = HashMap::new();

    let result = url_for.call(&args);
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
    let group = Group { site_data };
    let mut args = HashMap::new();
    args.insert("kind".to_string(), Value::String("tag".to_string()));

    let result = group.call(&args);
    assert!(result.is_ok());
}

#[test]
fn test_group_function_invalid_kind() {
    let site_data = create_test_data();
    let group = Group { site_data };
    let mut args = HashMap::new();
    args.insert("kind".to_string(), Value::String("invalid".to_string()));

    let result = group.call(&args);
    assert!(result.is_err());
}

#[test]
fn test_group_function_missing_kind() {
    let site_data = create_test_data();
    let group = Group { site_data };
    let args = HashMap::new();

    let result = group.call(&args);
    assert!(result.is_err());
}

#[test]
fn test_source_link_empty() {
    let site_data = create_test_data();
    let source_link = SourceLink { site_data };
    let mut args = HashMap::new();
    let content = json!({
        "source_path": "/path/to/file.md"
    });
    args.insert("content".to_string(), content);

    let result = source_link.call(&args).unwrap();
    assert_eq!(result, Value::String(String::new()));
}

#[test]
fn test_display_name_stream_without_config() {
    let site_data = create_test_data();
    let display_name = DisplayName {
        site_data,
        kind: "stream".to_string(),
    };
    let mut args = HashMap::new();
    args.insert("stream".to_string(), Value::String("main".to_string()));

    let result = display_name.call(&args).unwrap();
    assert_eq!(result, Value::String("main".to_string()));
}

#[test]
fn test_get_posts_default() {
    let site_data = create_test_data();
    let get_posts = GetPosts { site_data };
    let args = HashMap::new();

    let result = get_posts.call(&args);
    assert!(result.is_ok());

    // Should return all posts in default desc order
    let posts = result.unwrap();
    assert!(posts.is_array());
}

#[test]
fn test_get_posts_with_limit() {
    let site_data = create_test_data();
    let get_posts = GetPosts { site_data };
    let mut args = HashMap::new();
    args.insert("items".to_string(), Value::Number(2.into()));

    let result = get_posts.call(&args);
    assert!(result.is_ok());
}

#[test]
fn test_get_posts_asc_order() {
    let site_data = create_test_data();
    let get_posts = GetPosts { site_data };
    let mut args = HashMap::new();
    args.insert("ord".to_string(), Value::String("asc".to_string()));

    let result = get_posts.call(&args);
    assert!(result.is_ok());
}

#[test]
fn test_get_posts_with_string_limit() {
    let site_data = create_test_data();
    let get_posts = GetPosts { site_data };
    let mut args = HashMap::new();
    args.insert("items".to_string(), Value::String("5".to_string()));

    let result = get_posts.call(&args);
    assert!(result.is_ok());
}

#[test]
fn test_group_function_tags_with_params() {
    let site_data = create_test_data();
    let group = Group { site_data };
    let mut args = HashMap::new();
    args.insert("kind".to_string(), Value::String("tag".to_string()));
    args.insert("ord".to_string(), Value::String("asc".to_string()));
    args.insert("items".to_string(), Value::Number(2.into()));

    let result = group.call(&args);
    assert!(result.is_ok());
}

#[test]
fn test_group_function_series_with_params() {
    let site_data = create_test_data();
    let group = Group { site_data };
    let mut args = HashMap::new();
    args.insert("kind".to_string(), Value::String("series".to_string()));
    args.insert("ord".to_string(), Value::String("desc".to_string()));

    let result = group.call(&args);
    assert!(result.is_ok());
}

#[test]
fn test_group_function_streams_with_params() {
    let site_data = create_test_data();
    let group = Group { site_data };
    let mut args = HashMap::new();
    args.insert("kind".to_string(), Value::String("stream".to_string()));
    args.insert("items".to_string(), Value::String("5".to_string()));

    let result = group.call(&args);
    assert!(result.is_ok());
}

#[test]
fn test_get_data_by_slug_missing_slug() {
    let site_data = create_test_data();
    let get_data_by_slug = GetDataBySlug { site_data };
    let args = HashMap::new();

    let result = get_data_by_slug.call(&args);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Missing `slug` argument"));
}

#[test]
fn test_get_data_by_slug_nonexistent_slug() {
    let site_data = create_test_data();
    let get_data_by_slug = GetDataBySlug { site_data };
    let mut args = HashMap::new();
    args.insert(
        "slug".to_string(),
        Value::String("nonexistent-slug".to_string()),
    );

    let result = get_data_by_slug.call(&args);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Content not found for slug"));
}

#[test]
fn test_get_data_by_slug_tag_not_found() {
    let site_data = create_test_data();
    let get_data_by_slug = GetDataBySlug { site_data };
    let mut args = HashMap::new();
    args.insert(
        "slug".to_string(),
        Value::String("tag-nonexistent".to_string()),
    );

    let result = get_data_by_slug.call(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Tag not found"));
}
