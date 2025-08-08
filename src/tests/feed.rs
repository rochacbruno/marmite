use super::*;
use crate::content::ContentBuilder;
use std::path::PathBuf;

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

#[test]
fn test_generate_rss() {
    let (contents, output_path, config) = setup_test_environment();
    let filename = "test_rss";

    let result = generate_rss(&contents, &output_path, filename, &config);
    assert!(result.is_ok());

    // Check that the RSS file was created
    let feed_path = output_path.join(format!("{filename}.rss"));
    assert!(feed_path.exists());

    // Read and parse the RSS content
    let rss_content = std::fs::read_to_string(feed_path).expect("Failed to read RSS feed");
    let channel = rss::Channel::read_from(rss_content.as_bytes()).expect("Failed to parse RSS");

    assert_eq!(channel.title(), config.name);
    assert_eq!(channel.description(), config.tagline);
    assert_eq!(channel.items().len(), contents.len());

    let item = &channel.items()[0];
    let content = &contents[0];
    assert_eq!(item.title(), Some(content.title.as_str()));
    assert!(item.link().unwrap().contains(&content.slug));
    assert_eq!(item.description(), content.description.as_deref());
}

#[test]
fn test_generate_rss_with_https_config() {
    let (contents, output_path, mut config) = setup_test_environment();
    config.url = "example.com".to_string();
    config.https = Some(true);
    let filename = "test_rss_https";

    let result = generate_rss(&contents, &output_path, filename, &config);
    assert!(result.is_ok());

    let feed_path = output_path.join(format!("{filename}.rss"));
    let rss_content = std::fs::read_to_string(feed_path).expect("Failed to read RSS feed");
    assert!(rss_content.contains("https://example.com"));
}

#[test]
fn test_generate_rss_with_http_config() {
    let (contents, output_path, mut config) = setup_test_environment();
    config.url = "example.com".to_string();
    config.https = Some(false);
    let filename = "test_rss_http";

    let result = generate_rss(&contents, &output_path, filename, &config);
    assert!(result.is_ok());

    let feed_path = output_path.join(format!("{filename}.rss"));
    let rss_content = std::fs::read_to_string(feed_path).expect("Failed to read RSS feed");
    assert!(rss_content.contains("http://example.com"));
}

#[test]
fn test_generate_rss_with_full_url() {
    let (contents, output_path, mut config) = setup_test_environment();
    config.url = "https://fullurl.com".to_string();
    let filename = "test_rss_full_url";

    let result = generate_rss(&contents, &output_path, filename, &config);
    assert!(result.is_ok());

    let feed_path = output_path.join(format!("{filename}.rss"));
    let rss_content = std::fs::read_to_string(feed_path).expect("Failed to read RSS feed");
    assert!(rss_content.contains("https://fullurl.com"));
}

#[test]
fn test_generate_rss_with_card_image() {
    let (contents, output_path, mut config) = setup_test_environment();
    config.card_image = "site-image.png".to_string();
    let filename = "test_rss_image";

    let result = generate_rss(&contents, &output_path, filename, &config);
    assert!(result.is_ok());

    let feed_path = output_path.join(format!("{filename}.rss"));
    let rss_content = std::fs::read_to_string(feed_path).expect("Failed to read RSS feed");
    let channel = rss::Channel::read_from(rss_content.as_bytes()).expect("Failed to parse RSS");

    assert!(channel.image().is_some());
    assert!(channel.image().unwrap().url().contains("site-image.png"));
}

