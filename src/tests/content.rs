use super::*;
use crate::config::Marmite;
use chrono::NaiveDate;
use frontmatter_gen::{Frontmatter, Value};
use std::fs;
use std::path::Path;

#[test]
fn test_extract_stream_from_date_pattern() {
    assert_eq!(
        extract_stream_from_date_pattern("tutorial-2024-01-01-getting-started"),
        Some("tutorial".to_string())
    );
    assert_eq!(
        extract_stream_from_date_pattern("news-2024-01-15-site-update"),
        Some("news".to_string())
    );
    assert_eq!(
        extract_stream_from_date_pattern("2024-01-01-no-stream"),
        None
    );
    assert_eq!(extract_stream_from_date_pattern("invalid-format"), None);
}

#[test]
fn test_extract_stream_from_s_pattern() {
    assert_eq!(
        extract_stream_from_s_pattern("guide-S-my-comprehensive-guide"),
        Some("guide".to_string())
    );
    assert_eq!(
        extract_stream_from_s_pattern("tutorial-S-advanced-tips"),
        Some("tutorial".to_string())
    );
    assert_eq!(extract_stream_from_s_pattern("no-s-pattern-here"), None);
}

#[test]
fn test_get_stream_from_filename() {
    use std::path::Path;

    assert_eq!(
        get_stream_from_filename(Path::new("tutorial-2024-01-01-getting-started.md")),
        Some("tutorial".to_string())
    );
    assert_eq!(
        get_stream_from_filename(Path::new("guide-S-my-guide.md")),
        Some("guide".to_string())
    );
    assert_eq!(
        get_stream_from_filename(Path::new("2024-01-01-no-stream.md")),
        None
    );
}

#[test]
fn test_get_title_from_frontmatter() {
    let mut frontmatter = Frontmatter::new();
    frontmatter.insert("title".to_string(), Value::String("Test Title".to_string()));
    let markdown = "# HTML Title";

    let (title, markdown) = get_title(&frontmatter, markdown);
    assert_eq!(title, "Test Title");
    assert!(markdown.contains("HTML Title"));
}

#[test]
fn test_get_title_from_html() {
    let frontmatter = Frontmatter::new();
    let markdown = "# HTML Title";

    let (title, markdown) = get_title(&frontmatter, markdown);
    assert_eq!(title, "HTML Title");
    assert!(!markdown.contains("HTML Title"));
}

#[test]
fn test_get_title_from_html_with_no_title_tag() {
    let frontmatter = Frontmatter::new();
    let markdown = "title here";

    let (title, markdown) = get_title(&frontmatter, markdown);
    assert_eq!(title, "title here");
    assert!(!markdown.contains("title here"));
}

#[test]
fn test_get_title_from_html_with_multiple_lines() {
    let frontmatter = Frontmatter::new();
    let markdown = "
# First Title
Second Title
    ";

    let (title, markdown) = get_title(&frontmatter, markdown);
    assert_eq!(title, "First Title");
    assert!(!markdown.contains("First Title"));
    assert!(markdown.contains("Second Title"));
}

#[test]
fn test_get_description_from_frontmatter() {
    let mut frontmatter = Frontmatter::new();
    frontmatter.insert(
        "description".to_string(),
        Value::String("Test Description".to_string()),
    );

    let description = get_description(&frontmatter);
    assert_eq!(description, Some("\"Test Description\"".to_string()));
}

#[test]
fn test_get_description_from_empty_frontmatter() {
    let frontmatter = Frontmatter::new();

    let description = get_description(&frontmatter);
    assert_eq!(description, None);
}

#[test]
fn test_get_slug_from_frontmatter() {
    let mut frontmatter = Frontmatter::new();
    frontmatter.insert("slug".to_string(), Value::String("test-slug".to_string()));
    let path = Path::new("2024-01-01-myfile.md");

    let slug = get_slug(&frontmatter, path);
    assert_eq!(slug, "test-slug");
}

#[test]
fn test_get_slug_from_title() {
    let mut frontmatter = Frontmatter::new();
    frontmatter.insert("title".to_string(), Value::String("Test Title".to_string()));
    let path = Path::new("2024-01-01-myfile.md");

    let slug = get_slug(&frontmatter, path);
    assert_eq!(slug, "test-title");
}

#[test]
fn test_get_slug_from_filename() {
    let frontmatter = Frontmatter::new();
    let path = Path::new("2024-01-01-myfile.md");

    let slug = get_slug(&frontmatter, path);
    assert_eq!(slug, "myfile");
}

