use super::*;
use crate::config::Marmite;
use chrono::NaiveDate;
use frontmatter_gen::{Frontmatter, Value};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

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
    let slug = slug::slugify(text);
    assert_eq!(slug, "simple-text");
}

#[test]
fn test_slugify_with_special_characters() {
    let text = "Text with Special Characters!@#";
    let slug = slug::slugify(text);
    assert_eq!(slug, "text-with-special-characters");
}

#[test]
fn test_slugify_with_accents() {
    let text = "Téxt wíth Áccénts";
    let slug = slug::slugify(text);
    assert_eq!(slug, "text-with-accents");
}

#[test]
fn test_slugify_with_multiple_spaces() {
    let text = "Text    with    multiple    spaces";
    let slug = slug::slugify(text);
    assert_eq!(slug, "text-with-multiple-spaces");
}

#[test]
fn test_slugify_with_underscores() {
    let text = "Text_with_underscores";
    let slug = slug::slugify(text);
    assert_eq!(slug, "text-with-underscores");
}

#[test]
fn test_slugify_with_numbers() {
    let text = "Text with numbers 123";
    let slug = slug::slugify(text);
    assert_eq!(slug, "text-with-numbers-123");
}

#[test]
fn test_slugify_empty_string() {
    let text = "";
    let slug = slug::slugify(text);
    assert_eq!(slug, "");
}

#[test]
fn test_slugify_comunicacao() {
    let text = "Comunicação";
    let slug = slug::slugify(text);
    assert_eq!(slug, "comunicacao");
}

#[test]
fn test_slugify_programacao() {
    let text = "Programação";
    let slug = slug::slugify(text);
    assert_eq!(slug, "programacao");
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
        get_card_image(&frontmatter, html, Path::new("test"), "test", "media", None),
        expected
    );
}

#[test]
fn test_get_card_image_from_html() {
    let frontmatter = Frontmatter::new();
    let html = r#"<p>Some content</p><img src="media/image.jpg" />"#;
    let expected = Some("media/image.jpg".to_string());
    assert_eq!(
        get_card_image(&frontmatter, html, Path::new("test"), "test", "media", None),
        expected
    );
}

#[test]
fn test_get_card_image_no_image() {
    let frontmatter = Frontmatter::new();
    let html = "<p>Some content</p>";
    let expected: Option<String> = None;
    assert_eq!(
        get_card_image(&frontmatter, html, Path::new("test"), "test", "media", None),
        expected
    );
}

#[test]
fn test_get_card_image_with_multiple_images() {
    let frontmatter = Frontmatter::new();
    let html = r#"<p>Some content</p><img src="image1.jpg" /><img src="image2.jpg" />"#;
    let expected = Some("image1.jpg".to_string());
    assert_eq!(
        get_card_image(&frontmatter, html, Path::new("test"), "test", "media", None),
        expected
    );
}

#[test]
fn test_get_card_image_with_invalid_html() {
    let frontmatter = Frontmatter::new();
    let html = r#"<p>Some content</p><img src="image.jpg"#;
    let expected: Option<String> = None;
    assert_eq!(
        get_card_image(&frontmatter, html, Path::new("test"), "test", "media", None),
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
    let result =
        Content::from_markdown(path, None, &Marmite::default(), None, None, None, None).unwrap();
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
    let result = Content::from_markdown(path, None, &Marmite::default(), None, None, None, None);
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
    let result =
        Content::from_markdown(path, None, &Marmite::default(), None, None, None, None).unwrap();
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
    let result =
        Content::from_markdown(path, None, &Marmite::default(), None, None, None, None).unwrap();
    assert_eq!(result.slug, "test_get_content_with_empty_file".to_string());
    fs::remove_file(path).unwrap();
}

#[test]
fn test_find_matching_file_subfolder_banner() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path();
    let media_dir = content_dir.join("media").join("test-slug");
    fs::create_dir_all(&media_dir).unwrap();
    fs::write(media_dir.join("banner.png"), b"fake image").unwrap();

    let dummy_file = content_dir.join("test.md");
    fs::write(&dummy_file, "").unwrap();

    let result = find_matching_file(
        "test-slug",
        &dummy_file,
        "banner",
        &["png", "jpg"],
        "media",
        None,
    );
    assert_eq!(result, Some("media/test-slug/banner.png".to_string()));
}

#[test]
fn test_find_matching_file_subfolder_card() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path();
    let media_dir = content_dir.join("media").join("test-slug");
    fs::create_dir_all(&media_dir).unwrap();
    fs::write(media_dir.join("card.jpg"), b"fake image").unwrap();

    let dummy_file = content_dir.join("test.md");
    fs::write(&dummy_file, "").unwrap();

    let result = find_matching_file(
        "test-slug",
        &dummy_file,
        "card",
        &["png", "jpg"],
        "media",
        None,
    );
    assert_eq!(result, Some("media/test-slug/card.jpg".to_string()));
}

#[test]
fn test_find_matching_file_subfolder_takes_precedence_over_flat() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path();
    let media_dir = content_dir.join("media");
    fs::create_dir_all(media_dir.join("test-slug")).unwrap();
    fs::write(media_dir.join("test-slug.banner.png"), b"flat").unwrap();
    fs::write(media_dir.join("test-slug").join("banner.png"), b"subfolder").unwrap();

    let dummy_file = content_dir.join("test.md");
    fs::write(&dummy_file, "").unwrap();

    let result = find_matching_file("test-slug", &dummy_file, "banner", &["png"], "media", None);
    assert_eq!(
        result,
        Some("media/test-slug/banner.png".to_string()),
        "Subfolder should take precedence over flat file"
    );
}

