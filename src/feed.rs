use log::info;
use rss::{ChannelBuilder, ItemBuilder};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::config::Marmite;
use crate::content::Content;

pub fn generate_rss(
    contents: &[Content],
    output_path: &Path,
    filename: &str,
    config: &Marmite,
) -> Result<(), String> {
    let date_format = "%a, %d %b %Y %H:%M:%S GMT"; // Loose RFC-822 format

    let feed_url = if !config.url.starts_with("http://") && !config.url.starts_with("https://") {
        let protocol = if config.https.unwrap_or(false) {
            "https://"
        } else {
            "http://"
        };
        format!("{}{}", protocol, &config.url)
    } else {
        config.url.to_string()
    };

    let mut channel = ChannelBuilder::default()
        .title(&config.name)
        .link(&feed_url)
        .description(&config.tagline)
        .generator("marmite".to_string())
        .build();

    // Filter out content with stream "draft"
    let filtered_contents: Vec<&Content> = contents
        .iter()
        .filter(|content| {
            content
                .stream
                .as_ref()
                .is_none_or(|stream| stream != "draft")
        })
        .collect();

    for content in filtered_contents.iter().take(15) {
        let mut item = ItemBuilder::default()
            .title(content.title.clone())
            .link(format!("{}/{}.html", &feed_url, &content.slug))
            .description(content.description.clone())
            .guid(
                rss::GuidBuilder::default()
                    .value(format!("{}/{}.html", &feed_url, &content.slug))
                    .build(),
            )
            .pub_date(content.date.unwrap().format(date_format).to_string())
            .content(content.html.clone())
            .source(
                rss::SourceBuilder::default()
                    .url(&feed_url)
                    .title(filename.to_string())
                    .build(),
            )
            .build();

        if let Some(author) = content.authors.first() {
            item.author = Some(author.clone());
        }
        item.categories = content
            .tags
            .iter()
            .map(|tag| rss::CategoryBuilder::default().name(tag.clone()).build())
            .collect();
        channel.items.push(item);
    }

    if let Some(latest_item) = channel.items.first() {
        channel.pub_date = latest_item.pub_date.clone();
    }

    channel.last_build_date = Some(chrono::Utc::now().format(date_format).to_string());

    if !config.card_image.is_empty() {
        channel.image = Some(
            rss::ImageBuilder::default()
                .url(format!("{}/{}", &feed_url, &config.card_image))
                .build(),
        );
    }

    let rss = channel.to_string();
    let feed_path = output_path.join(format!("{filename}.rss"));
    let mut file = File::create(&feed_path).map_err(|e| e.to_string())?;
    file.write_all(rss.as_bytes()).map_err(|e| e.to_string())?;
    info!("Generated {}", &feed_path.display());

    Ok(())
}

/// Struct to represent a JSON feed for a Content
/// <https://jsonfeed.org/version/1>
#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonFeed {
    version: String,
    title: String,
    home_page_url: String,
    feed_url: String,
    description: String,
    items: Vec<JsonFeedItem>,
}

/// Struct to represent a JSON feed item for a Content
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonFeedItem {
    id: String,
    url: String,
    title: String,
    content_html: String,
    // content_text: String,
    summary: String,
    date_published: String,
    image: String,
    authors: Vec<JsonFeedAuthor>,
    tags: Vec<String>,
    language: String,
}

/// Struct to represent a JSON feed author
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonFeedAuthor {
    name: String,
    url: String,
    avatar: String,
}

