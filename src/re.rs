//! Centralized regex patterns used across the codebase

// === HTML and Template Patterns ===

/// Matches HTML tags or template expressions ({{...}} or {%...%})
/// Used for removing HTML/template syntax from text
pub const HTML_OR_TEMPLATE_TAGS: &str = r"<[^>]*>|(\{\{[^>]*\}\})|(\{%[^>]*%\})";

/// Matches basic HTML tags
/// Used for stripping HTML from text content
pub const HTML_TAGS: &str = r"<[^>]*>";

/// Matches href attributes in HTML that point to .html files with optional anchors
/// Captures: 1) the file path without .html, 2) optional anchor (#section)
/// Used for converting markdown links to HTML links
pub const HREF_HTML_LINKS: &str = r#"href="([^"]+)\.html(#[^"]+)?""#;

/// Matches HTML heading tags (h1-h6) with optional anchor links inside
/// Captures: 1) heading level, 2) optional anchor href, 3) heading content
/// Used for extracting table of contents from HTML
pub const HTML_HEADINGS_WITH_ANCHORS: &str =
    r#"<h([1-6])[^>]*>(?:<a[^>]*href="([^"]+)"[^>]*></a>)?(.*?)</h[1-6]>"#;

/// Matches anchor tags in HTML
/// Captures: 1) href attribute value, 2) link text
/// Used for extracting links from HTML content
pub const HTML_ANCHOR_TAGS: &str = r#"<a[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#;

/// Matches img tags to extract src attributes
/// Captures: 1) the src attribute value
/// Used for extracting image URLs from HTML content
pub const HTML_IMG_SRC: &str = r#"<img[^>]*src="([^"]+)""#;

// === Shortcode Patterns ===

/// Default pattern for HTML comment-style shortcodes
/// Matches: <!-- .name params -->
/// Used as the default shortcode pattern when none is specified
pub const SHORTCODE_HTML_COMMENT: &str = r"<!-- \.(\w+)(\s+[^>]+)?\s*-->";

/// Matches Tera macro calls in templates
/// Captures: 1) macro name
/// Used for detecting macro usage in templates
pub const TERA_MACRO_CALL: &str = r"\{%\s*macro\s+(\w+)\s*\(";

// === Date and Filename Patterns ===

/// Matches date prefix in filenames (YYYY-MM-DD with optional time)
/// Format: YYYY-MM-DD[-THH[:MM[:SS]]]-
/// Used for extracting dates from content filenames
pub const DATE_PREFIX_FILENAME: &str = r"^\d{4}-\d{2}-\d{2}([-T]\d{2}([:-]\d{2})?([:-]\d{2})?)?-";

/// Matches stream-style date in filenames
/// Format: stream-YYYY-MM-DD[-THH[:MM[:SS]]]-title
/// Used for extracting dates from stream content filenames
pub const STREAM_DATE_FILENAME: &str =
    r"^[a-zA-Z0-9]+-\d{4}-\d{2}-\d{2}(?:[-T]\d{2}(?:[:-]\d{2})?(?:[:-]\d{2})?)?-(.+)$";

/// Matches stream "S" pattern in filenames
/// Format: stream-S-title (where S indicates a stream without date)
/// Used for identifying stream content without specific dates
pub const STREAM_S_PATTERN: &str = r"^[a-zA-Z0-9]+-S-(.+)$";

/// Matches stream prefix with date extraction
/// Captures: 1) stream name, 2) date part
/// Used for extracting stream name and date from filenames
pub const STREAM_WITH_DATE: &str = r"^([a-zA-Z0-9]+)-(\d{4}-\d{2}-\d{2})";

/// Matches stream prefix without date (S pattern)
/// Captures: 1) stream name
/// Used for identifying stream content files
pub const STREAM_PREFIX: &str = r"^([a-zA-Z0-9]+)-S-";

/// Matches date at the beginning of a string (for parsing dates from text)
/// Format: YYYY-MM-DD[ HH:MM[:SS]]
/// Used for parsing dates from content metadata
pub const DATE_PREFIX_TEXT: &str = r"^\d{4}-\d{2}-\d{2}( \d{2}:\d{2}(:\d{2})?)?";

/// Matches stream date pattern in directory names
/// Format: stream-YYYY-MM-DD[-THH[:MM[:SS]]]
/// Used for extracting dates from stream directory names
pub const STREAM_DATE_DIRECTORY: &str =
    r"^[a-zA-Z0-9]+-(\d{4}-\d{2}-\d{2}(?:[-T]\d{2}(?:[:-]\d{2})?(?:[:-]\d{2})?)?)";

// === Text Processing Patterns ===

/// Matches non-alphanumeric characters (excluding hyphens between words)
/// Used for slugifying text (converting to URL-friendly format)
pub const SLUGIFY_CHARS: &str = r"[^a-z0-9]+";