#[test]
fn test_get_slug_from_filename_without_date() {
    let frontmatter = Frontmatter::new();
    let path = Path::new("myfile.md");

    let slug = get_slug(&frontmatter, path);
    assert_eq!(slug, "myfile");
}

#[test]
fn test_get_slug_from_various_filenames() {
    let frontmatter = Frontmatter::new();
    let filenames = vec![
        "my-file.md",
        "2024-01-01-my-file.md",
        "2024-01-01-15-30-my-file.md",
        "2024-01-01-15-30-12-my-file.md",
        "2024-01-01T15:30-my-file.md",
        "2024-01-01T15:30:12-my-file.md",
    ];

    for filename in filenames {
        let path = Path::new(filename);
        let slug = get_slug(&frontmatter, path);
        assert_eq!(slug, "my-file", "Failed for filename: {filename}");
    }
}

#[test]
fn test_get_slug_with_special_characters() {
    let mut frontmatter = Frontmatter::new();
    frontmatter.insert(
        "title".to_string(),
        Value::String("Test Title with Special Characters!@#".to_string()),
    );
    let path = Path::new("2024-01-01-myfile.md");

    let slug = get_slug(&frontmatter, path);
    assert_eq!(slug, "test-title-with-special-characters");
}

#[test]
fn test_get_tags_from_frontmatter_array() {
    let mut frontmatter = Frontmatter::new();
    frontmatter.insert(
        "tags".to_string(),
        Value::Array(vec![
            Value::String("tag1".to_string()),
            Value::String("tag2".to_string()),
        ]),
    );

    let tags = get_tags(&frontmatter);
    assert_eq!(tags, vec!["tag1", "tag2"]);
}

#[test]
fn test_get_tags_from_frontmatter_string() {
    let mut frontmatter = Frontmatter::new();
    frontmatter.insert("tags".to_string(), Value::String("tag1, tag2".to_string()));

    let tags = get_tags(&frontmatter);
    assert_eq!(tags, vec!["tag1", "tag2"]);
}

#[test]
fn test_get_tags_with_no_tags() {
    let frontmatter = Frontmatter::new();

    let tags = get_tags(&frontmatter);
    assert!(tags.is_empty());
}

#[test]
fn test_get_tags_with_empty_str() {
    let mut frontmatter = Frontmatter::new();
    frontmatter.insert("tags".to_string(), Value::String(String::new()));

    let tags = get_tags(&frontmatter);
    assert!(tags.is_empty());
}

#[test]
fn test_get_date_from_frontmatter() {
    let mut frontmatter = Frontmatter::new();
    frontmatter.insert(
        "date".to_string(),
        Value::String("2024-01-01 15:40:56".to_string()),
    );
    let path = Path::new("myfile.md");

    let date = get_date(&frontmatter, path).unwrap();
    assert_eq!(
        date,
        NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(15, 40, 56)
            .unwrap()
    );
}

#[test]
fn test_get_date_from_frontmatter_without_time() {
    let mut frontmatter = Frontmatter::new();
    frontmatter.insert("date".to_string(), Value::String("2024-01-01".to_string()));
    let path = Path::new("myfile.md");

    let date = get_date(&frontmatter, path).unwrap();
    assert_eq!(
        date,
        NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    );
}

#[test]
fn test_get_date_from_filename() {
    let frontmatter = Frontmatter::new();
    let path = Path::new("2024-01-01-myfile.md");

    let date = get_date(&frontmatter, path).unwrap();
    assert_eq!(
        date,
        NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    );
}

#[test]
fn test_get_date_no_date() {
    let frontmatter = Frontmatter::new();
    let path = Path::new("myfile.md");

    let date = get_date(&frontmatter, path);
    assert!(date.is_none());
}

#[test]
fn test_slugify_simple_text() {
    let text = "Simple Text";
    let slug = slugify(text);
    assert_eq!(slug, "simple-text");
}

#[test]
fn test_slugify_with_special_characters() {
    let text = "Text with Special Characters!@#";
    let slug = slugify(text);
    assert_eq!(slug, "text-with-special-characters");
}

#[test]
fn test_slugify_with_accents() {
    let text = "Téxt wíth Áccénts";
    let slug = slugify(text);
    assert_eq!(slug, "te-xt-wi-th-a-cce-nts");
}

#[test]
fn test_slugify_with_multiple_spaces() {
    let text = "Text    with    multiple    spaces";
    let slug = slugify(text);
    assert_eq!(slug, "text-with-multiple-spaces");
}

#[test]
fn test_slugify_with_underscores() {
    let text = "Text_with_underscores";
    let slug = slugify(text);
    assert_eq!(slug, "text-with-underscores");
}

