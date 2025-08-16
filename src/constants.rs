// Constants used throughout the Marmite static site generator

// File extensions and patterns
pub const MARKDOWN_EXTENSION: &str = "md";
pub const HTML_EXTENSION: &str = "html";
pub const YAML_EXTENSION: &str = "yaml";
pub const YML_EXTENSION: &str = "yml";

// Directory names
pub const CONTENT_DIR: &str = "content";
pub const TEMPLATES_DIR: &str = "templates";
pub const STATIC_DIR: &str = "static";
pub const MEDIA_DIR: &str = "media";

// File names
pub const CONFIG_FILE: &str = "marmite.yaml";
pub const INDEX_FILE: &str = "index.html";
pub const ROBOTS_FILE: &str = "robots.txt";
pub const FEED_FILE: &str = "feed.xml";
pub const ATOM_FILE: &str = "atom.xml";

// Template names
pub const BASE_TEMPLATE: &str = "base.html";
pub const LIST_TEMPLATE: &str = "list.html";
pub const POST_TEMPLATE: &str = "post.html";
pub const PAGE_TEMPLATE: &str = "page.html";
pub const TAG_TEMPLATE: &str = "tag.html";
pub const STREAM_TEMPLATE: &str = "stream.html";
pub const AUTHOR_TEMPLATE: &str = "author.html";
pub const CARD_TEMPLATE: &str = "card.html";

// Regex patterns
pub const INTERNAL_LINK_PATTERN: &str = r"\[\[([^\]]+)\]\]";
pub const SHORTCODE_PATTERN: &str = r"\{\{%\s*(\w+)\s*(.*?)\s*%\}\}";
pub const SHORTCODE_MACRO_PATTERN: &str = r"\{%\s*macro\s+(\w+)\s*\(";
pub const DATE_PREFIX_PATTERN: &str = r"^\d{4}-\d{2}-\d{2}([-T]\d{2}([:-]\d{2})?([:-]\d{2})?)?-";
pub const DATE_PATTERN: &str = r"^\d{4}-\d{2}-\d{2}( \d{2}:\d{2}(:\d{2})?)?";
pub const STREAM_DATE_PATTERN: &str =
    r"^(\d{4})-(\d{2})-(\d{2})([-T]\d{2}([:-]\d{2})?([:-]\d{2})?)?-";
pub const STREAM_S_PATTERN: &str = r"^[a-zA-Z0-9]+-S-(.+)$";
pub const STREAM_PREFIX_PATTERN: &str = r"^([a-zA-Z0-9]+)-S-";
pub const STREAM_NAME_DATE_PATTERN: &str = r"^([a-zA-Z0-9]+)-(\d{4}-\d{2}-\d{2})";
pub const SLUGIFY_PATTERN: &str = r"[^a-z0-9]+";
pub const IMAGE_SRC_PATTERN: &str = r#"<img[^>]*src="([^"]+)""#;
pub const LINK_HREF_PATTERN: &str = r#"href="([^"]+)\.html(#[^"]+)?""#;
pub const HEADING_PATTERN: &str =
    r#"<h([1-6])[^>]*>(?:<a[^>]*href="([^"]+)"[^>]*></a>)?(.*?)</h[1-6]>"#;
pub const ANCHOR_PATTERN: &str = r#"<a[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#;
pub const HTML_TAG_STRIP_PATTERN: &str = r"<[^>]*>|(\{\{[^>]*\}\})|(\{%[^>]*%\})";

// Default values
pub const DEFAULT_PORT: u16 = 8000;
pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:8000";
pub const DEFAULT_DATE_FORMAT: &str = "%Y-%m-%d %H:%M";
pub const DEFAULT_OUTPUT_DIR: &str = "site";
pub const DEFAULT_PAGINATION_SIZE: usize = 10;
pub const DEFAULT_EXCERPT_LENGTH: usize = 150;
pub const DEFAULT_CONFIG_FILE: &str = "marmite.yaml";

// Content types
pub const CONTENT_TYPE_POST: &str = "post";
pub const CONTENT_TYPE_PAGE: &str = "page";

// Server constants
pub const WATCH_INTERVAL_MS: u64 = 1000;
pub const SERVER_THREAD_POOL_SIZE: usize = 4;

// Feed constants
pub const FEED_MAX_ITEMS: usize = 50;
pub const FEED_MIME_TYPE: &str = "application/rss+xml";
pub const ATOM_MIME_TYPE: &str = "application/atom+xml";

// Template variable names
pub const VAR_SITE: &str = "site";
pub const VAR_DATA: &str = "data";
pub const VAR_CONTENT: &str = "content";
pub const VAR_PAGE: &str = "page";
pub const VAR_PAGINATOR: &str = "paginator";

// CSS classes and IDs
pub const ACTIVE_CLASS: &str = "active";
pub const HIGHLIGHT_CLASS: &str = "highlight";

// Embedded asset paths
pub const EMBEDDED_CSS_PATH: &str = "css/style.css";
pub const EMBEDDED_JS_PATH: &str = "js/main.js";

// Mandatory templates array
pub const MANDATORY_TEMPLATES: &[&str] = &["base.html", "list.html", "group.html", "content.html"];

// Logging levels
pub const LOG_LEVEL_WARN: &str = "marmite=warn";
pub const LOG_LEVEL_INFO: &str = "marmite=info";
pub const LOG_LEVEL_DEBUG: &str = "marmite=debug";
pub const LOG_LEVEL_TRACE: &str = "marmite=trace";
pub const LOG_LEVEL_ALL_TRACE: &str = "trace";

// Error messages
pub const ERROR_CONFIG_NOT_FOUND: &str = "Configuration file not found";
pub const ERROR_TEMPLATE_NOT_FOUND: &str = "Template file not found";
pub const ERROR_CONTENT_DIR_NOT_FOUND: &str = "Content directory not found";

// Success messages
pub const SUCCESS_SITE_GENERATED: &str = "Site generated successfully";
pub const SUCCESS_SERVER_STARTED: &str = "Server started at";
