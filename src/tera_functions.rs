use indexmap::IndexMap;
use std::collections::HashMap;
use tera::{to_value, Function, Result as TeraResult, Value};
use url::Url;

use crate::content::Content;
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
            "stream" => &self.site_data.stream,
            "series" => &self.site_data.series,
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

/// Tera template function that generates source links for content
/// It takes a `content` argument and returns a URL pointing to the source markdown file
/// If `source_repository` is configured, it generates a link to the repository
/// If `publish_md` is true, it generates a link to the local markdown file
pub struct SourceLink {
    pub site_data: Data,
}

impl Function for SourceLink {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let content = args
            .get("content")
            .ok_or_else(|| tera::Error::msg("Missing `content` argument"))?;

        // Extract the source_path from the content
        let source_path = content
            .get("source_path")
            .and_then(Value::as_str)
            .ok_or_else(|| tera::Error::msg("Missing `source_path` in content"))?;

        // If source_repository is configured, generate repository link
        if let Some(source_repository) = &self.site_data.site.source_repository {
            let source_path_buf = std::path::Path::new(source_path);
            let file_name = source_path_buf
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.md");

            let repo_url = format!("{}/{}", source_repository.trim_end_matches('/'), file_name);
            return to_value(repo_url).map_err(tera::Error::from);
        }

        // If publish_md is true and no source_repository, generate local link
        if self.site_data.site.publish_md {
            let source_path_buf = std::path::Path::new(source_path);
            let file_name = source_path_buf
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.md");

            let local_url = format!("./{file_name}");
            return to_value(local_url).map_err(tera::Error::from);
        }

        // Return empty string if neither option is enabled
        to_value("").map_err(tera::Error::from)
    }
}

/// Tera template function that returns the display name for a stream or series
/// It takes a `stream` or `series` argument and returns the configured display name
/// If no display name is configured, returns the stream/series name itself
pub struct DisplayName {
    pub site_data: Data,
    pub kind: String,
}

impl Function for DisplayName {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let name = args
            .get(&self.kind)
            .and_then(Value::as_str)
            .ok_or_else(|| tera::Error::msg(format!("Missing `{}` argument", self.kind)))?;

        // Check if there's a configured display name based on the kind
        let display_name = match self.kind.as_str() {
            "stream" => self
                .site_data
                .site
                .streams
                .get(name)
                .map(|config| &config.display_name),
            "series" => self
                .site_data
                .site
                .series
                .get(name)
                .map(|config| &config.display_name),
            _ => None,
        };

        if let Some(display_name) = display_name {
            to_value(display_name).map_err(tera::Error::from)
        } else {
            // Return the name itself if no display name is configured
            to_value(name).map_err(tera::Error::from)
        }
    }
}

/// Tera function to get filtered and sorted posts
/// Args: ord (optional, default="desc"), items (optional, default=0 for all)
pub struct GetPosts {
    pub site_data: Data,
}

impl Function for GetPosts {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let ord = args.get("ord").and_then(Value::as_str).unwrap_or("desc");

        let items = args.get("items").and_then(Value::as_u64).unwrap_or(0) as usize;

        let mut posts = self.site_data.posts.clone();

        // Sort posts
        if ord == "asc" {
            posts.reverse();
        }

        // Limit items if specified
        if items > 0 && items < posts.len() {
            posts.truncate(items);
        }

        to_value(posts).map_err(tera::Error::from)
    }
}

/// Tera function to get filtered and sorted tags
/// Args: ord (optional, default="desc"), items (optional, default=0 for all)
pub struct GetTags {
    pub site_data: Data,
}

impl Function for GetTags {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let ord = args.get("ord").and_then(Value::as_str).unwrap_or("desc");

        let items = args.get("items").and_then(Value::as_u64).unwrap_or(0) as usize;

