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