#[test]
fn test_slugify_with_numbers() {
    let text = "Text with numbers 123";
    let slug = slugify(text);
    assert_eq!(slug, "text-with-numbers-123");
}

#[test]
fn test_slugify_empty_string() {
    let text = "";
    let slug = slugify(text);
    assert_eq!(slug, "");
}

#[test]
fn test_check_for_duplicate_slugs_no_duplicates() {
    let post1: Content = ContentBuilder::new()
        .title("Title 1".to_string())
        .slug("slug-1".to_string())
        .build();

    let post2: Content = ContentBuilder::new()
        .title("Title 2".to_string())
        .slug("slug-2".to_string())
        .build();

    let contents = vec![&post1, &post2];
    let result = check_for_duplicate_slugs(&contents);
    assert!(result.is_ok());
}

#[test]
fn test_check_for_duplicate_slugs_with_duplicates() {
    let post1: Content = ContentBuilder::new()
        .title("Title 1".to_string())
        .slug("duplicate-slug".to_string())
        .build();

    let post2: Content = ContentBuilder::new()
        .title("Title 2".to_string())
        .slug("duplicate-slug".to_string())
        .build();

    let contents = vec![&post1, &post2];

    let result = check_for_duplicate_slugs(&contents);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), "duplicate-slug".to_string());
}

#[test]
fn test_check_for_duplicate_slugs_empty_list() {
    let contents: Vec<&Content> = vec![];

    let result = check_for_duplicate_slugs(&contents);
    assert!(result.is_ok());
}

#[test]
fn test_extract_date_from_filename_valid_date() {
    let path = Path::new("2024-01-01-myfile.md");
    let date = extract_date_from_filename(path).unwrap();
    assert_eq!(
        date,
        NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    );
}

#[test]
fn test_extract_date_from_filename_invalid_date() {
    let path = Path::new("not-a-date-myfile.md");
    let date = extract_date_from_filename(path);
    assert!(date.is_none());
}

#[test]
fn test_extract_date_from_filename_empty() {
    let path = Path::new("");
    let date = extract_date_from_filename(path);
    assert!(date.is_none());
}

#[test]
fn test_extract_date_from_filename_with_time() {
    let path = Path::new("2024-01-01-15-30-myfile.md");
    let date = extract_date_from_filename(path).unwrap();
    assert_eq!(
        date,
        NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    );
}

#[test]
fn test_extract_date_from_filename_with_multiple_dates() {
    let path = Path::new("2024-01-01-2025-02-02-myfile.md");
    let date = extract_date_from_filename(path).unwrap();
    assert_eq!(
        date,
        NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    );
}

#[test]
fn test_extract_date_from_filename_with_stream_prefix() {
    let path = Path::new("news-2024-01-15-site-update.md");
    let date = extract_date_from_filename(path).unwrap();
    assert_eq!(
        date,
        NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    );
}

#[test]
fn test_extract_date_from_filename_with_stream_prefix_various() {
    let test_cases = vec![
        ("tutorial-2024-01-01-getting-started.md", (2024, 1, 1)),
        ("news-2024-01-15-site-update.md", (2024, 1, 15)),
        ("test-2024-01-01-simple-test.md", (2024, 1, 1)),
    ];

    for (filename, (year, month, day)) in test_cases {
        let path = Path::new(filename);
        let date = extract_date_from_filename(path).unwrap();
        let expected = NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        assert_eq!(date, expected, "Failed for filename: {filename}");
    }
}

#[test]
fn test_try_to_parse_date() {
    let inputs = vec![
        "2024-01-01",
        "2024-01-01 15:40",
        "2024-01-01-15:40",
        "2024-01-01 15:40:56",
        "2024-01-01-15:40:56",
        "2024-01-01 15:40:56.123Z",
        "2024-01-01T15:40",
        "2024-01-01T15:40:56",
        "2024-01-01T15:40:56.123Z",
        "2024-01-01T15:40:56+0000",
        "2024-01-01T15:40:56.123+0000",
        "2024-01-01T15:40:56.123456+0000",
        "2024-01-01T15:40:56.123456Z",
        "2024-01-01T15:40:56.123456789+0000",
        "2024-01-01T15:40:56.123456789Z",
        "2020-01-19T21:05:12.984Z",
        "2020-01-19T21:05:12+0000",
        "2024-11-22 20:29:53.211984268 +00:00",
    ];

    for input in inputs {
        let date = try_to_parse_date(input);
        assert!(date.is_ok(), "Failed for input: {input}");
    }
}

