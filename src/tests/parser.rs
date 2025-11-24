use super::*;

#[test]
fn test_fix_internal_links_with_md_extension() {
    let html = r#"<a href="test.md">test.md</a>"#;
    let expected = r#"<a href="test.html">test</a>"#;
    assert_eq!(fix_internal_links(html), expected);
}

#[test]
fn test_fix_internal_links_with_html_extension() {
    let html = r#"<a href="test.html">test.html</a>"#;
    let expected = r#"<a href="test.html">test</a>"#;
    assert_eq!(fix_internal_links(html), expected);
}

#[test]
fn test_fix_internal_links_without_extension() {
    let html = r#"<a href="test">test</a>"#;
    let expected = r#"<a href="test.html">test</a>"#;
    assert_eq!(fix_internal_links(html), expected);
}

#[test]
fn test_fix_internal_links_external_link() {
    let html = r#"<a href="http://example.com">example</a>"#;
    let expected = r#"<a href="http://example.com">example</a>"#;
    assert_eq!(fix_internal_links(html), expected);
}

#[test]
fn test_fix_internal_links_mixed_content() {
    let html = r#"<a href="test.md">test.md</a> and <a href="http://example.com">example</a>"#;
    let expected = r#"<a href="test.html">test</a> and <a href="http://example.com">example</a>"#;
    assert_eq!(fix_internal_links(html), expected);
}

#[test]
fn test_get_links_to_with_internal_links() {
    let html = r#"<a href="./test1.html">test1</a> <a href="./test2.html">test2</a>"#;
    let expected = Some(vec!["test1".to_string(), "test2".to_string()]);
    assert_eq!(get_links_to(html), expected);
}

#[test]
fn test_get_links_to_with_internal_links_no_slash() {
    let html = r#"<a href="test1.html">test1</a> <a href="test2.html">test2</a>"#;
    let expected = Some(vec!["test1".to_string(), "test2".to_string()]);
    assert_eq!(get_links_to(html), expected);
}

#[test]
fn test_get_links_to_with_no_internal_links() {
    let html = r#"<a href="http://example.com">example</a>"#;
    let expected: Option<Vec<String>> = None;
    assert_eq!(get_links_to(html), expected);
}

#[test]
fn test_get_links_to_with_mixed_links() {
    let html = r#"<a href="./test1.html">test1</a> <a href="test2.html">test2</a> <a href="http://example.com">example</a>"#;
    let expected = Some(vec!["test1".to_string(), "test2".to_string()]);
    assert_eq!(get_links_to(html), expected);
}

#[test]
fn test_get_links_to_with_no_links() {
    let html = r"<p>No links here</p>";
    let expected: Option<Vec<String>> = None;
    assert_eq!(get_links_to(html), expected);
}

#[test]
fn test_get_links_to_with_empty_string() {
    let html = "";
    let expected: Option<Vec<String>> = None;
    assert_eq!(get_links_to(html), expected);
}

#[test]
fn test_get_links_to_with_internal_link_with_heading() {
    let html = r#"<a href="test1.html#heading">test</a>"#;
    let expected = Some(vec!["test1#heading".to_string()]);

    assert_eq!(get_links_to(html), expected);
}

#[test]
fn test_get_html_basic_markdown() {
    let markdown = "# Title\n\nThis is a paragraph.";
    let expected = "<h1><a href=\"#title\" aria-hidden=\"true\" class=\"anchor\" id=\"title\"></a>Title</h1>\n<p>This is a paragraph.</p>\n";
    assert_eq!(get_html(markdown), expected);
}

#[test]
fn test_get_html_with_links() {
    let markdown = "[example](http://example.com)";
    let expected = "<p><a href=\"http://example.com\">example</a></p>\n";
    assert_eq!(get_html(markdown), expected);
}

#[test]
fn test_get_html_with_internal_relative_links() {
    let markdown = "[internal](./test.md)";
    let expected = "<p><a href=\"./test.md\">internal</a></p>\n";
    assert_eq!(get_html(markdown), expected);
}

#[test]
fn test_get_html_with_internal_links_no_slash() {
    let markdown = "[internal](test.md)";
    let expected = "<p><a href=\"test.html\">internal</a></p>\n";
    assert_eq!(get_html(markdown), expected);
}

#[test]
fn test_get_html_with_images() {
    let markdown = "![alt text](media/image.jpg)";
    let expected = "<p><figure><img src=\"media/image.jpg\" alt=\"alt text\" /></figure></p>\n";
    assert_eq!(get_html(markdown), expected);
}

#[test]
fn test_blockquotes_without_space() {
    let markdown = ">testing blockquote";
    let expected = "<blockquote>\n<p>testing blockquote</p>\n</blockquote>\n";
    assert_eq!(get_html(markdown), expected);
}

#[test]
fn test_blockquotes_with_space() {
    let markdown = "> testing blockquote";
    let expected = "<blockquote>\n<p>testing blockquote</p>\n</blockquote>\n";
    assert_eq!(get_html(markdown), expected);
}

#[test]
fn test_blockquotes_with_multiline() {
    let markdown = "> testing blockquote\n> line 2";
    let expected = "<blockquote>\n<p>testing blockquote\nline 2</p>\n</blockquote>\n";
    assert_eq!(get_html(markdown), expected);
}