#[test]
fn test_find_matching_file_subfolder_slug_named() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path();
    let media_dir = content_dir.join("media").join("my-post");
    fs::create_dir_all(&media_dir).unwrap();
    fs::write(media_dir.join("my-post.jpg"), b"fake image").unwrap();

    let dummy_file = content_dir.join("test.md");
    fs::write(&dummy_file, "").unwrap();

    let result = find_matching_file(
        "my-post",
        &dummy_file,
        "card",
        &["png", "jpg"],
        "media",
        None,
    );
    assert_eq!(result, Some("media/my-post/my-post.jpg".to_string()));
}

#[test]
fn test_find_matching_file_subfolder_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path();
    fs::create_dir_all(content_dir.join("media")).unwrap();

    let dummy_file = content_dir.join("test.md");
    fs::write(&dummy_file, "").unwrap();

    let result = find_matching_file(
        "no-such-slug",
        &dummy_file,
        "banner",
        &["png", "jpg"],
        "media",
        None,
    );
    assert_eq!(result, None);
}

#[test]
fn test_at_prefix_replacement_in_content() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path();
    fs::create_dir_all(content_dir.join("media").join("my-post")).unwrap();
    let path = content_dir.join("2024-01-01-my-post.md");
    let content = "---\ntitle: My Post\nslug: my-post\ndate: 2024-01-01\n---\n![](@/photo.png)\n";
    fs::write(&path, content).unwrap();

    let result = Content::from_markdown(
        &path,
        None,
        &Marmite::new(),
        None,
        None,
        None,
        Some(content_dir),
    )
    .unwrap();
    assert!(
        result.html.contains("media/my-post/photo.png"),
        "Expected @/ to be replaced with media/my-post/, got: {}",
        result.html
    );
}

#[test]
fn test_at_prefix_fallback_to_root_media() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path();
    fs::create_dir_all(content_dir.join("media")).unwrap();
    let path = content_dir.join("2024-01-01-my-post.md");
    let content = "---\ntitle: My Post\nslug: my-post\ndate: 2024-01-01\n---\n![](@/photo.png)\n";
    fs::write(&path, content).unwrap();

    let result = Content::from_markdown(
        &path,
        None,
        &Marmite::new(),
        None,
        None,
        None,
        Some(content_dir),
    )
    .unwrap();
    assert!(
        result.html.contains("media/photo.png"),
        "Expected @/ to fall back to media/ when no slug subfolder exists, got: {}",
        result.html
    );
}

#[test]
fn test_at_prefix_replacement_multiple_occurrences() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path();
    fs::create_dir_all(content_dir.join("media").join("my-post")).unwrap();
    let path = content_dir.join("2024-01-01-my-post.md");
    let content =
        "---\ntitle: My Post\nslug: my-post\ndate: 2024-01-01\n---\n![](@/a.png)\n\n[PDF](@/doc.pdf)\n";
    fs::write(&path, content).unwrap();

    let result = Content::from_markdown(
        &path,
        None,
        &Marmite::new(),
        None,
        None,
        None,
        Some(content_dir),
    )
    .unwrap();
    assert!(
        result.html.contains("media/my-post/a.png"),
        "got: {}",
        result.html
    );
    assert!(
        result.html.contains("media/my-post/doc.pdf"),
        "got: {}",
        result.html
    );
}

#[test]
fn test_at_prefix_no_replacement_in_fragments() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("_fragment.md");
    let content = "---\ntitle: Fragment\n---\n![](@/should-stay.png)\n";
    fs::write(&path, content).unwrap();

    let result =
        Content::from_markdown(&path, None, &Marmite::new(), None, None, None, None).unwrap();
    assert!(
        result.html.contains("@/should-stay.png"),
        "Fragments should not have @/ replaced, got: {}",
        result.html
    );
}

#[test]
fn test_at_prefix_with_custom_media_path() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path();
    fs::create_dir_all(content_dir.join("assets").join("my-post")).unwrap();
    let path = content_dir.join("2024-01-01-my-post.md");
    let content = "---\ntitle: My Post\nslug: my-post\ndate: 2024-01-01\n---\n![](@/photo.png)\n";
    fs::write(&path, content).unwrap();

    let mut config = Marmite::new();
    config.media_path = "assets".to_string();

    let result =
        Content::from_markdown(&path, None, &config, None, None, None, Some(content_dir)).unwrap();
    assert!(
        result.html.contains("assets/my-post/photo.png"),
        "Expected @/ to use custom media_path 'assets', got: {}",
        result.html
    );
}

#[test]
fn test_at_prefix_not_replaced_in_plain_text() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("2024-01-01-my-post.md");
    let content =
        "---\ntitle: My Post\nslug: my-post\ndate: 2024-01-01\n---\nHello, this is my symbol @/ and should not be replaced\n";
    fs::write(&path, content).unwrap();

    let result =
        Content::from_markdown(&path, None, &Marmite::new(), None, None, None, None).unwrap();
    assert!(
        result.html.contains("@/"),
        "Plain text @/ should not be replaced, got: {}",
        result.html
    );
    assert!(
        !result.html.contains("media/my-post/"),
        "Plain text @/ should not become a media path, got: {}",
        result.html
    );
}

