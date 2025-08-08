use super::*;
use serde_json::json;
use std::collections::HashMap;
use tera::Value;

#[test]
fn test_default_date_format_valid_date() {
    let filter = DefaultDateFormat {
        date_format: "%Y-%m-%d".to_string(),
    };
    let value = Value::String("2023-12-25T10:30:00".to_string());
    let args = HashMap::new();

    let result = filter.filter(&value, &args).unwrap();
    assert_eq!(result, Value::String("2023-12-25".to_string()));
}

#[test]
fn test_default_date_format_invalid_date_string() {
    let filter = DefaultDateFormat {
        date_format: "%Y-%m-%d".to_string(),
    };
    let value = Value::String("invalid-date".to_string());
    let args = HashMap::new();

    let result = filter.filter(&value, &args);
    assert!(result.is_err());
}

#[test]
fn test_default_date_format_non_string_value() {
    let filter = DefaultDateFormat {
        date_format: "%Y-%m-%d".to_string(),
    };
    let value = Value::Number(123.into());
    let args = HashMap::new();

    let result = filter.filter(&value, &args);
    assert!(result.is_err());
}

#[test]
fn test_remove_draft_filter_items() {
    let filter = RemoveDraft;
    let items = json!([
        {"title": "Published Post", "stream": "main"},
        {"title": "Draft Post", "stream": "draft"},
        {"title": "Another Post", "stream": "blog"}
    ]);
    let args = HashMap::new();

    let result = filter.filter(&items, &args).unwrap();
    let filtered_array = result.as_array().unwrap();

    assert_eq!(filtered_array.len(), 2);
    assert_eq!(filtered_array[0]["title"], "Published Post");
    assert_eq!(filtered_array[1]["title"], "Another Post");
}

#[test]
fn test_remove_draft_filter_no_stream_field() {
    let filter = RemoveDraft;
    let items = json!([
        {"title": "Post Without Stream"},
        {"title": "Another Post", "stream": "main"}
    ]);
    let args = HashMap::new();

    let result = filter.filter(&items, &args).unwrap();
    let filtered_array = result.as_array().unwrap();

    assert_eq!(filtered_array.len(), 2);
}

#[test]
fn test_remove_draft_filter_non_array_value() {
    let filter = RemoveDraft;
    let value = Value::String("not an array".to_string());
    let args = HashMap::new();

    let result = filter.filter(&value, &args);
    assert!(result.is_err());
}

#[test]
fn test_remove_draft_filter_empty_array() {
    let filter = RemoveDraft;
    let items = json!([]);
    let args = HashMap::new();

    let result = filter.filter(&items, &args).unwrap();
    let filtered_array = result.as_array().unwrap();

    assert_eq!(filtered_array.len(), 0);
}