        // Convert tag map to vector of (name, posts) tuples
        let mut tag_list: Vec<(String, Vec<Content>)> = self
            .site_data
            .tag
            .map
            .iter()
            .map(|(name, posts)| (name.clone(), posts.clone()))
            .collect();

        // Sort by post count (desc) or alphabetically by name (asc)
        if ord == "asc" {
            tag_list.sort_by(|a, b| a.0.cmp(&b.0));
        } else {
            tag_list.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        }

        // Limit items if specified
        if items > 0 && items < tag_list.len() {
            tag_list.truncate(items);
        }

        // Convert back to IndexMap to preserve order
        let mut ordered_map = IndexMap::new();
        for (name, posts) in tag_list {
            ordered_map.insert(name, posts);
        }

        to_value(ordered_map).map_err(tera::Error::from)
    }
}

/// Tera function to get filtered and sorted series
/// Args: ord (optional, default="desc"), items (optional, default=0 for all)
pub struct GetSeries {
    pub site_data: Data,
}

impl Function for GetSeries {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let ord = args.get("ord").and_then(Value::as_str).unwrap_or("desc");

        let items = args.get("items").and_then(Value::as_u64).unwrap_or(0) as usize;

        // Convert series map to vector of (name, posts) tuples
        let mut series_list: Vec<(String, Vec<Content>)> = self
            .site_data
            .series
            .map
            .iter()
            .map(|(name, posts)| (name.clone(), posts.clone()))
            .collect();

        // Sort by post count (desc) or alphabetically by name (asc)
        if ord == "asc" {
            series_list.sort_by(|a, b| a.0.cmp(&b.0));
        } else {
            series_list.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        }

        // Limit items if specified
        if items > 0 && items < series_list.len() {
            series_list.truncate(items);
        }

        // Convert back to IndexMap to preserve order
        let mut ordered_map = IndexMap::new();
        for (name, posts) in series_list {
            ordered_map.insert(name, posts);
        }

        to_value(ordered_map).map_err(tera::Error::from)
    }
}

/// Tera function to get filtered and sorted streams
/// Args: ord (optional, default="desc"), items (optional, default=0 for all)
pub struct GetStreams {
    pub site_data: Data,
}

impl Function for GetStreams {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let ord = args.get("ord").and_then(Value::as_str).unwrap_or("desc");

        let items = args.get("items").and_then(Value::as_u64).unwrap_or(0) as usize;

        // Convert stream map to vector of (name, posts) tuples
        let mut stream_list: Vec<(String, Vec<Content>)> = self
            .site_data
            .stream
            .map
            .iter()
            .map(|(name, posts)| (name.clone(), posts.clone()))
            .collect();

        // Sort by post count (desc) or alphabetically by name (asc)
        if ord == "asc" {
            stream_list.sort_by(|a, b| a.0.cmp(&b.0));
        } else {
            stream_list.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        }

        // Limit items if specified
        if items > 0 && items < stream_list.len() {
            stream_list.truncate(items);
        }

        // Convert back to IndexMap to preserve order
        let mut ordered_map = IndexMap::new();
        for (name, posts) in stream_list {
            ordered_map.insert(name, posts);
        }

        to_value(ordered_map).map_err(tera::Error::from)
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(result, Value::String("".to_string()));
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
    fn test_get_tags_default() {
        let site_data = create_test_data();
        let get_tags = GetTags { site_data };
        let args = HashMap::new();

        let result = get_tags.call(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_tags_with_limit() {
        let site_data = create_test_data();
        let get_tags = GetTags { site_data };
        let mut args = HashMap::new();
        args.insert("items".to_string(), Value::Number(2.into()));

        let result = get_tags.call(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_series_default() {
        let site_data = create_test_data();
        let get_series = GetSeries { site_data };
        let args = HashMap::new();

        let result = get_series.call(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_streams_default() {
        let site_data = create_test_data();
        let get_streams = GetStreams { site_data };
        let args = HashMap::new();

        let result = get_streams.call(&args);
        assert!(result.is_ok());
    }
}
