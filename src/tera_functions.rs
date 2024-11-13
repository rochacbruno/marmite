use indexmap::IndexMap;
use std::collections::HashMap;
use tera::{to_value, Function, Result as TeraResult, Value};
use url::Url;

use crate::site::Data;

#[derive(Default)]
pub struct UrlFor {
    pub base_url: String,
}

impl Function for UrlFor {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        // Extract the "path" argument
        let mut path = args
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| tera::Error::msg("Missing `path` argument"))?
            .trim_start_matches("./")
            .to_string();

        let abs_prefixes = ["http", "https", "mailto"];
        if abs_prefixes.iter().any(|&prefix| path.starts_with(prefix)) {
            return to_value(path).map_err(tera::Error::from);
        }

        // Ensure the path starts with "/" by adding it if necessary
        if !path.starts_with('/') {
            path = format!("/{path}");
        }

        // Trim trailing slashes from base_url if it's not empty
        let base_url = if self.base_url.is_empty() {
            String::new()
        } else {
            self.base_url.trim_end_matches('/').to_string()
        };

        // Parse the base_url to extract the path part if not empty
        let base_path = if base_url.is_empty() {
            String::new()
        } else {
            Url::parse(&base_url)
                .map(|parsed_url| parsed_url.path().trim_end_matches('/').to_string())
                .unwrap_or_default()
        };

        // Check if the "abs" argument is provided and set to true
        let abs = args.get("abs").and_then(Value::as_bool).unwrap_or(false);

        // Construct the URL based on the presence of base_url and abs flag
        let url = if abs && !base_url.is_empty() {
            // Absolute URL with base_url
            format!("{}/{}", base_url, path.trim_start_matches('/'))
        } else if !base_path.is_empty() {
            // Relative URL with base path from base_url
            format!("{base_path}{path}")
        } else {
            // Just the path if no base_url or base_path
            path
        };

        // Return the URL as a Tera Value
        to_value(url).map_err(tera::Error::from)
    }
}

/// Tera template function that takes a `kind` argument and returns the grouped content
/// based on the kind. The function is used to group the content by tags or archive.
/// The grouped content is built using the `site_data` field from the `Group` struct.
/// and converted to an `IndexMap` to preserve the order of insertion that is
/// determined by the iter on `GroupedContent`.
pub struct Group {
    pub site_data: Data,
}

impl Function for Group {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let kind = args
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| tera::Error::msg("Missing `kind` argument"))?;

        let grouped_content = match kind {
            "tag" => &self.site_data.tag,
            "archive" => &self.site_data.archive,
            "author" => &self.site_data.author,
            _ => return Err(tera::Error::msg("Invalid `kind` argument")),
        };

        // create an IndexMap from the iterated content
        let mut ordered_map = IndexMap::new();
        for (k, v) in grouped_content.iter() {
            ordered_map.insert(k.clone(), v.clone());
        }

        let json_value = serde_json::to_value(&ordered_map)
            .map_err(|e| tera::Error::msg(format!("Failed to convert to JSON: {e}")))?;

        to_value(json_value).map_err(tera::Error::from)
    }
}