#[test]
fn test_at_prefix_not_replaced_in_code_block() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("2024-01-01-my-post.md");
    let content = "---\ntitle: My Post\nslug: my-post\ndate: 2024-01-01\n---\nHere is how you use it:\n\n```markdown\n![](@/foo.png)\n```\n";
    fs::write(&path, content).unwrap();

    let result =
        Content::from_markdown(&path, None, &Marmite::new(), None, None, None, None).unwrap();
    assert!(
        !result.html.contains("media/my-post/foo.png"),
        "Code block @/ should not be replaced, got: {}",
        result.html
    );
}

#[test]
fn test_get_aliases_from_array() {
    let mut frontmatter = Frontmatter::new();
    frontmatter.insert(
        "aliases".to_string(),
        Value::Array(vec![
            Value::String("old-url".to_string()),
            Value::String("legacy-path".to_string()),
        ]),
    );

    let aliases = get_aliases(&frontmatter);
    assert_eq!(aliases, vec!["old-url", "legacy-path"]);
}

#[test]
fn test_get_aliases_from_string() {
    let mut frontmatter = Frontmatter::new();
    frontmatter.insert(
        "aliases".to_string(),
        Value::String("old-url, legacy-path".to_string()),
    );

    let aliases = get_aliases(&frontmatter);
    assert_eq!(aliases, vec!["old-url", "legacy-path"]);
}

#[test]
fn test_get_aliases_empty() {
    let frontmatter = Frontmatter::new();

    let aliases = get_aliases(&frontmatter);
    assert!(aliases.is_empty());
}

#[test]
fn test_get_aliases_with_empty_string() {
    let mut frontmatter = Frontmatter::new();
    frontmatter.insert("aliases".to_string(), Value::String(String::new()));

    let aliases = get_aliases(&frontmatter);
    assert!(aliases.is_empty());
}

#[test]
fn test_aliases_parsed_from_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("2024-01-01-my-post.md");
    let content = "---\ntitle: My Post\nslug: my-post\ndate: 2024-01-01\naliases:\n  - old-post\n  - legacy-post\n---\n# Content\n";
    fs::write(&path, content).unwrap();

    let result =
        Content::from_markdown(&path, None, &Marmite::default(), None, None, None, None).unwrap();
    assert_eq!(result.aliases, vec!["old-post", "legacy-post"]);
}

#[test]
fn test_aliases_empty_when_not_specified() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("2024-01-01-my-post.md");
    let content = "---\ntitle: My Post\nslug: my-post\ndate: 2024-01-01\n---\n# Content\n";
    fs::write(&path, content).unwrap();

    let result =
        Content::from_markdown(&path, None, &Marmite::default(), None, None, None, None).unwrap();
    assert!(result.aliases.is_empty());
}

#[test]
fn test_detect_language_from_subfolder_with_prefix() {
    let content_dir = Path::new("/site/content");
    let path = Path::new("/site/content/hello/pt-ola.md");
    assert_eq!(
        detect_language_from_path(path, content_dir),
        Some("pt".to_string())
    );
}

#[test]
fn test_detect_language_from_subfolder_en_prefix() {
    let content_dir = Path::new("/site/content");
    let path = Path::new("/site/content/hello/en-hello-world.md");
    assert_eq!(
        detect_language_from_path(path, content_dir),
        Some("en".to_string())
    );
}

#[test]
fn test_detect_language_flat_file_no_detection() {
    let content_dir = Path::new("/site/content");
    let path = Path::new("/site/content/pt-ola.md");
    assert_eq!(detect_language_from_path(path, content_dir), None);
}

#[test]
fn test_detect_language_without_config() {
    let content_dir = Path::new("/site/content");
    let path = Path::new("/site/content/hello/pt-ola.md");
    assert_eq!(
        detect_language_from_path(path, content_dir),
        Some("pt".to_string())
    );
}

#[test]
fn test_detect_language_no_false_positive() {
    let content_dir = Path::new("/site/content");
    let path = Path::new("/site/content/hello/essential-guide.md");
    assert_eq!(detect_language_from_path(path, content_dir), None);
}

#[test]
fn test_detect_language_base_content_no_prefix() {
    let content_dir = Path::new("/site/content");
    let path = Path::new("/site/content/hello/hello.md");
    assert_eq!(detect_language_from_path(path, content_dir), None);
}

#[test]
fn test_language_frontmatter_parsed() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("hello.md");
    let content = "---\ntitle: Hello\nlanguage: en\n---\n# Hello\n";
    fs::write(&path, content).unwrap();

    let result =
        Content::from_markdown(&path, None, &Marmite::default(), None, None, None, None).unwrap();
    assert_eq!(result.language, Some("en".to_string()));
}

#[test]
fn test_translations_frontmatter_parsed() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("2024-01-01-hello.md");
    let content = "---\ntitle: Hello\ndate: 2024-01-01\ntranslations:\n  - pt-ola\n  - es-hola\n---\n# Hello\n";
    fs::write(&path, content).unwrap();

    let result =
        Content::from_markdown(&path, None, &Marmite::default(), None, None, None, None).unwrap();
    assert_eq!(result.translations.len(), 2);
    assert_eq!(result.translations[0].slug, "pt-ola");
    assert_eq!(result.translations[1].slug, "es-hola");
}

#[test]
fn test_translations_frontmatter_empty_when_not_specified() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("hello.md");
    let content = "---\ntitle: Hello\n---\n# Hello\n";
    fs::write(&path, content).unwrap();

    let result =
        Content::from_markdown(&path, None, &Marmite::default(), None, None, None, None).unwrap();
    assert!(result.translations.is_empty());
}

