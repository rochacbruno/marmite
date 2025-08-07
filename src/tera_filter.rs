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
#[path = "tests/tera_filter.rs"]
mod tests;
