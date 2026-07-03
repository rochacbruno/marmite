use regex::Regex;

use crate::re::*;

#[test]
fn test_all_patterns_compile() {
    Regex::new(MATCH_HTML_OR_TEMPLATE_TAGS).unwrap();
    Regex::new(MATCH_HTML_TAGS).unwrap();
    Regex::new(CAPTURE_SLUG_ANCHOR_FROM_HREF).unwrap();
    Regex::new(CAPTURE_LEVEL_ANCHOR_TEXT_FROM_H_TAG).unwrap();
    Regex::new(CAPTURE_LINK_AND_TEXT_FROM_A_TAG).unwrap();
    Regex::new(CAPTURE_SRC_FROM_IMG_HTMLTAG).unwrap();
    Regex::new(CAPTURE_WIKILINK_HREF_AND_TITLE).unwrap();
    Regex::new(SHORTCODE_HTML_COMMENT).unwrap();
    Regex::new(CAPTURE_SHORTCODE_DEF).unwrap();
    Regex::new(MATCH_DATE_PREFIX_FROM_FILENAME).unwrap();
    Regex::new(CAPTURE_SLUG_FROM_STREAM_DATED_FILENAME).unwrap();
    Regex::new(CAPTURE_DATE_FROM_STREAM_DATED_FILENAME).unwrap();
    Regex::new(CAPTURE_SLUG_FROM_STREAM_S_FILENAME).unwrap();
    Regex::new(CAPTURE_STREAM_AND_DATE_FROM_FILENAME).unwrap();
    Regex::new(CAPTURE_STREAM_FROM_S_FILENAME).unwrap();
    Regex::new(CAPTURE_DATE_PREFIX_FROM_TEXT).unwrap();
    Regex::new(REPLACE_AT_MEDIA_REF_IN_HTML).unwrap();
}

#[test]
fn test_match_html_or_template_tags() {
    let re = Regex::new(MATCH_HTML_OR_TEMPLATE_TAGS).unwrap();
    assert!(re.is_match("<div>"));
    assert!(re.is_match("<br/>"));
    assert!(re.is_match("{{variable}}"));
    assert!(re.is_match("{% block content %}"));
    assert!(!re.is_match("plain text"));
}