#[test]
fn test_translates_frontmatter_parsed() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("ola.md");
    let content = "---\ntitle: Ola\nlanguage: pt\ntranslates: hello\n---\n# Ola\n";
    fs::write(&path, content).unwrap();

    let result =
        Content::from_markdown(&path, None, &Marmite::default(), None, None, None, None).unwrap();
    assert_eq!(result.translates, Some("hello".to_string()));
    assert_eq!(result.language, Some("pt".to_string()));
}

#[test]
fn test_translates_frontmatter_empty_when_not_specified() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("hello.md");
    let content = "---\ntitle: Hello\n---\n# Hello\n";
    fs::write(&path, content).unwrap();

    let result =
        Content::from_markdown(&path, None, &Marmite::default(), None, None, None, None).unwrap();
    assert!(result.translates.is_none());
}

#[test]
fn test_is_iso_639_1_code() {
    use crate::content::is_iso_639_1_code;

    assert!(is_iso_639_1_code("en"));
    assert!(is_iso_639_1_code("pt"));
    assert!(is_iso_639_1_code("es"));
    assert!(is_iso_639_1_code("zh"));
    assert!(!is_iso_639_1_code("xyz"));
    assert!(!is_iso_639_1_code("english"));
    assert!(!is_iso_639_1_code(""));
}

#[test]
fn test_merge_frontmatter_inherits_defaults() {
    let mut defaults = Frontmatter::new();
    defaults.insert("stream".to_string(), Value::String("python".to_string()));
    defaults.insert(
        "tags".to_string(),
        Value::Array(vec![Value::String("python".to_string())]),
    );

    let mut file_fm = Frontmatter::new();
    merge_frontmatter(&defaults, &mut file_fm);

    assert_eq!(
        file_fm.get("stream").and_then(|v| v.as_str()),
        Some("python")
    );
    assert!(file_fm.get("tags").is_some());
}

#[test]
fn test_merge_frontmatter_file_overrides() {
    let mut defaults = Frontmatter::new();
    defaults.insert("stream".to_string(), Value::String("python".to_string()));

    let mut file_fm = Frontmatter::new();
    file_fm.insert("stream".to_string(), Value::String("rust".to_string()));

    merge_frontmatter(&defaults, &mut file_fm);

    assert_eq!(file_fm.get("stream").and_then(|v| v.as_str()), Some("rust"));
}

#[test]
fn test_merge_frontmatter_title_slug_excluded() {
    let mut defaults = Frontmatter::new();
    defaults.insert("title".to_string(), Value::String("Default".to_string()));
    defaults.insert("slug".to_string(), Value::String("default".to_string()));
    defaults.insert("stream".to_string(), Value::String("python".to_string()));

    let mut file_fm = Frontmatter::new();
    merge_frontmatter(&defaults, &mut file_fm);

    assert!(file_fm.get("title").is_none());
    assert!(file_fm.get("slug").is_none());
    assert_eq!(
        file_fm.get("stream").and_then(|v| v.as_str()),
        Some("python")
    );
}

#[test]
fn test_merge_frontmatter_empty_defaults() {
    let defaults = Frontmatter::new();
    let mut file_fm = Frontmatter::new();
    file_fm.insert("stream".to_string(), Value::String("rust".to_string()));

    merge_frontmatter(&defaults, &mut file_fm);

    assert_eq!(file_fm.get("stream").and_then(|v| v.as_str()), Some("rust"));
}

#[test]
fn test_merge_frontmatter_multiple_keys() {
    let mut defaults = Frontmatter::new();
    defaults.insert("stream".to_string(), Value::String("python".to_string()));
    defaults.insert("series".to_string(), Value::String("tutorial".to_string()));
    defaults.insert("pinned".to_string(), Value::Boolean(true));

    let mut file_fm = Frontmatter::new();
    file_fm.insert("stream".to_string(), Value::String("rust".to_string()));

    merge_frontmatter(&defaults, &mut file_fm);

    assert_eq!(file_fm.get("stream").and_then(|v| v.as_str()), Some("rust"));
    assert_eq!(
        file_fm.get("series").and_then(|v| v.as_str()),
        Some("tutorial")
    );
    assert_eq!(file_fm.get("pinned").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn test_from_markdown_with_folder_defaults() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.md");
    let content = "---\ntitle: My Post\n---\n# My Post\n\nContent here.\n";
    fs::write(&path, content).unwrap();

    let mut defaults = Frontmatter::new();
    defaults.insert("stream".to_string(), Value::String("python".to_string()));
    defaults.insert("date".to_string(), Value::String("2026-01-01".to_string()));

    let result = Content::from_markdown(
        &path,
        None,
        &Marmite::default(),
        None,
        None,
        Some(&defaults),
        None,
    )
    .unwrap();

    assert_eq!(result.stream.as_deref(), Some("python"));
    assert!(result.date.is_some());
    assert_eq!(result.title, "My Post");
}

#[test]
fn test_from_markdown_file_frontmatter_overrides_defaults() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.md");
    let content = "---\ntitle: My Post\nstream: rust\n---\n# My Post\n\nContent here.\n";
    fs::write(&path, content).unwrap();

    let mut defaults = Frontmatter::new();
    defaults.insert("stream".to_string(), Value::String("python".to_string()));
    defaults.insert("date".to_string(), Value::String("2026-01-01".to_string()));

    let result = Content::from_markdown(
        &path,
        None,
        &Marmite::default(),
        None,
        None,
        Some(&defaults),
        None,
    )
    .unwrap();

    assert_eq!(result.stream.as_deref(), Some("rust"));
}