#[test]
fn test_blockquotes_with_triple() {
    let markdown = ">>>\ntesting blockquote\n line 2\n>>>";
    let expected = "<blockquote>\n<p>testing blockquote\nline 2</p>\n</blockquote>\n";
    assert_eq!(get_html(markdown), expected);
}

#[test]
fn test_fix_internal_links_with_media_files() {
    let html = r#"<a href="media/image.jpg">View Image</a>"#;
    let expected = r#"<a href="media/image.jpg">View Image</a>"#;
    assert_eq!(fix_internal_links(html), expected);
}

#[test]
fn test_fix_internal_links_with_media_files_webp() {
    let html = r#"<a href="/media/homecloudpart1-mikrotik.webp">View Image</a>"#;
    let expected = r#"<a href="/media/homecloudpart1-mikrotik.webp">View Image</a>"#;
    assert_eq!(fix_internal_links(html), expected);
}

#[test]
fn test_fix_internal_links_with_various_media_extensions() {
    let test_cases = vec![
        ("media/image.jpg", "media/image.jpg"),
        ("media/image.jpeg", "media/image.jpeg"),
        ("media/image.png", "media/image.png"),
        ("media/image.gif", "media/image.gif"),
        ("media/image.webp", "media/image.webp"),
        ("media/image.svg", "media/image.svg"),
        ("media/image.avif", "media/image.avif"),
        ("media/video.mp4", "media/video.mp4"),
        ("media/audio.mp3", "media/audio.mp3"),
        ("media/document.pdf", "media/document.pdf"),
    ];

    for (input, expected) in test_cases {
        let html = format!(r#"<a href="{input}">Link</a>"#);
        let expected_html = format!(r#"<a href="{expected}">Link</a>"#);
        assert_eq!(fix_internal_links(&html), expected_html);
    }
}

#[test]
fn test_get_html_with_code_block() {
    let markdown = "```\nlet x = 1;\n```";
    let expected = "<pre><code>let x = 1;\n</code></pre>\n";
    assert_eq!(get_html(markdown), expected);
}

#[test]
fn test_get_html_with_task_list() {
    let markdown = "- [x] Task 1\n- [ ] Task 2";
    let expected = "<ul>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> Task 1</li>\n<li><input type=\"checkbox\" disabled=\"\" /> Task 2</li>\n</ul>\n";
    assert_eq!(get_html(markdown), expected);
}

#[test]
fn test_get_html_with_table() {
    let markdown = "| Header1 | Header2 |\n| ------- | ------- |\n| Cell1   | Cell2   |";
    let expected = "<table>\n<thead>\n<tr>\n<th>Header1</th>\n<th>Header2</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>Cell1</td>\n<td>Cell2</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(get_html(markdown), expected);
}

#[test]
fn test_get_table_of_contents_from_html_with_single_header() {
    let html = r##"<h1><a href="#header1"></a>Header 1</h1>"##;
    let expected = "<ul>\n<li><a href=\"#header1\">Header 1</a></li>\n</ul>\n";
    assert_eq!(get_table_of_contents_from_html(html), expected);
}

#[test]
fn test_get_table_of_contents_from_html_with_multiple_headers() {
    let html = r##"
        <h1><a href="#header1"></a>Header 1</h1>
        <h2><a href="#header2"></a>Header 2</h2>
        <h3><a href="#header3"></a>Header 3</h3>
    "##;
    let expected = "<ul>\n<li><a href=\"#header1\">Header 1</a></li>\n<ul>\n<li><a href=\"#header2\">Header 2</a></li>\n<ul>\n<li><a href=\"#header3\">Header 3</a></li>\n</ul>\n</ul>\n</ul>\n";
    assert_eq!(get_table_of_contents_from_html(html), expected);
}

#[test]
fn test_get_table_of_contents_from_html_with_nested_headers() {
    let html = r##"
        <h1><a href="#header1"></a>Header 1</h1>
        <h2><a href="#header2"></a>Header 2</h2>
        <h1><a href="#header3"></a>Header 3</h1>
    "##;
    let expected = "<ul>\n<li><a href=\"#header1\">Header 1</a></li>\n<ul>\n<li><a href=\"#header2\">Header 2</a></li>\n</ul>\n<li><a href=\"#header3\">Header 3</a></li>\n</ul>\n";
    assert_eq!(get_table_of_contents_from_html(html), expected);
}

#[test]
fn test_get_table_of_contents_from_html_with_no_headers() {
    let html = r"<p>No headers here</p>";
    let expected = "";
    assert_eq!(get_table_of_contents_from_html(html), expected);
}

#[test]
fn test_get_table_of_contents_from_html_with_mixed_content() {
    let html = r##"
        <h1><a href="#header1"></a>Header 1</h1>
        <p>Some content</p>
        <h2><a href="#header2"></a>Header 2</h2>
        <p>More content</p>
    "##;
    let expected = "<ul>\n<li><a href=\"#header1\">Header 1</a></li>\n<ul>\n<li><a href=\"#header2\">Header 2</a></li>\n</ul>\n</ul>\n";
    assert_eq!(get_table_of_contents_from_html(html), expected);
}
