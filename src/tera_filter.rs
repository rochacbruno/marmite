use std::str::FromStr;

use tera::{to_value, Filter, Value};

pub struct DefaultDateFormat {
    pub date_format: String,
}

impl Filter for DefaultDateFormat {
    fn filter(
        &self,
        value: &tera::Value,
        _: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let date_str = value
            .as_str()
            .ok_or(tera::Error::msg("Missing date string"))?;
        let date = chrono::NaiveDateTime::from_str(date_str)
            .map_err(|e| tera::Error::msg(e.to_string()))?;
        let formatted_date = date.format(self.date_format.as_str()).to_string();

        to_value(formatted_date).map_err(tera::Error::from)
    }
}

pub struct RemoveDraft;

impl Filter for RemoveDraft {
    fn filter(
        &self,
        value: &Value,
        _: &std::collections::HashMap<String, Value>,
    ) -> tera::Result<Value> {
        let items = value
            .as_array()
            .ok_or_else(|| tera::Error::msg("Expected an array"))?;

        let filtered: Vec<Value> = items
            .iter()
            .filter(|item| {
                // Check if the item has a stream field that equals "draft"
                if let Some(stream) = item.get("stream") {
                    if let Some(stream_str) = stream.as_str() {
                        return stream_str != "draft";
                    }
                }
                // If no stream field or stream is not a string, include the item
                true
            })
            .cloned()
            .collect();

        to_value(filtered).map_err(tera::Error::from)
    }
}

#[cfg(test)]
mod tests {
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
}