#[test]
fn test_find_file_by_slug_dated_filename() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path();
    fs::write(
        content_dir.join("2024-01-01-hello-world.md"),
        "# Hello World\n",
    )
    .unwrap();

    let result = find_file_by_slug(content_dir, "hello-world");
    assert!(result.is_some());
    assert!(result
        .unwrap()
        .to_str()
        .unwrap()
        .contains("2024-01-01-hello-world.md"));
}

#[test]
fn test_find_file_by_slug_page_filename() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path();
    fs::write(content_dir.join("about.md"), "# About\n").unwrap();

    let result = find_file_by_slug(content_dir, "about");
    assert!(result.is_some());
    assert!(result.unwrap().to_str().unwrap().contains("about.md"));
}

#[test]
fn test_find_file_by_slug_in_subfolder() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path();
    let sub = content_dir.join("hello-world");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("hello-world.md"), "# Hello World\n").unwrap();

    let result = find_file_by_slug(content_dir, "hello-world");
    assert!(result.is_some());
    assert!(result
        .unwrap()
        .to_str()
        .unwrap()
        .contains("hello-world/hello-world.md"));
}

#[test]
fn test_find_file_by_slug_no_match() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path();
    fs::write(content_dir.join("other-post.md"), "# Other\n").unwrap();

    let result = find_file_by_slug(content_dir, "nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_find_file_by_slug_skips_lang_prefix() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path();
    let sub = content_dir.join("hello-world");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("hello-world.md"), "# Hello World\n").unwrap();
    fs::write(sub.join("pt-ola-mundo.md"), "# Ola Mundo\n").unwrap();

    let result = find_file_by_slug(content_dir, "hello-world");
    assert!(result.is_some());
    let path = result.unwrap();
    assert!(
        path.to_str().unwrap().contains("hello-world.md"),
        "Should find the base file, not the translation: {}",
        path.display()
    );
}

#[test]
fn test_merge_mermaid_configs_all_none() {
    let result = merge_mermaid_configs(None, None, None);
    assert!(result.is_none());
}

#[test]
fn test_merge_mermaid_configs_site_only() {
    let site: serde_yaml::Value = serde_yaml::from_str("theme: dark").unwrap();
    let result = merge_mermaid_configs(Some(&site), None, None);
    assert!(result.is_some());
    let mapping = result.unwrap().as_mapping().unwrap().clone();
    assert_eq!(
        mapping
            .get(serde_yaml::Value::String("theme".into()))
            .unwrap()
            .as_str()
            .unwrap(),
        "dark"
    );
}

#[test]
fn test_merge_mermaid_configs_page_overrides_folder() {
    let folder: serde_yaml::Value = serde_yaml::from_str("theme: forest").unwrap();
    let page: serde_yaml::Value = serde_yaml::from_str("theme: dark").unwrap();
    let result = merge_mermaid_configs(None, Some(&folder), Some(&page)).unwrap();
    let mapping = result.as_mapping().unwrap();
    assert_eq!(
        mapping
            .get(serde_yaml::Value::String("theme".into()))
            .unwrap()
            .as_str()
            .unwrap(),
        "dark"
    );
}

#[test]
fn test_merge_mermaid_configs_deep_merge_all_three() {
    let site: serde_yaml::Value = serde_yaml::from_str(
        r#"
theme: dark
flowchart:
  nodeSpacing: 50
  rankSpacing: 50
"#,
    )
    .unwrap();
    let folder: serde_yaml::Value = serde_yaml::from_str(
        r#"
flowchart:
  nodeSpacing: 80
"#,
    )
    .unwrap();
    let page: serde_yaml::Value = serde_yaml::from_str(
        r#"
flowchart:
  rankSpacing: 100
"#,
    )
    .unwrap();
    let result = merge_mermaid_configs(Some(&site), Some(&folder), Some(&page)).unwrap();
    let mapping = result.as_mapping().unwrap();

    assert_eq!(
        mapping
            .get(serde_yaml::Value::String("theme".into()))
            .unwrap()
            .as_str()
            .unwrap(),
        "dark"
    );

    let flow = mapping
        .get(serde_yaml::Value::String("flowchart".into()))
        .unwrap()
        .as_mapping()
        .unwrap();
    assert_eq!(
        flow.get(serde_yaml::Value::String("nodeSpacing".into()))
            .unwrap()
            .as_u64()
            .unwrap(),
        80
    );
    assert_eq!(
        flow.get(serde_yaml::Value::String("rankSpacing".into()))
            .unwrap()
            .as_u64()
            .unwrap(),
        100
    );
}

#[test]
fn test_create_content_basic_post() {
    let temp = TempDir::new().unwrap();
    let input = temp.path().join("site");
    let content = input.join("content");
    fs::create_dir_all(&content).unwrap();
    fs::write(input.join("marmite.yaml"), "name: Test\ntagline: t").unwrap();

    let params = CreateContentParams {
        title: "Hello World".to_string(),
        tags: None,
        directory: None,
        page: false,
        lang: None,
        translates: None,
    };
    let result = create_content(&input, &input.join("marmite.yaml"), &params).unwrap();

    assert_eq!(result.title, "Hello World");
    assert_eq!(result.slug, "hello-world");
    assert!(!result.is_page);
    assert!(result.date.is_some());
    assert!(result.file_path.exists());
    assert_eq!(
        result.file_path.file_name().unwrap().to_str().unwrap(),
        "hello-world.md"
    );
    let file_content = fs::read_to_string(&result.file_path).unwrap();
    assert!(file_content.contains("# Hello World"));
    assert!(file_content.contains("date:"));
}