#[test]
fn test_match_html_tags() {
    let re = Regex::new(MATCH_HTML_TAGS).unwrap();
    assert!(re.is_match("<p>"));
    assert!(re.is_match("<br/>"));
    assert!(re.is_match(r#"<a href="test">"#));
    assert!(!re.is_match("{{variable}}"));
    assert!(!re.is_match("plain text"));
}

#[test]
fn test_capture_slug_anchor_from_href() {
    let re = Regex::new(CAPTURE_SLUG_ANCHOR_FROM_HREF).unwrap();

    let caps = re.captures(r#"href="my-post.html#section""#).unwrap();
    assert_eq!(&caps[1], "my-post");
    assert_eq!(&caps[2], "#section");

    let caps = re.captures(r#"href="about.html""#).unwrap();
    assert_eq!(&caps[1], "about");
    assert!(caps.get(2).is_none());

    assert!(re.captures(r#"href="https://example.com""#).is_none());
}

#[test]
fn test_capture_level_anchor_text_from_h_tag() {
    let re = Regex::new(CAPTURE_LEVEL_ANCHOR_TEXT_FROM_H_TAG).unwrap();

    let caps = re.captures(r#"<h2>My Heading</h2>"#).unwrap();
    assert_eq!(&caps[1], "2");
    assert_eq!(&caps[3], "My Heading");

    let caps = re
        .captures(r##"<h3><a href="#anchor"></a>With Anchor</h3>"##)
        .unwrap();
    assert_eq!(&caps[1], "3");
    assert_eq!(&caps[2], "#anchor");
    assert_eq!(&caps[3], "With Anchor");
}

#[test]
fn test_capture_link_and_text_from_a_tag() {
    let re = Regex::new(CAPTURE_LINK_AND_TEXT_FROM_A_TAG).unwrap();

    let caps = re
        .captures(r#"<a href="https://example.com">Example</a>"#)
        .unwrap();
    assert_eq!(&caps[1], "https://example.com");
    assert_eq!(&caps[2], "Example");
}

#[test]
fn test_capture_src_from_img_htmltag() {
    let re = Regex::new(CAPTURE_SRC_FROM_IMG_HTMLTAG).unwrap();

    let caps = re.captures(r#"<img src="photo.jpg" alt="Photo">"#).unwrap();
    assert_eq!(&caps[1], "photo.jpg");

    let caps = re.captures(r#"<img src='image.png'>"#).unwrap();
    assert_eq!(&caps[1], "image.png");
}

#[test]
fn test_capture_wikilink_href_and_title() {
    let re = Regex::new(CAPTURE_WIKILINK_HREF_AND_TITLE).unwrap();

    let caps = re
        .captures(r#"<a href="My Page" data-wikilink="true">Display Text</a>"#)
        .unwrap();
    assert_eq!(&caps[1], "My Page");
    assert_eq!(&caps[2], "Display Text");

    assert!(re
        .captures(r#"<a href="not-a-wikilink">Text</a>"#)
        .is_none());
}

#[test]
fn test_shortcode_html_comment() {
    let re = Regex::new(SHORTCODE_HTML_COMMENT).unwrap();

    let caps = re.captures("<!-- .youtube id=abc123 -->").unwrap();
    assert_eq!(&caps[1], "youtube");
    assert_eq!(&caps[2], "id=abc123");

    let caps = re.captures("<!-- .divider -->").unwrap();
    assert_eq!(&caps[1], "divider");
    assert!(caps.get(2).is_none());
}

#[test]
fn test_capture_shortcode_def() {
    let re = Regex::new(CAPTURE_SHORTCODE_DEF).unwrap();

    let caps = re.captures("{% macro youtube(id) %}").unwrap();
    assert_eq!(&caps[1], "youtube");

    let caps = re.captures("{% shortcode gallery(images) %}").unwrap();
    assert_eq!(&caps[1], "gallery");
}

#[test]
fn test_match_date_prefix_from_filename() {
    let re = Regex::new(MATCH_DATE_PREFIX_FROM_FILENAME).unwrap();

    assert!(re.is_match("2024-01-01-my-post"));
    assert!(re.is_match("2024-01-01-15-30-my-post"));
    assert!(re.is_match("2024-01-01T15-30-00-my-post"));
    assert!(!re.is_match("my-post"));
    assert!(!re.is_match("not-a-date-prefix"));
}

#[test]
fn test_capture_slug_from_stream_dated_filename() {
    let re = Regex::new(CAPTURE_SLUG_FROM_STREAM_DATED_FILENAME).unwrap();

    let caps = re.captures("news-2024-01-15-site-update").unwrap();
    assert_eq!(&caps[1], "site-update");

    let caps = re.captures("blog-2024-06-01T10-30-00-my-post").unwrap();
    assert_eq!(&caps[1], "my-post");
}

#[test]
fn test_capture_date_from_stream_dated_filename() {
    let re = Regex::new(CAPTURE_DATE_FROM_STREAM_DATED_FILENAME).unwrap();

    let caps = re.captures("news-2024-01-15-site-update").unwrap();
    assert_eq!(&caps[1], "2024-01-15");

    let caps = re.captures("blog-2024-06-01T10-30-site-update").unwrap();
    assert_eq!(&caps[1], "2024-06-01T10-30");
}

#[test]
fn test_capture_slug_from_stream_s_filename() {
    let re = Regex::new(CAPTURE_SLUG_FROM_STREAM_S_FILENAME).unwrap();

    let caps = re.captures("news-S-about-page").unwrap();
    assert_eq!(&caps[1], "about-page");

    assert!(re.captures("news-2024-01-01-post").is_none());
}

#[test]
fn test_capture_stream_and_date_from_filename() {
    let re = Regex::new(CAPTURE_STREAM_AND_DATE_FROM_FILENAME).unwrap();

    let caps = re.captures("news-2024-01-15-slug").unwrap();
    assert_eq!(&caps[1], "news");
    assert_eq!(&caps[2], "2024-01-15");
}

#[test]
fn test_capture_stream_from_s_filename() {
    let re = Regex::new(CAPTURE_STREAM_FROM_S_FILENAME).unwrap();

    let caps = re.captures("docs-S-getting-started").unwrap();
    assert_eq!(&caps[1], "docs");

    assert!(re.captures("docs-2024-01-01-post").is_none());
}

#[test]
fn test_capture_date_prefix_from_text() {
    let re = Regex::new(CAPTURE_DATE_PREFIX_FROM_TEXT).unwrap();

    assert!(re.is_match("2024-01-15"));
    assert!(re.is_match("2024-01-15 10:30"));
    assert!(re.is_match("2024-01-15 10:30:45"));
    assert!(!re.is_match("not-a-date"));
}

#[test]
fn test_replace_at_media_ref_in_html() {
    let re = Regex::new(REPLACE_AT_MEDIA_REF_IN_HTML).unwrap();

    let caps = re.captures(r#"src="@/""#).unwrap();
    assert_eq!(&caps["attr"], "src");

    let caps = re.captures(r#"href="@/""#).unwrap();
    assert_eq!(&caps["attr"], "href");

    assert!(re.captures(r#"alt="@/""#).is_none());
}