/// Generate a JSON feed for the given contents
/// <https://jsonfeed.org/version/1>
/// saves the feed to the output path with the given filename and extension .json
pub fn generate_json(
    contents: &[Content],
    output_path: &Path,
    filename: &str,
    config: &Marmite,
) -> Result<(), String> {
    let date_format = "%Y-%m-%dT%H:%M:%S-00:00"; // Loose RFC3339 format
    let mut items = Vec::new();

    // Filter out content with stream "draft"
    let filtered_contents: Vec<&Content> = contents
        .iter()
        .filter(|content| {
            content
                .stream
                .as_ref()
                .is_none_or(|stream| stream != "draft")
        })
        .collect();

    for content in filtered_contents.iter().take(15) {
        let item = JsonFeedItem {
            id: format!("{}/{}.html", &config.url, &content.slug),
            url: format!("{}/{}.html", &config.url, &content.slug),
            title: content.title.clone(),
            content_html: content.html.clone(),
            // content_text: content.html.clone(), // requires stripping HTML tags
            summary: content.description.clone().unwrap_or(String::new()),
            // date_published: content.date.unwrap().to_string(),
            // date published should be in RFC-822 format
            date_published: content.date.unwrap().format(date_format).to_string(),
            image: content.card_image.clone().unwrap_or(String::new()),
            authors: content
                .authors
                .iter()
                .map(|author| {
                    if let Some(config_author) = config.authors.get(author) {
                        JsonFeedAuthor {
                            name: config_author.name.clone(),
                            url: {
                                if let Some(author_links) = &config_author.links {
                                    author_links.iter().next().unwrap().1.clone()
                                } else {
                                    String::new()
                                }
                            },
                            avatar: config_author.avatar.clone().unwrap_or(String::new()),
                        }
                    } else {
                        JsonFeedAuthor {
                            name: author.clone(),
                            url: String::new(),
                            avatar: String::new(),
                        }
                    }
                })
                .collect(),
            tags: content.tags.clone(),
            language: config.language.clone(),
        };
        items.push(item);
    }

    let feed = JsonFeed {
        version: "https://jsonfeed.org/version/1".to_string(),
        title: config.name.clone(),
        home_page_url: config.url.clone(),
        feed_url: format!("{}/{}.json", &config.url, filename),
        description: config.tagline.clone(),
        items,
    };

    let json = serde_json::to_string_pretty(&feed).map_err(|e| e.to_string())?;
    let feed_path = output_path.join(format!("{filename}.json"));
    let mut file = File::create(&feed_path).map_err(|e| e.to_string())?;
    file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
    info!("Generated {}", &feed_path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::content::ContentBuilder;

    use super::*;

    fn setup_test_environment() -> (Vec<Content>, PathBuf, Marmite) {
        let contents = vec![ContentBuilder::new()
            .title("Test Title".to_string())
            .slug("test-title".to_string())
            .html("<p>Test Content</p>".to_string())
            .description("Test Description".to_string())
            .date(
                chrono::NaiveDateTime::parse_from_str("2021-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            )
            .authors(vec!["rochacbruno".to_string()])
            .tags(vec!["tag1".to_string(), "tag2".to_string()])
            .card_image("test_image.png".to_string())
            .build()];

        let output_path = PathBuf::from("/tmp");

        let config = Marmite::new();

        (contents, output_path, config)
    }

    #[test]
    fn test_generate_json() {
        let (contents, output_path, config) = setup_test_environment();
        let filename = "test_feed";

        let result = generate_json(&contents, &output_path, filename, &config);
        assert!(result.is_ok());

        let feed_path = output_path.join(format!("{filename}.json"));
        let feed_content =
            std::fs::read_to_string(feed_path).expect("Failed to read generated JSON feed");

        let json_feed: JsonFeed =
            serde_json::from_str(&feed_content).expect("Failed to parse JSON feed");
        assert_eq!(json_feed.title, config.name);
        assert_eq!(json_feed.home_page_url, config.url);
        assert_eq!(json_feed.description, config.tagline);
        assert_eq!(json_feed.items.len(), contents.len());

        let item = &json_feed.items[0];
        let content = &contents[0];
        assert_eq!(item.title, content.title);
        assert_eq!(item.url, format!("{}/{}.html", config.url, content.slug));
        assert_eq!(item.content_html, content.html);
        assert_eq!(item.summary, content.description.clone().unwrap());
        let date_format = "%Y-%m-%dT%H:%M:%S-00:00"; // Loose RFC3339 format
        assert_eq!(
            item.date_published,
            content.date.unwrap().format(date_format).to_string()
        );
        assert_eq!(item.image, content.card_image.clone().unwrap());
        assert_eq!(item.authors.len(), content.authors.len());
        assert_eq!(item.tags, content.tags);
        assert_eq!(item.language, config.language);
    }
}