#[test]
fn test_create_content_page() {
    let temp = TempDir::new().unwrap();
    let input = temp.path().join("site");
    let content = input.join("content");
    fs::create_dir_all(&content).unwrap();
    fs::write(input.join("marmite.yaml"), "name: Test\ntagline: t").unwrap();

    let params = CreateContentParams {
        title: "About Me".to_string(),
        tags: None,
        directory: None,
        page: true,
        lang: None,
        translates: None,
    };
    let result = create_content(&input, &input.join("marmite.yaml"), &params).unwrap();

    assert_eq!(result.slug, "about-me");
    assert!(result.is_page);
    assert!(result.date.is_none());
    assert!(result.file_path.file_name().unwrap().to_str().unwrap() == "about-me.md");
}

#[test]
fn test_create_content_with_tags() {
    let temp = TempDir::new().unwrap();
    let input = temp.path().join("site");
    let content = input.join("content");
    fs::create_dir_all(&content).unwrap();
    fs::write(input.join("marmite.yaml"), "name: Test\ntagline: t").unwrap();

    let params = CreateContentParams {
        title: "Tagged Post".to_string(),
        tags: Some("rust, web".to_string()),
        directory: None,
        page: false,
        lang: None,
        translates: None,
    };
    let result = create_content(&input, &input.join("marmite.yaml"), &params).unwrap();

    let file_content = fs::read_to_string(&result.file_path).unwrap();
    assert!(file_content.contains("tags: rust, web"));
    assert!(file_content.contains("date:"));
}

#[test]
fn test_create_content_duplicate_post_fails() {
    let temp = TempDir::new().unwrap();
    let input = temp.path().join("site");
    let content = input.join("content");
    fs::create_dir_all(&content).unwrap();
    fs::write(input.join("marmite.yaml"), "name: Test\ntagline: t").unwrap();

    let params = CreateContentParams {
        title: "Unique Post".to_string(),
        tags: None,
        directory: None,
        page: false,
        lang: None,
        translates: None,
    };
    create_content(&input, &input.join("marmite.yaml"), &params).unwrap();
    let result = create_content(&input, &input.join("marmite.yaml"), &params);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("already exists"),
        "Expected 'already exists' error, got: {err}"
    );
}

#[test]
fn test_create_content_duplicate_page_fails() {
    let temp = TempDir::new().unwrap();
    let input = temp.path().join("site");
    let content = input.join("content");
    fs::create_dir_all(&content).unwrap();
    fs::write(input.join("marmite.yaml"), "name: Test\ntagline: t").unwrap();

    let params = CreateContentParams {
        title: "Unique Page".to_string(),
        tags: None,
        directory: None,
        page: true,
        lang: None,
        translates: None,
    };
    create_content(&input, &input.join("marmite.yaml"), &params).unwrap();
    let result = create_content(&input, &input.join("marmite.yaml"), &params);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("already exists"),
        "Expected 'already exists' error, got: {err}"
    );
}

#[test]
fn test_create_content_invalid_lang() {
    let temp = TempDir::new().unwrap();
    let input = temp.path().join("site");
    let content = input.join("content");
    fs::create_dir_all(&content).unwrap();
    fs::write(input.join("marmite.yaml"), "name: Test\ntagline: t").unwrap();

    let params = CreateContentParams {
        title: "Bad Lang".to_string(),
        tags: None,
        directory: None,
        page: false,
        lang: Some("xyz".to_string()),
        translates: None,
    };
    let result = create_content(&input, &input.join("marmite.yaml"), &params);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid language code"));
}

#[test]
fn test_update_frontmatter_add_fields() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.md");
    fs::write(
        &file_path,
        "---\ntitle: Original\n---\n# Content\n\nBody text.",
    )
    .unwrap();

    let mut updates = serde_json::Map::new();
    updates.insert("tags".into(), serde_json::json!("rust, web"));
    updates.insert("pinned".into(), serde_json::json!(true));

    let result = update_frontmatter(&file_path, &updates).unwrap();

    assert_eq!(result.get("tags").unwrap().as_str().unwrap(), "rust, web");
    assert!(result.get("pinned").unwrap().as_bool().unwrap());
    assert_eq!(result.get("title").unwrap().as_str().unwrap(), "Original");

    let written = fs::read_to_string(&file_path).unwrap();
    assert!(written.starts_with("---\n"));
    assert!(written.contains("# Content"));
    assert!(written.contains("Body text."));
}

#[test]
fn test_update_frontmatter_remove_field() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.md");
    fs::write(&file_path, "---\ntitle: Hello\ntags: old\n---\n# Hello\n").unwrap();

    let mut updates = serde_json::Map::new();
    updates.insert("tags".into(), serde_json::Value::Null);

    let result = update_frontmatter(&file_path, &updates).unwrap();

    assert!(result.get("tags").is_none());
    assert_eq!(result.get("title").unwrap().as_str().unwrap(), "Hello");
}

#[test]
fn test_update_frontmatter_no_existing_frontmatter() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.md");
    fs::write(&file_path, "# Just a title\n\nSome content.").unwrap();

    let mut updates = serde_json::Map::new();
    updates.insert("tags".into(), serde_json::json!("new-tag"));

    let result = update_frontmatter(&file_path, &updates).unwrap();

    assert_eq!(result.get("tags").unwrap().as_str().unwrap(), "new-tag");

    let written = fs::read_to_string(&file_path).unwrap();
    assert!(written.starts_with("---\n"));
    assert!(written.contains("# Just a title"));
}