#[test]
fn test_generate_feeds_filter_draft_content() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path();

    let contents = vec![
        ContentBuilder::new()
            .title("Published Post".to_string())
            .slug("published".to_string())
            .html("<p>Published content</p>".to_string())
            .description("Published description".to_string())
            .date(
                chrono::NaiveDateTime::parse_from_str("2021-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            )
            .stream("blog".to_string())
            .build(),
        ContentBuilder::new()
            .title("Draft Post".to_string())
            .slug("draft".to_string())
            .html("<p>Draft content</p>".to_string())
            .description("Draft description".to_string())
            .date(
                chrono::NaiveDateTime::parse_from_str("2021-01-02 00:00:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            )
            .stream("draft".to_string())
            .build(),
    ];

    let config = Marmite::new();

    // Test RSS feed filtering
    let rss_result = generate_rss(&contents, output_path, "test_filter", &config);
    assert!(rss_result.is_ok());

    let rss_path = output_path.join("test_filter.rss");
    let rss_content = std::fs::read_to_string(rss_path).expect("Failed to read RSS feed");
    let channel = rss::Channel::read_from(rss_content.as_bytes()).expect("Failed to parse RSS");

    // Should only have 1 item (draft filtered out)
    assert_eq!(channel.items().len(), 1);
    assert_eq!(channel.items()[0].title(), Some("Published Post"));

    // Test JSON feed filtering
    let json_result = generate_json(&contents, output_path, "test_filter_json", &config);
    assert!(json_result.is_ok());

    let json_path = output_path.join("test_filter_json.json");
    let json_content = std::fs::read_to_string(json_path).expect("Failed to read JSON feed");
    let feed: JsonFeed = serde_json::from_str(&json_content).expect("Failed to parse JSON");

    // Should only have 1 item (draft filtered out)
    assert_eq!(feed.items.len(), 1);
    assert_eq!(feed.items[0].title, "Published Post");
}

#[test]
fn test_generate_json_with_authors_config() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path();

    let contents = vec![ContentBuilder::new()
        .title("Test Post".to_string())
        .slug("test".to_string())
        .html("<p>Test content</p>".to_string())
        .description("Test description".to_string())
        .date(
            chrono::NaiveDateTime::parse_from_str("2021-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
        )
        .authors(vec!["alice".to_string(), "unknown".to_string()])
        .build()];

    let mut config = Marmite::new();

    // Add author configuration
    let alice_links = vec![(
        "website".to_string(),
        "https://alice.example.com".to_string(),
    )];

    let alice_author = crate::config::Author {
        name: "Alice Smith".to_string(),
        bio: Some("Alice is a developer".to_string()),
        links: Some(alice_links),
        avatar: Some("alice.jpg".to_string()),
    };

    config.authors.insert("alice".to_string(), alice_author);

    let result = generate_json(&contents, output_path, "test_authors", &config);
    assert!(result.is_ok());

    let json_path = output_path.join("test_authors.json");
    let json_content = std::fs::read_to_string(json_path).expect("Failed to read JSON feed");
    let feed: JsonFeed = serde_json::from_str(&json_content).expect("Failed to parse JSON");

    assert_eq!(feed.items[0].authors.len(), 2);

    // Check configured author
    let alice_author = &feed.items[0].authors[0];
    assert_eq!(alice_author.name, "Alice Smith");
    assert_eq!(alice_author.url, "https://alice.example.com");
    assert_eq!(alice_author.avatar, "alice.jpg");

    // Check unconfigured author (should use defaults)
    let unknown_author = &feed.items[0].authors[1];
    assert_eq!(unknown_author.name, "unknown");
    assert_eq!(unknown_author.url, "");
    assert_eq!(unknown_author.avatar, "");
}

#[test]
fn test_generate_feeds_empty_content() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path();
    let contents: Vec<Content> = vec![];
    let config = Marmite::new();

    // Test RSS with empty content
    let rss_result = generate_rss(&contents, output_path, "empty_rss", &config);
    assert!(rss_result.is_ok());

    let rss_path = output_path.join("empty_rss.rss");
    let rss_content = std::fs::read_to_string(rss_path).expect("Failed to read RSS feed");
    let channel = rss::Channel::read_from(rss_content.as_bytes()).expect("Failed to parse RSS");
    assert_eq!(channel.items().len(), 0);

    // Test JSON with empty content
    let json_result = generate_json(&contents, output_path, "empty_json", &config);
    assert!(json_result.is_ok());

    let json_path = output_path.join("empty_json.json");
    let json_content = std::fs::read_to_string(json_path).expect("Failed to read JSON feed");
    let feed: JsonFeed = serde_json::from_str(&json_content).expect("Failed to parse JSON");
    assert_eq!(feed.items.len(), 0);
}