#[test]
fn test_get_card_image_from_frontmatter() {
    let mut frontmatter = Frontmatter::new();
    frontmatter.insert(
        "card_image".to_string(),
        frontmatter_gen::Value::String("media/image.jpg".to_string()),
    );
    let html = r#"<p>Some content</p><img src="media/other.jpg" />"#;
    let expected = Some("\"media/image.jpg\"".to_string());
    // assert_eq!(get_card_image(&frontmatter, html, ), expected);
    assert_eq!(
        get_card_image(&frontmatter, html, Path::new("test"), "test", "media"),
        expected
    );
}

#[test]
fn test_get_card_image_from_html() {
    let frontmatter = Frontmatter::new();
    let html = r#"<p>Some content</p><img src="media/image.jpg" />"#;
    let expected = Some("media/image.jpg".to_string());
    assert_eq!(
        get_card_image(&frontmatter, html, Path::new("test"), "test", "media"),
        expected
    );
}

#[test]
fn test_get_card_image_no_image() {
    let frontmatter = Frontmatter::new();
    let html = "<p>Some content</p>";
    let expected: Option<String> = None;
    assert_eq!(
        get_card_image(&frontmatter, html, Path::new("test"), "test", "media"),
        expected
    );
}

#[test]
fn test_get_card_image_with_multiple_images() {
    let frontmatter = Frontmatter::new();
    let html = r#"<p>Some content</p><img src="image1.jpg" /><img src="image2.jpg" />"#;
    let expected = Some("image1.jpg".to_string());
    assert_eq!(
        get_card_image(&frontmatter, html, Path::new("test"), "test", "media"),
        expected
    );
}

#[test]
fn test_get_card_image_with_invalid_html() {
    let frontmatter = Frontmatter::new();
    let html = r#"<p>Some content</p><img src="image.jpg"#;
    let expected: Option<String> = None;
    assert_eq!(
        get_card_image(&frontmatter, html, Path::new("test"), "test", "media"),
        expected
    );
}

#[test]
fn test_get_content_with_valid_frontmatter() {
    let path = Path::new("test_get_content_with_valid_frontmatter.md");
    let content = r#"
---
title: Test Title
description: "Test Description"
tags: ["tag1", "tag2"]
slug: "test-title"
date: "2023-01-01"
---
# Test Content
This is a test content.
"#;
    fs::write(path, content).unwrap();
    let result = Content::from_markdown(path, None, &Marmite::default(), None).unwrap();
    assert_eq!(result.title, "Test Title");
    assert_eq!(result.description, Some("\"Test Description\"".to_string()));
    assert_eq!(result.slug, "test-title");
    assert_eq!(result.tags, vec!["tag1".to_string(), "tag2".to_string()]);
    assert_eq!(result.date.unwrap().to_string(), "2023-01-01 00:00:00");
    assert_eq!(result.html, "<h1><a href=\"#test-content\" aria-hidden=\"true\" class=\"anchor\" id=\"test-content\"></a>Test Content</h1>\n<p>This is a test content.</p>\n");
    fs::remove_file(path).unwrap();
}

#[test]
fn test_get_content_with_invalid_frontmatter() {
    let path = Path::new("test_get_content_with_invalid_frontmatter.md");
    let content = r#"
---
title: "Test Title"
description: "Test Description"
tags: ["tag1", "tag2"
slug: "test-title"
date: "2023-01-01"
extra: "extra content"
---
# Test Content
This is a test content.
"#;
    fs::write(path, content).unwrap();
    let result = Content::from_markdown(path, None, &Marmite::default(), None);
    assert!(result.is_err());
    fs::remove_file(path).unwrap();
}

#[test]
fn test_get_content_without_frontmatter() {
    let path = Path::new("test_get_content_without_frontmatter.md");
    let content = r"
# Test Content
This is a test content.
";
    fs::write(path, content).unwrap();
    let result = Content::from_markdown(path, None, &Marmite::default(), None).unwrap();
    assert_eq!(result.title, "Test Content".to_string());
    assert_eq!(result.description, None);
    assert_eq!(result.slug, "test_get_content_without_frontmatter");
    assert!(result.tags.is_empty());
    assert!(result.date.is_none());
    assert!(result.extra.is_none());
    assert_eq!(result.html, "<p>This is a test content.</p>\n");
    fs::remove_file(path).unwrap();
}

#[test]
fn test_get_content_with_empty_file() {
    let path = Path::new("test_get_content_with_empty_file.md");
    let content = "";
    fs::write(path, content).unwrap();
    let result = Content::from_markdown(path, None, &Marmite::default(), None).unwrap();
    assert_eq!(result.slug, "test_get_content_with_empty_file".to_string());
    fs::remove_file(path).unwrap();
}