#[test]
fn test_get_raw_content_success() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(
        content_dir.join("hello-world.md"),
        "---\ntitle: Hello World\ntags: rust, web\n---\nThis is the body.\n\nWith multiple paragraphs.",
    )
    .unwrap();

    let (fm, body, _path, fm_lines) = get_raw_content(&content_dir, "hello-world").unwrap();
    assert_eq!(fm.get("title").unwrap().as_str().unwrap(), "Hello World");
    assert!(body.contains("This is the body."));
    assert!(body.contains("With multiple paragraphs."));
    assert_eq!(fm_lines, 4);
}

#[test]
fn test_get_raw_content_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    let result = get_raw_content(&content_dir, "nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_update_content_body_success() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(
        content_dir.join("hello-world.md"),
        "---\ntitle: Hello World\n---\nOriginal body.",
    )
    .unwrap();

    let result = update_content_body(&content_dir, "hello-world", "New body content.\n", None);
    assert!(result.is_ok());

    let written = fs::read_to_string(content_dir.join("hello-world.md")).unwrap();
    assert!(written.contains("New body content."));
    assert!(!written.contains("Original body."));
    assert!(written.contains("title: Hello World"));
}

#[test]
fn test_update_content_body_with_frontmatter_updates() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(
        content_dir.join("hello-world.md"),
        "---\ntitle: Hello World\n---\nOriginal body.",
    )
    .unwrap();

    let mut updates = serde_json::Map::new();
    updates.insert("title".into(), serde_json::json!("Updated Title"));
    updates.insert("tags".into(), serde_json::json!("rust, web"));

    let result = update_content_body(
        &content_dir,
        "hello-world",
        "Updated body.\n",
        Some(&updates),
    );
    assert!(result.is_ok());

    let fm = result.unwrap();
    assert_eq!(fm.get("title").unwrap().as_str().unwrap(), "Updated Title");

    let written = fs::read_to_string(content_dir.join("hello-world.md")).unwrap();
    assert!(written.contains("Updated body."));
    assert!(written.contains("Updated Title"));
}

#[test]
fn test_update_content_body_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    let result = update_content_body(&content_dir, "nonexistent", "body", None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_json_to_fm_value_conversions() {
    assert!(json_to_fm_value(&serde_json::Value::Null).is_none());

    let s = json_to_fm_value(&serde_json::json!("hello")).unwrap();
    assert!(matches!(s, frontmatter_gen::Value::String(ref v) if v == "hello"));

    let b = json_to_fm_value(&serde_json::json!(true)).unwrap();
    assert!(matches!(b, frontmatter_gen::Value::Boolean(true)));

    let n = json_to_fm_value(&serde_json::json!(42.0)).unwrap();
    if let frontmatter_gen::Value::Number(v) = n {
        assert!((v - 42.0).abs() < f64::EPSILON);
    } else {
        panic!("Expected Number");
    }

    let arr = json_to_fm_value(&serde_json::json!(["a", "b"])).unwrap();
    if let frontmatter_gen::Value::Array(items) = arr {
        assert_eq!(items.len(), 2);
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_fm_value_to_json_roundtrip() {
    let original = serde_json::json!({
        "title": "Test",
        "tags": ["a", "b"],
        "pinned": true,
        "count": 5.0
    });
    let fm_val = json_to_fm_value(&original).unwrap();
    let back = fm_value_to_json(&fm_val);

    assert_eq!(back.get("title").unwrap().as_str().unwrap(), "Test");
    assert_eq!(back.get("tags").unwrap().as_array().unwrap().len(), 2);
    assert!(back.get("pinned").unwrap().as_bool().unwrap());
}

#[test]
fn test_delete_content_success() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(
        content_dir.join("hello-world.md"),
        "---\ntitle: Hello World\n---\nHello",
    )
    .unwrap();

    let result = delete_content(&content_dir, "hello-world");
    assert!(result.is_ok());
    assert!(!content_dir.join("hello-world.md").exists());
}

#[test]
fn test_delete_content_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    let result = delete_content(&content_dir, "nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_move_content_rename() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(
        content_dir.join("old-name.md"),
        "---\ntitle: Old Name\n---\nContent",
    )
    .unwrap();

    let result = move_content(&content_dir, "old-name", "new-name.md");
    assert!(result.is_ok());
    let (old_path, new_path) = result.unwrap();
    assert!(!old_path.exists());
    assert!(new_path.exists());
    assert!(new_path.ends_with("new-name.md"));
}

#[test]
fn test_move_content_to_subdirectory() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(
        content_dir.join("hello-world.md"),
        "---\ntitle: Hello World\n---\nContent",
    )
    .unwrap();

    let result = move_content(&content_dir, "hello-world", "posts/hello-world.md");
    assert!(result.is_ok());
    let (old_path, new_path) = result.unwrap();
    assert!(!old_path.exists());
    assert!(new_path.exists());
    assert!(content_dir.join("posts").join("hello-world.md").exists());
}

#[test]
fn test_move_content_target_exists() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(content_dir.join("first.md"), "---\ntitle: First\n---\n").unwrap();
    fs::write(content_dir.join("second.md"), "---\ntitle: Second\n---\n").unwrap();

    let result = move_content(&content_dir, "first", "second.md");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn test_move_content_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    let result = move_content(&content_dir, "nonexistent", "new.md");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_move_content_requires_md_extension() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(
        content_dir.join("test-page.md"),
        "---\ntitle: Test Page\n---\n",
    )
    .unwrap();

    let result = move_content(&content_dir, "test-page", "test-page.txt");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains(".md"));
}

