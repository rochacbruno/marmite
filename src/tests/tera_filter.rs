use super::*;
use serde_json::json;

#[test]
fn test_default_date_format_valid_date() {
    let filter = DefaultDateFormat {
        date_format: "%Y-%m-%d".to_string(),
    };
    let mut tera = tera::Tera::default();
    tera.register_filter("default_date_format", filter);
    tera.add_raw_template("test", r#"{{ val | default_date_format }}"#)
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("val", "2023-12-25T10:30:00");
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result, "2023-12-25");
}

#[test]
fn test_default_date_format_invalid_date_string() {
    let filter = DefaultDateFormat {
        date_format: "%Y-%m-%d".to_string(),
    };
    let mut tera = tera::Tera::default();
    tera.register_filter("default_date_format", filter);
    tera.add_raw_template("test", r#"{{ val | default_date_format }}"#)
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("val", "invalid-date");
    let result = tera.render("test", &ctx);
    assert!(result.is_err());
}

#[test]
fn test_default_date_format_non_string_value() {
    let filter = DefaultDateFormat {
        date_format: "%Y-%m-%d".to_string(),
    };
    let mut tera = tera::Tera::default();
    tera.register_filter("default_date_format", filter);
    tera.add_raw_template("test", r#"{{ val | default_date_format }}"#)
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("val", &123);
    let result = tera.render("test", &ctx);
    assert!(result.is_err());
}

#[test]
fn test_remove_draft_filter_items() {
    let filter = RemoveDraft;
    let mut tera = tera::Tera::default();
    tera.register_filter("remove_draft", filter);
    tera.add_raw_template("test", r#"{{ items | remove_draft | length }}"#)
        .unwrap();

    let items = json!([
        {"title": "Published Post", "stream": "main"},
        {"title": "Draft Post", "stream": "draft"},
        {"title": "Another Post", "stream": "blog"}
    ]);
    let mut ctx = tera::Context::new();
    ctx.insert("items", &items);

    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result.trim(), "2");
}

#[test]
fn test_remove_draft_filter_no_stream_field() {
    let filter = RemoveDraft;
    let mut tera = tera::Tera::default();
    tera.register_filter("remove_draft", filter);
    tera.add_raw_template("test", r#"{{ items | remove_draft | length }}"#)
        .unwrap();

    let items = json!([
        {"title": "Post Without Stream"},
        {"title": "Another Post", "stream": "main"}
    ]);
    let mut ctx = tera::Context::new();
    ctx.insert("items", &items);

    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result.trim(), "2");
}

#[test]
fn test_remove_draft_filter_non_array_value() {
    let filter = RemoveDraft;
    let mut tera = tera::Tera::default();
    tera.register_filter("remove_draft", filter);
    tera.add_raw_template("test", r#"{{ items | remove_draft }}"#)
        .unwrap();

    let mut ctx = tera::Context::new();
    ctx.insert("items", "not an array");

    let result = tera.render("test", &ctx);
    assert!(result.is_err());
}

#[test]
fn test_remove_draft_filter_empty_array() {
    let filter = RemoveDraft;
    let mut tera = tera::Tera::default();
    tera.register_filter("remove_draft", filter);
    tera.add_raw_template("test", r#"{{ items | remove_draft | length }}"#)
        .unwrap();

    let items: Vec<serde_json::Value> = vec![];
    let mut ctx = tera::Context::new();
    ctx.insert("items", &items);

    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result.trim(), "0");
}

// --- Tera 2.0 compatibility filter shim tests ---

#[test]
fn test_date_filter_datetime_format() {
    let mut tera = tera::Tera::default();
    tera.register_filter("date", date);
    tera.add_raw_template("test", r#"{{ val | date(format="%Y-%m-%d") }}"#)
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("val", "2024-06-15T14:30:00");
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result, "2024-06-15");
}

#[test]
fn test_date_filter_date_only() {
    let mut tera = tera::Tera::default();
    tera.register_filter("date", date);
    tera.add_raw_template("test", r#"{{ val | date(format="%Y-%m-%d") }}"#)
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("val", "2024-06-15");
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result, "2024-06-15");
}

#[test]
fn test_date_filter_rfc3339() {
    let mut tera = tera::Tera::default();
    tera.register_filter("date", date);
    tera.add_raw_template("test", r#"{{ val | date(format="%+") }}"#)
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("val", "2024-06-15T14:30:00");
    let result = tera.render("test", &ctx).unwrap();
    assert!(result.contains("2024-06-15"));
}

#[test]
fn test_date_filter_default_format() {
    let mut tera = tera::Tera::default();
    tera.register_filter("date", date);
    tera.add_raw_template("test", r#"{{ val | date }}"#)
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("val", "2024-06-15T14:30:00");
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result, "2024-06-15");
}

#[test]
fn test_date_filter_invalid_date() {
    let mut tera = tera::Tera::default();
    tera.register_filter("date", date);
    tera.add_raw_template("test", r#"{{ val | date }}"#)
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("val", "not-a-date");
    let result = tera.render("test", &ctx);
    assert!(result.is_err());
}

#[test]
fn test_striptags_filter() {
    let mut tera = tera::Tera::default();
    tera.register_filter("striptags", striptags);
    tera.add_raw_template("test", r#"{{ val | striptags }}"#)
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("val", "<p>Hello <strong>world</strong></p>");
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result, "Hello world");
}

#[test]
fn test_striptags_filter_no_tags() {
    let mut tera = tera::Tera::default();
    tera.register_filter("striptags", striptags);
    tera.add_raw_template("test", r#"{{ val | striptags }}"#)
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("val", "plain text");
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result, "plain text");
}

#[test]
fn test_striptags_filter_nested() {
    let mut tera = tera::Tera::default();
    tera.register_filter("striptags", striptags);
    tera.add_raw_template("test", r#"{{ val | striptags }}"#)
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert(
        "val",
        r#"<div class="wrapper"><h1>Title</h1><p>Body</p></div>"#,
    );
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result, "TitleBody");
}

#[test]
fn test_trim_start_matches_filter_with_pattern() {
    let mut tera = tera::Tera::default();
    tera.register_filter("trim_start_matches", trim_start_matches);
    tera.add_raw_template("test", r#"{{ val | trim_start_matches(pat="Hello ") }}"#)
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("val", "Hello World");
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result, "World");
}

#[test]
fn test_trim_start_matches_filter_no_match() {
    let mut tera = tera::Tera::default();
    tera.register_filter("trim_start_matches", trim_start_matches);
    tera.add_raw_template("test", r#"{{ val | trim_start_matches(pat="xyz") }}"#)
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("val", "Hello World");
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result, "Hello World");
}

#[test]
fn test_trim_start_matches_filter_no_pattern() {
    let mut tera = tera::Tera::default();
    tera.register_filter("trim_start_matches", trim_start_matches);
    tera.add_raw_template("test", r#"{{ val | trim_start_matches }}"#)
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("val", "   spaces");
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result, "spaces");
}

#[test]
fn test_slice_filter_with_end() {
    let mut tera = tera::Tera::default();
    tera.register_filter("slice", slice);
    tera.add_raw_template("test", r#"{{ items | slice(end=2) | length }}"#)
        .unwrap();
    let items = vec!["a", "b", "c", "d"];
    let mut ctx = tera::Context::new();
    ctx.insert("items", &items);
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result.trim(), "2");
}

#[test]
fn test_slice_filter_with_start_and_end() {
    let mut tera = tera::Tera::default();
    tera.register_filter("slice", slice);
    tera.add_raw_template("test", r#"{{ items | slice(start=1, end=3) | length }}"#)
        .unwrap();
    let items = vec!["a", "b", "c", "d", "e"];
    let mut ctx = tera::Context::new();
    ctx.insert("items", &items);
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result.trim(), "2");
}

#[test]
fn test_slice_filter_end_beyond_length() {
    let mut tera = tera::Tera::default();
    tera.register_filter("slice", slice);
    tera.add_raw_template("test", r#"{{ items | slice(end=100) | length }}"#)
        .unwrap();
    let items = vec!["a", "b"];
    let mut ctx = tera::Context::new();
    ctx.insert("items", &items);
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result.trim(), "2");
}

#[test]
fn test_slice_filter_empty_array() {
    let mut tera = tera::Tera::default();
    tera.register_filter("slice", slice);
    tera.add_raw_template("test", r#"{{ items | slice(end=3) | length }}"#)
        .unwrap();
    let items: Vec<&str> = vec![];
    let mut ctx = tera::Context::new();
    ctx.insert("items", &items);
    let result = tera.render("test", &ctx).unwrap();
    assert_eq!(result.trim(), "0");
}

#[test]
fn test_slice_filter_non_array_error() {
    let mut tera = tera::Tera::default();
    tera.register_filter("slice", slice);
    tera.add_raw_template("test", r#"{{ val | slice(end=2) }}"#)
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("val", "not an array");
    let result = tera.render("test", &ctx);
    assert!(result.is_err());
}
