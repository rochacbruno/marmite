use std::collections::HashMap;
use tera::{to_value, Function, Result as TeraResult, Value};
use url::Url;

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
            .to_string();

        // Ensure the path starts with "/" by adding it if necessary
        if !path.starts_with('/') {
            path = format!("/{}", path);
        }

        // Trim trailing slashes from base_url if it's not empty
        let base_url = if self.base_url.is_empty() {
            "".to_string()
        } else {
            self.base_url.trim_end_matches('/').to_string()
        };

        // Parse the base_url to extract the path part if not empty
        let base_path = if !base_url.is_empty() {
            Url::parse(&base_url)
                .map(|parsed_url| parsed_url.path().trim_end_matches('/').to_string())
                .unwrap_or_default()
        } else {
            "".to_string()
        };

        // Check if the "abs" argument is provided and set to true
        let abs = args.get("abs").and_then(Value::as_bool).unwrap_or(false);

        // Construct the URL based on the presence of base_url and abs flag
        let url = if abs && !base_url.is_empty() {
            // Absolute URL with base_url
            format!("{}/{}", base_url, path.trim_start_matches('/'))
        } else if !base_path.is_empty() {
            // Relative URL with base path from base_url
            format!("{}{}", base_path, path)
        } else {
            // Just the path if no base_url or base_path
            path
        };

        // Return the URL as a Tera Value
        to_value(url).map_err(tera::Error::from)
    }
}