#[test]
fn test_create_translation_flat_directory() {
    // posts/hello.md -> translation goes to posts/ola.md with explicit frontmatter
    let temp = TempDir::new().unwrap();
    let input = temp.path().join("site");
    let content = input.join("content");
    let posts = content.join("posts");
    fs::create_dir_all(&posts).unwrap();
    fs::write(input.join("marmite.yaml"), "name: Test\ntagline: t").unwrap();
    fs::write(
        posts.join("hello.md"),
        "---\ntitle: Hello\ndate: 2024-01-01\n---\nHello!",
    )
    .unwrap();

    let params = CreateContentParams {
        title: "Ola".to_string(),
        tags: None,
        directory: None,
        page: false,
        lang: Some("pt".to_string()),
        translates: Some("hello".to_string()),
    };
    let result = create_content(&input, &input.join("marmite.yaml"), &params).unwrap();

    // Translation goes in the same directory as the original
    assert_eq!(
        result.file_path.parent().unwrap().file_name().unwrap(),
        "posts"
    );
    assert_eq!(
        result.file_path.file_name().unwrap().to_str().unwrap(),
        "ola.md"
    );
    // Frontmatter must have explicit title, language, and translates
    let content = fs::read_to_string(&result.file_path).unwrap();
    assert!(content.contains("title:"), "must have explicit title");
    assert!(content.contains("language: pt"), "must have language");
    assert!(
        content.contains("translates: hello"),
        "must have translates"
    );
}

#[test]
fn test_create_translation_slug_subfolder() {
    // posts/hello/hello.md -> translation goes to posts/hello/pt-ola.md (lang prefix)
    let temp = TempDir::new().unwrap();
    let input = temp.path().join("site");
    let content = input.join("content");
    let subfolder = content.join("posts").join("hello");
    fs::create_dir_all(&subfolder).unwrap();
    fs::write(input.join("marmite.yaml"), "name: Test\ntagline: t").unwrap();
    fs::write(
        subfolder.join("hello.md"),
        "---\ntitle: Hello\ndate: 2024-01-01\n---\nHello!",
    )
    .unwrap();

    let params = CreateContentParams {
        title: "Ola".to_string(),
        tags: None,
        directory: None,
        page: false,
        lang: Some("pt".to_string()),
        translates: Some("hello".to_string()),
    };
    let result = create_content(&input, &input.join("marmite.yaml"), &params).unwrap();

    // Translation goes in the slug-named subfolder with lang prefix
    assert_eq!(
        result.file_path.parent().unwrap().file_name().unwrap(),
        "hello"
    );
    assert_eq!(
        result.file_path.file_name().unwrap().to_str().unwrap(),
        "pt-ola.md"
    );
    // No translates field needed (auto-discovered from folder)
    let content = fs::read_to_string(&result.file_path).unwrap();
    assert!(
        !content.contains("translates:"),
        "subfolder translations should not have translates field"
    );
    assert!(content.contains("language: pt"), "must have language");
}

#[test]
fn test_clone_content_copies_everything() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(
        content_dir.join("original.md"),
        "---\ntitle: Original\ndate: 2024-01-01\ntags: rust, web\nstream: tutorial\nlanguage: en\n---\n\nThis is the full body.\n\n## Section\n\nMore content here.\n",
    )
    .unwrap();

    let (dest, slug) = clone_content(&content_dir, "original", "Copy of Original", None).unwrap();

    assert_eq!(slug, "copy-of-original");
    assert!(dest.exists());
    let cloned = fs::read_to_string(&dest).unwrap();
    // Title and slug are updated
    assert!(cloned.contains("Copy of Original"));
    assert!(cloned.contains("copy-of-original"));
    // Markdown body is preserved
    assert!(cloned.contains("This is the full body."));
    assert!(cloned.contains("## Section"));
    assert!(cloned.contains("More content here."));
    // Other frontmatter is preserved
    assert!(cloned.contains("tags:"));
    assert!(cloned.contains("stream: tutorial"));
    assert!(cloned.contains("language: en"));
}

#[test]
fn test_clone_content_custom_slug() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(
        content_dir.join("original.md"),
        "---\ntitle: Original\n---\nBody",
    )
    .unwrap();

    let (_, slug) =
        clone_content(&content_dir, "original", "New Title", Some("custom-slug")).unwrap();
    assert_eq!(slug, "custom-slug");
}

#[test]
fn test_clone_content_strips_aliases_and_translates() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(
        content_dir.join("original.md"),
        "---\ntitle: Original\naliases: old-url\ntranslates: some-post\n---\nBody",
    )
    .unwrap();

    let (dest, _) = clone_content(&content_dir, "original", "Clone", None).unwrap();
    let cloned = fs::read_to_string(&dest).unwrap();
    assert!(!cloned.contains("aliases:"), "aliases should be removed");
    assert!(
        !cloned.contains("translates:"),
        "translates should be removed"
    );
}

#[test]
fn test_clone_content_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    let result = clone_content(&content_dir, "nonexistent", "Title", None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_clone_content_duplicate_slug() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(
        content_dir.join("original.md"),
        "---\ntitle: Original\n---\nBody",
    )
    .unwrap();
    fs::write(
        content_dir.join("existing.md"),
        "---\ntitle: Existing\n---\nBody",
    )
    .unwrap();

    let result = clone_content(&content_dir, "original", "Existing", None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}
