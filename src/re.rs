//! Centralized regex patterns used across the codebase

// === HTML and Template Patterns ===

/// Matches HTML tags or template expressions ({{...}} or {%...%})
/// Used for removing HTML/template syntax from text
pub const MATCH_HTML_OR_TEMPLATE_TAGS: &str = r"<[^>]*>|\{\{[^}]*\}\}|\{%[^%]*%\}";

/// Matches basic HTML tags
/// Used for stripping HTML from text content
pub const MATCH_HTML_TAGS: &str = r"<[^>]*>";

/// Matches href attributes in HTML that point to .html files with optional anchors
/// Captures: 1) the file path without .html, 2) optional anchor (#section)
/// Used for converting markdown links to HTML links
pub const CAPTURE_SLUG_ANCHOR_FROM_HREF: &str = r#"href=['\"]([^'\"]+)\.html(#[^'\"]+)?['\"]"#;

/// Matches HTML heading tags (h1-h6) with optional anchor links inside
/// Captures: 1) heading level, 2) optional anchor href, 3) heading content
/// Used for extracting table of contents from HTML
pub const CAPTURE_LEVEL_ANCHOR_TEXT_FROM_H_TAG: &str =
    r#"<h([1-6])[^>]*>(?:<a[^>]*href=['\"]([^'\"]+)['\"][^>]*></a>)?(.*?)</h[1-6]>"#;

/// Matches anchor tags in HTML
/// Captures: 1) href attribute value, 2) link text
/// Used for extracting links from HTML content
pub const CAPTURE_LINK_AND_TEXT_FROM_A_TAG: &str =
    r#"<a[^>]*href=['\"]([^'\"]+)['\"][^>]*>(.*?)</a>"#;

/// Matches img tags to extract src attributes
/// Captures: 1) the src attribute value
/// Used for extracting image URLs from HTML content
pub const CAPTURE_SRC_FROM_IMG_HTMLTAG: &str = r#"<img[^>]*src=['\"]([^'\"]+)['\"]"#;

/// Matches wikilink anchor tags with data-wikilink attribute
/// Captures: 1) href attribute value, 2) link text content
/// Used for fixing Obsidian wikilinks to use proper slugs instead of filename-based hrefs
pub const CAPTURE_WIKILINK_HREF_AND_TITLE: &str =
    r#"<a[^>]*href=['\"]([^'\"]+)['\"][^>]*data-wikilink=['\"]true['\"][^>]*>(.*?)</a>"#;

// === Shortcode Patterns ===

/// Default pattern for HTML comment-style shortcodes
/// Matches: <!-- .name params -->
/// Used as the default shortcode pattern when none is specified
pub const SHORTCODE_HTML_COMMENT: &str = r"<!-- \.(\w+)(?:\s+([^-][\s\S]*?))?\s*-->";

/// Matches Tera macro calls in templates
/// Captures: 1) macro name
/// Used for detecting macro usage in templates
pub const CAPTURE_TERA_MACRO_CALL: &str = r"\{%\s*macro\s+(\w+)\s*\(";

// === Date and Filename Patterns ===

/// Matches date prefix in filenames (YYYY-MM-DD with optional time)
/// Format: YYYY-MM-DD[-THH[:MM[:SS]]]-
/// Used for extracting date prefix from content filenames
/// `2024-01-01-myfile.md` -> `2024-01-01-`
pub const MATCH_DATE_PREFIX_FROM_FILENAME: &str =
    r"^\d{4}-\d{2}-\d{2}([-T]\d{2}([:-]\d{2})?([:-]\d{2})?)?-";

/// Matches stream-style date in filenames
/// Format: stream-YYYY-MM-DD[-THH[:MM[:SS]]]-title
/// Used for extracting stream-date prefix from stream content filenames
/// `stream-2024-01-01-myfile.md` -> `myfile` and `stream-S-myfile.md` -> `myfile`
/// Captures the slug (text after the date/time).
pub const CAPTURE_SLUG_FROM_STREAM_DATED_FILENAME: &str =
    r"^[a-zA-Z0-9]+-\d{4}-\d{2}-\d{2}(?:[-T]\d{2}(?:[:-]\d{2})?(?:[:-]\d{2})?)?-(.+)$";

/// Matches stream date pattern in filenames with slug
/// Format: stream-YYYY-MM-DD[-THH[:MM[:SS]]]
/// Used for extracting dates from stream named files
/// `news-2024-01-15-site-update.md` -> `2025-08-16`
/// Captures the date from stream-date-slug pattern
pub const CAPTURE_DATE_FROM_STREAM_DATED_FILENAME: &str =
    r"^[a-zA-Z0-9]+-(\d{4}-\d{2}-\d{2}(?:[-T]\d{2}(?:[:-]\d{2})?(?:[:-]\d{2})?)?)";

/// Matches stream "S" pattern in filenames
/// Format: stream-S-title (where S indicates a stream without date)
/// Used for identifying stream content without specific dates
/// stream-S-slug -> slug
pub const CAPTURE_SLUG_FROM_STREAM_S_FILENAME: &str = r"^[a-zA-Z0-9]+-S-(.+)$";

/// Matches stream prefix with date extraction
/// Captures: 1) stream name, 2) date part
/// Used for extracting stream name and date from filenames
/// Extract stream from filename pattern: {stream}-{date}-{slug}
/// Only accepts single word before date (no hyphens allowed in stream name)
pub const CAPTURE_STREAM_AND_DATE_FROM_FILENAME: &str = r"^([a-zA-Z0-9]+)-(\d{4}-\d{2}-\d{2})";

/// Matches stream prefix without date (S pattern)
/// Captures: 1) stream name
/// Used for identifying stream content files
/// Extract stream from filename pattern: {stream}-S-{slug}
/// Only accepts single word before 'S' marker
pub const CAPTURE_STREAM_FROM_S_FILENAME: &str = r"^([a-zA-Z0-9]+)-S-";

/// Matches date at the beginning of a string (for parsing dates from text)
/// Format: YYYY-MM-DD[ HH:MM[:SS]]
/// Used for parsing dates from content metadata
pub const CAPTURE_DATE_PREFIX_FROM_TEXT: &str = r"^\d{4}-\d{2}-\d{2}( \d{2}:\d{2}(:\d{2})?)?";

// === Text Processing Patterns ===
