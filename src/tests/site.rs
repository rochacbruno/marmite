use super::*;
use crate::content::ContentBuilder;
use chrono::NaiveDate;
use std::path::PathBuf;
use std::sync::Mutex;

#[test]
fn test_next_and_previous_links_are_stream_specific() {
    let mut site_data = Data::new("", Path::new("marmite.yaml"));
    let post1 = ContentBuilder::new()
        .title("Post 1".to_string())
        .slug("post-1".to_string())
        .stream("blog".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();
    let post2 = ContentBuilder::new()
        .title("Post 2".to_string())
        .slug("post-2".to_string())
        .stream("blog".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 2)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();
    let post3 = ContentBuilder::new()
        .title("Post 3".to_string())
        .slug("post-3".to_string())
        .stream("news".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 3)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    site_data.posts.push(post1.clone());
    site_data.posts.push(post2.clone());
    site_data.posts.push(post3.clone());
    site_data.sort_all();

    let mutex = Mutex::new(site_data);
    let mut site_data_guard = mutex.lock().unwrap();

    set_next_and_previous_links(&mut site_data_guard);

    let blog_post1 = site_data_guard
        .posts
        .iter()
        .find(|p| p.slug == "post-1")
        .unwrap();
    assert!(blog_post1.next.is_some());
    assert_eq!(blog_post1.next.as_ref().unwrap().slug, "post-2");
    assert!(blog_post1.previous.is_none());

    let blog_post2 = site_data_guard
        .posts
        .iter()
        .find(|p| p.slug == "post-2")
        .unwrap();
    assert!(blog_post2.previous.is_some());
    assert_eq!(blog_post2.previous.as_ref().unwrap().slug, "post-1");
    assert!(blog_post2.next.is_none());

    let news_post = site_data_guard
        .posts
        .iter()
        .find(|p| p.slug == "post-3")
        .unwrap();
    assert!(news_post.next.is_none());
    assert!(news_post.previous.is_none());
}

#[test]
fn test_file_mapping() {
    use crate::config::FileMapping;
    use tempfile::TempDir;

    // Create temp directories
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    // Create test files in input directory
    let test_file = input_dir.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    let ai_dir = input_dir.path().join("ai");
    fs::create_dir(&ai_dir).unwrap();
    let llms_file = ai_dir.join("llms.txt");
    fs::write(&llms_file, "llms content").unwrap();

    // Create file mappings
    let mappings = vec![
        FileMapping {
            source: "test.txt".to_string(),
            dest: "output_test.txt".to_string(),
        },
        FileMapping {
            source: "ai/llms.txt".to_string(),
            dest: "llms.txt".to_string(),
        },
    ];

    // Execute file mappings
    handle_file_mappings(input_dir.path(), output_dir.path(), &mappings);

    // Check that files were copied correctly
    let output_test = output_dir.path().join("output_test.txt");
    assert!(output_test.exists());
    assert_eq!(fs::read_to_string(output_test).unwrap(), "test content");

    let output_llms = output_dir.path().join("llms.txt");
    assert!(output_llms.exists());
    assert_eq!(fs::read_to_string(output_llms).unwrap(), "llms content");
}

#[test]
fn test_file_mapping_glob() {
    use crate::config::FileMapping;
    use tempfile::TempDir;

    // Create temp directories
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    // Create test files
    let assets_dir = input_dir.path().join("assets");
    let imgs_dir = assets_dir.join("imgs");
    fs::create_dir_all(&imgs_dir).unwrap();

    fs::write(imgs_dir.join("image1.jpg"), "image1").unwrap();
    fs::write(imgs_dir.join("image2.jpg"), "image2").unwrap();
    fs::write(imgs_dir.join("doc.pdf"), "pdf").unwrap();

    // Create glob mapping
    let mappings = vec![FileMapping {
        source: "assets/imgs/*.jpg".to_string(),
        dest: "media/photos".to_string(),
    }];

    // Execute file mappings
    handle_file_mappings(input_dir.path(), output_dir.path(), &mappings);

    // Check that only jpg files were copied
    let photos_dir = output_dir.path().join("media/photos");
    assert!(photos_dir.exists());
    assert!(photos_dir.join("image1.jpg").exists());
    assert!(photos_dir.join("image2.jpg").exists());
    assert!(!photos_dir.join("doc.pdf").exists());
}

#[test]
fn test_file_mapping_nested_destination() {
    use crate::config::FileMapping;
    use tempfile::TempDir;

    // Create temp directories
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    // Create test file
    let test_file = input_dir.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    // Create mapping with nested destination
    let mappings = vec![FileMapping {
        source: "test.txt".to_string(),
        dest: "foo/bar/baz/test.txt".to_string(),
    }];

    // Execute file mappings
    handle_file_mappings(input_dir.path(), output_dir.path(), &mappings);

    // Check that nested directories were created and file was copied
    let nested_file = output_dir.path().join("foo/bar/baz/test.txt");
    assert!(nested_file.exists());
    assert_eq!(fs::read_to_string(nested_file).unwrap(), "test content");
}

#[test]
fn test_file_mapping_glob_nested_destination() {
    use crate::config::FileMapping;
    use tempfile::TempDir;

    // Create temp directories
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    // Create test files
    let docs_dir = input_dir.path().join("docs");
    fs::create_dir(&docs_dir).unwrap();
    fs::write(docs_dir.join("manual.pdf"), "manual").unwrap();
    fs::write(docs_dir.join("guide.pdf"), "guide").unwrap();
    fs::write(docs_dir.join("readme.txt"), "readme").unwrap();

    // Create glob mapping with nested destination
    let mappings = vec![FileMapping {
        source: "docs/*.pdf".to_string(),
        dest: "assets/downloads/pdfs".to_string(),
    }];

    // Execute file mappings
    handle_file_mappings(input_dir.path(), output_dir.path(), &mappings);

    // Check that nested directories were created and files were copied
    let pdfs_dir = output_dir.path().join("assets/downloads/pdfs");
    assert!(pdfs_dir.exists());
    assert!(pdfs_dir.join("manual.pdf").exists());
    assert!(pdfs_dir.join("guide.pdf").exists());
    assert!(!pdfs_dir.join("readme.txt").exists());
}

#[test]
fn test_file_mapping_directory() {
    use crate::config::FileMapping;
    use tempfile::TempDir;

    // Create temp directories
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    // Create a directory with files
    let src_dir = input_dir.path().join("source_dir");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("file1.txt"), "content1").unwrap();
    fs::write(src_dir.join("file2.txt"), "content2").unwrap();

    // Create directory mapping
    let mappings = vec![FileMapping {
        source: "source_dir".to_string(),
        dest: "dest_dir".to_string(),
    }];

    // Execute file mappings
    handle_file_mappings(input_dir.path(), output_dir.path(), &mappings);

    // Check that directory was copied
    // fs_extra::dir::copy copies the contents AND the directory itself into dest
    let dest_dir = output_dir.path().join("dest_dir");

    // First check if dest_dir was created
    assert!(dest_dir.exists(), "dest_dir should exist");

    // The source directory 'source_dir' should be copied INTO dest_dir
    let copied_dir = dest_dir.join("source_dir");
    assert!(
        copied_dir.exists(),
        "source_dir should be copied into dest_dir"
    );
    assert!(copied_dir.join("file1.txt").exists());
    assert!(copied_dir.join("file2.txt").exists());
}

#[test]
fn test_data_new() {
    let config_yaml = r#"
name: "Test Site"
tagline: "A test site"
pagination: 5
"#;
    let config_path = Path::new("test_config.yaml");
    let data = Data::new(config_yaml, config_path);

    assert_eq!(data.site.name, "Test Site");
    assert_eq!(data.site.tagline, "A test site");
    assert_eq!(data.site.pagination, 5);
    assert_eq!(data.config_path, "test_config.yaml");
    assert!(data.posts.is_empty());
    assert!(data.pages.is_empty());
    assert!(!data.force_render);
}

#[test]
fn test_data_sort_all() {
    let mut data = Data::new("", Path::new("test.yaml"));

    // Create test posts with different dates
    let post1 = ContentBuilder::new()
        .title("Post 1".to_string())
        .slug("post-1".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let post2 = ContentBuilder::new()
        .title("Post 2".to_string())
        .slug("post-2".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 2, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    // Add posts out of date order
    data.posts.push(post1.clone());
    data.posts.push(post2.clone());

    // Create test pages
    let page1 = ContentBuilder::new()
        .title("Z Page".to_string())
        .slug("z-page".to_string())
        .build();

    let page2 = ContentBuilder::new()
        .title("A Page".to_string())
        .slug("a-page".to_string())
        .build();

    // Add pages out of alphabetical order
    data.pages.push(page1.clone());
    data.pages.push(page2.clone());

    // Sort everything
    data.sort_all();

    // Check posts are sorted by date (newest first)
    assert_eq!(data.posts[0].title, "Post 2");
    assert_eq!(data.posts[1].title, "Post 1");

    // Check pages are sorted by title (alphabetical)
    assert_eq!(data.pages[0].title, "Z Page");
    assert_eq!(data.pages[1].title, "A Page");
}

#[test]
fn test_data_push_content_post() {
    let mut data = Data::new("", Path::new("test.yaml"));

    let post = ContentBuilder::new()
        .title("Test Post".to_string())
        .slug("test-post".to_string())
        .tags(vec!["rust".to_string(), "testing".to_string()])
        .authors(vec!["alice".to_string(), "bob".to_string()])
        .stream("blog".to_string())
        .series("tutorial".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 3, 15)
                .unwrap()
                .and_hms_opt(10, 30, 0)
                .unwrap(),
        )
        .build();

    data.push_content(post.clone());

    // Check post was added
    assert_eq!(data.posts.len(), 1);
    assert_eq!(data.posts[0].title, "Test Post");

    // Check tags were added
    assert!(data.tag.map.contains_key("rust"));
    assert!(data.tag.map.contains_key("testing"));
    assert_eq!(data.tag.map["rust"].len(), 1);
    assert_eq!(data.tag.map["testing"].len(), 1);

    // Check authors were added
    assert!(data.author.map.contains_key("alice"));
    assert!(data.author.map.contains_key("bob"));
    assert_eq!(data.author.map["alice"].len(), 1);
    assert_eq!(data.author.map["bob"].len(), 1);

    // Check archive by year
    assert!(data.archive.map.contains_key("2024"));
    assert_eq!(data.archive.map["2024"].len(), 1);

    // Check stream
    assert!(data.stream.map.contains_key("blog"));
    assert_eq!(data.stream.map["blog"].len(), 1);

    // Check series
    assert!(data.series.map.contains_key("tutorial"));
    assert_eq!(data.series.map["tutorial"].len(), 1);
}

#[test]
fn test_data_push_content_page() {
    let mut data = Data::new("", Path::new("test.yaml"));

    let page = ContentBuilder::new()
        .title("About Page".to_string())
        .slug("about".to_string())
        .build(); // No date, so it's a page

    data.push_content(page.clone());

    // Check page was added
    assert_eq!(data.pages.len(), 1);
    assert_eq!(data.pages[0].title, "About Page");

    // Check it wasn't added to post-related collections
    assert_eq!(data.posts.len(), 0);
    assert!(data.tag.map.is_empty());
    assert!(data.author.map.is_empty());
    assert!(data.archive.map.is_empty());
    assert!(data.stream.map.is_empty());
    assert!(data.series.map.is_empty());
}

#[test]
fn test_url_collection_add_url() {
    let mut url_collection = UrlCollection::default();

    url_collection.add_url("posts", "post-1.html".to_string());
    url_collection.add_url("pages", "about.html".to_string());
    url_collection.add_url("posts", "post-2.html".to_string());

    assert_eq!(url_collection.posts.len(), 2);
    assert_eq!(url_collection.pages.len(), 1);
    assert!(url_collection.posts.contains(&"post-1.html".to_string()));
    assert!(url_collection.posts.contains(&"post-2.html".to_string()));
    assert!(url_collection.pages.contains(&"about.html".to_string()));
}

#[test]
fn test_get_content_folder() {
    use tempfile::TempDir;

    // Create a temporary directory structure
    let temp_dir = TempDir::new().unwrap();
    let input_folder = temp_dir.path();

    // Create a custom_content directory
    let custom_content_path = input_folder.join("custom_content");
    fs::create_dir(&custom_content_path).unwrap();

    let config = Marmite {
        content_path: "custom_content".to_string(),
        ..Default::default()
    };

    let content_folder = get_content_folder(&config, input_folder);
    assert_eq!(content_folder, custom_content_path);

    // Test with non-existing directory - should fall back to input_folder
    let config_nonexistent = Marmite {
        content_path: "nonexistent".to_string(),
        ..Default::default()
    };

    let content_folder_fallback = get_content_folder(&config_nonexistent, input_folder);
    assert_eq!(content_folder_fallback, input_folder);
}

#[test]
fn test_validate_internal_links_no_broken() {
    let mut data = Data::new("", Path::new("test.yaml"));

    let post = ContentBuilder::new()
        .title("Post 1".to_string())
        .slug("post-1".to_string())
        .links_to(vec!["about".to_string()])
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let page = ContentBuilder::new()
        .title("About".to_string())
        .slug("about".to_string())
        .build();

    data.push_content(post);
    data.push_content(page);
    data.collect_all_urls();

    let broken = validate_internal_links(&data);
    assert!(broken.is_empty());
}

#[test]
fn test_validate_internal_links_broken() {
    let mut data = Data::new("", Path::new("test.yaml"));

    let post = ContentBuilder::new()
        .title("Post 1".to_string())
        .slug("post-1".to_string())
        .links_to(vec!["nonexistent-page".to_string()])
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    data.push_content(post);
    data.collect_all_urls();

    let broken = validate_internal_links(&data);
    assert_eq!(broken.len(), 1);
    assert_eq!(broken[0].0, "post-1");
    assert_eq!(broken[0].1, "nonexistent-page");
}

#[test]
fn test_validate_internal_links_with_anchor() {
    let mut data = Data::new("", Path::new("test.yaml"));

    let post = ContentBuilder::new()
        .title("Post 1".to_string())
        .slug("post-1".to_string())
        .links_to(vec!["about#section".to_string()])
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let page = ContentBuilder::new()
        .title("About".to_string())
        .slug("about".to_string())
        .build();

    data.push_content(post);
    data.push_content(page);
    data.collect_all_urls();

    let broken = validate_internal_links(&data);
    assert!(
        broken.is_empty(),
        "Anchored links to valid slugs should not be broken"
    );
}

#[test]
fn test_validate_internal_links_alias_is_valid() {
    let mut data = Data::new("", Path::new("test.yaml"));

    let post = ContentBuilder::new()
        .title("Post 1".to_string())
        .slug("post-1".to_string())
        .links_to(vec!["old-about-url".to_string()])
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let page = ContentBuilder::new()
        .title("About".to_string())
        .slug("about".to_string())
        .aliases(vec!["old-about-url".to_string()])
        .build();

    data.push_content(post);
    data.push_content(page);
    data.collect_all_urls();

    let broken = validate_internal_links(&data);
    assert!(
        broken.is_empty(),
        "Links to redirect aliases should be valid"
    );
}

#[test]
fn test_generate_redirect_html() {
    let html = generate_redirect_html("/my-post.html");
    assert!(html.contains(r#"url=/my-post.html"#));
    assert!(html.contains(r#"rel="canonical" href="/my-post.html""#));
    assert!(html.contains(r#"window.location.href = "/my-post.html";"#));
    assert!(html.contains("<title>Redirecting...</title>"));
}

#[test]
fn test_handle_redirect_aliases_writes_files() {
    use tempfile::TempDir;

    let mut data = Data::new("", Path::new("test.yaml"));

    let post = ContentBuilder::new()
        .title("New Post".to_string())
        .slug("new-post".to_string())
        .aliases(vec!["old-post".to_string(), "legacy".to_string()])
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    data.push_content(post);

    let temp_dir = TempDir::new().unwrap();
    let result = handle_redirect_aliases(&data, temp_dir.path());
    assert!(result.is_ok());

    let old_post = fs::read_to_string(temp_dir.path().join("old-post.html")).unwrap();
    assert!(old_post.contains("new-post.html"));

    let legacy = fs::read_to_string(temp_dir.path().join("legacy.html")).unwrap();
    assert!(legacy.contains("new-post.html"));
}

#[test]
fn test_handle_redirect_aliases_skips_slug_conflict() {
    use tempfile::TempDir;

    let mut data = Data::new("", Path::new("test.yaml"));

    let post = ContentBuilder::new()
        .title("Post A".to_string())
        .slug("post-a".to_string())
        .aliases(vec!["about".to_string()])
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let page = ContentBuilder::new()
        .title("About".to_string())
        .slug("about".to_string())
        .build();

    data.push_content(post);
    data.push_content(page);

    let temp_dir = TempDir::new().unwrap();
    let result = handle_redirect_aliases(&data, temp_dir.path());
    assert!(result.is_ok());

    assert!(
        !temp_dir.path().join("about.html").exists(),
        "Alias conflicting with existing slug should not be written"
    );
}

#[test]
fn test_handle_redirect_aliases_skips_duplicate() {
    use tempfile::TempDir;

    let mut data = Data::new("", Path::new("test.yaml"));

    let post1 = ContentBuilder::new()
        .title("Post 1".to_string())
        .slug("post-1".to_string())
        .aliases(vec!["shared-alias".to_string()])
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let post2 = ContentBuilder::new()
        .title("Post 2".to_string())
        .slug("post-2".to_string())
        .aliases(vec!["shared-alias".to_string()])
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 2)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    data.push_content(post1);
    data.push_content(post2);

    let temp_dir = TempDir::new().unwrap();
    let result = handle_redirect_aliases(&data, temp_dir.path());
    assert!(result.is_ok());

    let alias_html = fs::read_to_string(temp_dir.path().join("shared-alias.html")).unwrap();
    assert!(
        alias_html.contains("post-1.html") || alias_html.contains("post-2.html"),
        "First alias should be written, duplicate skipped"
    );
}

#[test]
fn test_url_collection_redirects() {
    let mut url_collection = UrlCollection::default();

    url_collection.add_url("redirects", "old-post.html".to_string());
    url_collection.add_url("posts", "new-post.html".to_string());

    assert_eq!(url_collection.redirects.len(), 1);
    assert!(url_collection
        .redirects
        .contains(&"old-post.html".to_string()));

    let all_urls = url_collection.get_all_urls();
    assert!(
        !all_urls.contains(&"old-post.html".to_string()),
        "Redirects should be excluded from get_all_urls"
    );
    assert!(all_urls.contains(&"new-post.html".to_string()));

    assert_eq!(url_collection.total_count(), 2);
}

#[test]
fn test_collect_all_urls_includes_aliases() {
    let mut data = Data::new("", Path::new("test.yaml"));

    let post = ContentBuilder::new()
        .title("My Post".to_string())
        .slug("my-post".to_string())
        .aliases(vec!["old-url".to_string()])
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    data.push_content(post);
    data.collect_all_urls();

    assert!(data
        .generated_urls
        .redirects
        .contains(&"old-url.html".to_string()));
    assert_eq!(data.generated_urls.redirects.len(), 1);
}

// --- parse_frontmatter_yaml tests ---

#[test]
fn test_parse_frontmatter_yaml_empty_string() {
    let result = parse_frontmatter_yaml("");
    assert!(result.is_some());
    let fm = result.unwrap();
    assert!(fm.is_empty());
}

#[test]
fn test_parse_frontmatter_yaml_whitespace_only() {
    let result = parse_frontmatter_yaml("   \n  \n  ");
    assert!(result.is_some());
    let fm = result.unwrap();
    assert!(fm.is_empty());
}

#[test]
fn test_parse_frontmatter_yaml_valid() {
    let yaml = "title: Hello World\ntags:\n  - rust\n  - testing";
    let result = parse_frontmatter_yaml(yaml);
    assert!(result.is_some());
    let fm = result.unwrap();
    assert_eq!(
        fm.get("title").and_then(|v| v.as_str()),
        Some("Hello World")
    );
}

#[test]
fn test_parse_frontmatter_yaml_invalid() {
    let yaml = ":::not valid yaml:::\n  - [broken";
    let result = parse_frontmatter_yaml(yaml);
    assert!(result.is_none());
}

// --- is_core_static_file tests ---

#[test]
fn test_is_core_static_file_known() {
    assert!(is_core_static_file("marmite.css"));
    assert!(is_core_static_file("marmite.js"));
    assert!(is_core_static_file("search.js"));
    assert!(is_core_static_file("pico.min.css"));
    assert!(is_core_static_file(
        "AtkinsonHyperlegibleNext-Regular.woff2"
    ));
}

#[test]
fn test_is_core_static_file_colorschemes() {
    assert!(is_core_static_file("colorschemes/gruvbox.css"));
    assert!(is_core_static_file("colorschemes/nord.css"));
}

#[test]
fn test_is_core_static_file_custom() {
    assert!(!is_core_static_file("custom.css"));
    assert!(!is_core_static_file("logo.png"));
    assert!(!is_core_static_file("favicon.ico"));
}

// --- ContentInfo::from_content tests ---

#[test]
fn test_content_info_from_content_post() {
    let post = ContentBuilder::new()
        .title("My Post".to_string())
        .slug("my-post".to_string())
        .description("A description".to_string())
        .tags(vec!["rust".to_string()])
        .authors(vec!["alice".to_string()])
        .stream("blog".to_string())
        .series("tutorial".to_string())
        .pinned(true)
        .source_path(PathBuf::from("/content/my-post.md"))
        .date(
            NaiveDate::from_ymd_opt(2024, 6, 15)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap(),
        )
        .build();

    let info = ContentInfo::from_content(&post);
    assert_eq!(info.title, "My Post");
    assert_eq!(info.slug, "my-post");
    assert_eq!(info.url, "/my-post.html");
    assert_eq!(info.date, Some("2024-06-15".to_string()));
    assert_eq!(info.tags, vec!["rust".to_string()]);
    assert_eq!(info.authors, vec!["alice".to_string()]);
    assert_eq!(info.description, Some("A description".to_string()));
    assert_eq!(info.stream, Some("blog".to_string()));
    assert_eq!(info.series, Some("tutorial".to_string()));
    assert!(info.pinned);
    assert!(info.source_path.is_some());
}

#[test]
fn test_content_info_from_content_page() {
    let page = ContentBuilder::new()
        .title("About".to_string())
        .slug("about".to_string())
        .build();

    let info = ContentInfo::from_content(&page);
    assert_eq!(info.title, "About");
    assert_eq!(info.slug, "about");
    assert_eq!(info.url, "/about.html");
    assert!(info.date.is_none());
    assert!(info.tags.is_empty());
    assert!(info.source_path.is_none());
    assert!(!info.pinned);
}

// --- BuildInfo::from_json tests ---

#[test]
fn test_build_info_from_json_valid() {
    let json = r#"{
        "marmite_version": "0.3.3-dev",
        "posts": [],
        "pages": [],
        "shortcodes": [],
        "generated_at": "2024-01-01T00:00:00",
        "timestamp": 1704067200,
        "elapsed_time": 1.5,
        "config": {}
    }"#;
    let result = BuildInfo::from_json(json);
    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.marmite_version, "0.3.3-dev");
    assert_eq!(info.timestamp, 1704067200);
}

#[test]
fn test_build_info_from_json_invalid() {
    let result = BuildInfo::from_json("not json");
    assert!(result.is_err());
}

// --- collect_back_links tests ---

#[test]
fn test_collect_back_links_basic() {
    let mut data = Data::new("", Path::new("test.yaml"));

    let post_a = ContentBuilder::new()
        .title("Post A".to_string())
        .slug("post-a".to_string())
        .links_to(vec!["post-b".to_string()])
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let post_b = ContentBuilder::new()
        .title("Post B".to_string())
        .slug("post-b".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 2)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    data.posts.push(post_a);
    data.posts.push(post_b);

    collect_back_links(&mut data);

    let post_b = data.posts.iter().find(|p| p.slug == "post-b").unwrap();
    assert_eq!(post_b.back_links.len(), 1);
    assert_eq!(post_b.back_links[0].slug, "post-a");

    let post_a = data.posts.iter().find(|p| p.slug == "post-a").unwrap();
    assert!(post_a.back_links.is_empty());
}

#[test]
fn test_collect_back_links_no_self_link() {
    let mut data = Data::new("", Path::new("test.yaml"));

    let post = ContentBuilder::new()
        .title("Self".to_string())
        .slug("self-post".to_string())
        .links_to(vec!["self-post".to_string()])
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    data.posts.push(post);
    collect_back_links(&mut data);

    assert!(data.posts[0].back_links.is_empty());
}

#[test]
fn test_collect_back_links_cross_post_page() {
    let mut data = Data::new("", Path::new("test.yaml"));

    let post = ContentBuilder::new()
        .title("Post".to_string())
        .slug("my-post".to_string())
        .links_to(vec!["about".to_string()])
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let page = ContentBuilder::new()
        .title("About".to_string())
        .slug("about".to_string())
        .build();

    data.posts.push(post);
    data.pages.push(page);

    collect_back_links(&mut data);

    assert_eq!(data.pages[0].back_links.len(), 1);
    assert_eq!(data.pages[0].back_links[0].slug, "my-post");
}

// --- collect_content_fragments tests ---

#[test]
fn test_collect_content_fragments_with_files() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("_markdown_header.md"), "# Header").unwrap();
    fs::write(temp.path().join("_markdown_footer.md"), "---\nFooter").unwrap();
    fs::write(
        temp.path().join("_references.md"),
        "[ref]: http://example.com",
    )
    .unwrap();

    let fragments = collect_content_fragments(temp.path());

    assert_eq!(fragments["markdown_header"], "# Header");
    assert_eq!(fragments["markdown_footer"], "---\nFooter");
    assert_eq!(fragments["references"], "[ref]: http://example.com");
}

#[test]
fn test_collect_content_fragments_missing_files() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    let fragments = collect_content_fragments(temp.path());

    assert_eq!(fragments["markdown_header"], "");
    assert_eq!(fragments["markdown_footer"], "");
    assert_eq!(fragments["references"], "");
}

// --- collect_global_fragments tests ---

#[test]
fn test_collect_global_fragments_inserts_into_context() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("_hero.md"), "# Welcome").unwrap();
    fs::write(temp.path().join("_footer.md"), "Copyright 2024").unwrap();

    let mut context = Context::new();
    let tera = Tera::default();
    let config = Marmite::default();

    collect_global_fragments(temp.path(), &mut context, &tera, &config, None);

    let hero: &Value = context.get("hero").unwrap();
    assert!(hero.as_str().unwrap().contains("Welcome"));

    let footer: &Value = context.get("footer").unwrap();
    assert!(footer.as_str().unwrap().contains("Copyright 2024"));
}

#[test]
fn test_collect_global_fragments_skips_missing() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    let mut context = Context::new();
    let tera = Tera::default();
    let config = Marmite::default();

    collect_global_fragments(temp.path(), &mut context, &tera, &config, None);

    assert!(context.get("hero").is_none());
    assert!(context.get("footer").is_none());
    assert!(context.get("announce").is_none());
}

// --- load_folder_frontmatter tests ---

#[test]
fn test_load_folder_frontmatter_basic() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    let sub = temp.path().join("posts");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("frontmatter.yaml"), "tags:\n  - default-tag").unwrap();

    let result = load_folder_frontmatter(temp.path());
    assert_eq!(result.len(), 1);
    assert!(result.contains_key(&sub));
}

#[test]
fn test_load_folder_frontmatter_empty_dir() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    let result = load_folder_frontmatter(temp.path());
    assert!(result.is_empty());
}

#[test]
fn test_load_folder_frontmatter_nested_inheritance() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    let parent = temp.path().join("blog");
    let child = parent.join("2024");
    fs::create_dir_all(&child).unwrap();

    fs::write(parent.join("frontmatter.yaml"), "stream: blog").unwrap();
    fs::write(child.join("frontmatter.yaml"), "tags:\n  - yearly").unwrap();

    let result = load_folder_frontmatter(temp.path());
    assert_eq!(result.len(), 2);
    assert!(result.contains_key(&parent));
    assert!(result.contains_key(&child));

    let child_fm = &result[&child];
    assert!(
        child_fm.get("stream").is_some(),
        "Child should inherit parent's stream field"
    );
}

// --- discover_translations tests ---

#[test]
fn test_discover_translations_from_stream_language() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    let mut data = Data::new("", Path::new("test.yaml"));

    let post_en = ContentBuilder::new()
        .title("Hello".to_string())
        .slug("hello".to_string())
        .stream("en".to_string())
        .source_path(temp.path().join("hello.md"))
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    data.posts.push(post_en);

    discover_translations(&mut data, temp.path());

    assert_eq!(
        data.posts[0].language,
        Some("en".to_string()),
        "Language should be inferred from stream"
    );
}

#[test]
fn test_discover_translations_subfolder_grouping() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();
    let content_dir = temp.path();
    let post_dir = content_dir.join("hello-world");
    fs::create_dir(&post_dir).unwrap();

    let mut data = Data::new("", Path::new("test.yaml"));

    let post_en = ContentBuilder::new()
        .title("Hello World".to_string())
        .slug("hello-world".to_string())
        .language("en".to_string())
        .source_path(post_dir.join("hello-world.md"))
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let post_pt = ContentBuilder::new()
        .title("Ola Mundo".to_string())
        .slug("pt-ola-mundo".to_string())
        .language("pt".to_string())
        .source_path(post_dir.join("pt-ola-mundo.md"))
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    data.posts.push(post_en);
    data.posts.push(post_pt);

    discover_translations(&mut data, content_dir);

    let en_post = data.posts.iter().find(|p| p.slug == "hello-world").unwrap();
    assert_eq!(en_post.translations.len(), 1);
    assert_eq!(en_post.translations[0].slug, "pt-ola-mundo");
    assert_eq!(en_post.translations[0].lang, "pt");

    let pt_post = data
        .posts
        .iter()
        .find(|p| p.slug == "pt-ola-mundo")
        .unwrap();
    assert_eq!(pt_post.translations.len(), 1);
    assert_eq!(pt_post.translations[0].slug, "hello-world");
    assert_eq!(pt_post.translations[0].lang, "en");
}

#[test]
fn test_discover_translations_manual_translates_frontmatter() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    let mut data = Data::new("", Path::new("test.yaml"));

    let post_en = ContentBuilder::new()
        .title("Hello".to_string())
        .slug("hello".to_string())
        .language("en".to_string())
        .source_path(temp.path().join("hello.md"))
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let post_pt = ContentBuilder::new()
        .title("Ola".to_string())
        .slug("ola".to_string())
        .language("pt".to_string())
        .translates("hello".to_string())
        .source_path(temp.path().join("ola.md"))
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    data.posts.push(post_en);
    data.posts.push(post_pt);

    discover_translations(&mut data, temp.path());

    let pt_post = data.posts.iter().find(|p| p.slug == "ola").unwrap();
    assert!(
        pt_post.translations.iter().any(|t| t.slug == "hello"),
        "translates: frontmatter should create translation link"
    );

    let en_post = data.posts.iter().find(|p| p.slug == "hello").unwrap();
    assert!(
        en_post.translations.iter().any(|t| t.slug == "ola"),
        "Bidirectional link should be created"
    );
}

#[test]
fn test_discover_translations_page_language_from_filename() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    let mut data = Data::new("", Path::new("test.yaml"));

    let page = ContentBuilder::new()
        .title("Sobre".to_string())
        .slug("pt-sobre".to_string())
        .source_path(temp.path().join("pt-sobre.md"))
        .build();

    data.pages.push(page);

    discover_translations(&mut data, temp.path());

    assert_eq!(
        data.pages[0].language,
        Some("pt".to_string()),
        "Page language should be inferred from pt- filename prefix"
    );
}

#[test]
fn test_discover_translations_no_translations() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    let mut data = Data::new("", Path::new("test.yaml"));

    let post = ContentBuilder::new()
        .title("Solo Post".to_string())
        .slug("solo".to_string())
        .source_path(temp.path().join("solo.md"))
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    data.posts.push(post);

    discover_translations(&mut data, temp.path());

    assert!(data.posts[0].translations.is_empty());
}

// --- create_urls_json tests ---

#[test]
fn test_create_urls_json_with_content() {
    let mut data = Data::new("url: https://example.com", Path::new("test.yaml"));

    let post = ContentBuilder::new()
        .title("Post".to_string())
        .slug("my-post".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let page = ContentBuilder::new()
        .title("About".to_string())
        .slug("about".to_string())
        .build();

    data.push_content(post);
    data.push_content(page);
    data.collect_all_urls();

    let json = create_urls_json(&data, "");

    assert!(json.get("posts").is_some());
    assert!(json.get("pages").is_some());

    let posts = json["posts"].as_array().unwrap();
    assert!(!posts.is_empty());

    let pages = json["pages"].as_array().unwrap();
    assert!(!pages.is_empty());
}

#[test]
fn test_create_urls_json_empty_site() {
    let data = Data::new("", Path::new("test.yaml"));
    let json = create_urls_json(&data, "");

    let posts = json["posts"].as_array().unwrap();
    assert!(posts.is_empty());

    let pages = json["pages"].as_array().unwrap();
    assert!(pages.is_empty());
}

// --- UrlCollection tests ---

#[test]
fn test_url_collection_total_count() {
    let mut urls = UrlCollection::default();
    urls.add_url("posts", "a.html".to_string());
    urls.add_url("pages", "b.html".to_string());
    urls.add_url("tags", "tag-rust.html".to_string());
    urls.add_url("redirects", "old.html".to_string());

    assert_eq!(urls.total_count(), 4);
}

#[test]
fn test_url_collection_get_all_urls_excludes_redirects() {
    let mut urls = UrlCollection::default();
    urls.add_url("posts", "post.html".to_string());
    urls.add_url("redirects", "old.html".to_string());

    let all = urls.get_all_urls();
    assert!(all.contains(&"post.html".to_string()));
    assert!(!all.contains(&"old.html".to_string()));
}

#[test]
fn test_url_collection_all_categories() {
    let mut urls = UrlCollection::default();
    urls.add_url("posts", "p.html".to_string());
    urls.add_url("pages", "pg.html".to_string());
    urls.add_url("tags", "t.html".to_string());
    urls.add_url("archives", "a.html".to_string());
    urls.add_url("authors", "au.html".to_string());
    urls.add_url("streams", "s.html".to_string());
    urls.add_url("series", "sr.html".to_string());
    urls.add_url("feeds", "f.xml".to_string());
    urls.add_url("other", "o.html".to_string());

    let all = urls.get_all_urls();
    assert!(all.contains(&"p.html".to_string()));
    assert!(all.contains(&"pg.html".to_string()));
    assert!(all.contains(&"t.html".to_string()));
    assert!(all.contains(&"a.html".to_string()));
    assert!(all.contains(&"au.html".to_string()));
    assert!(all.contains(&"s.html".to_string()));
    assert!(all.contains(&"sr.html".to_string()));
    assert!(all.contains(&"f.xml".to_string()));
    assert!(all.contains(&"o.html".to_string()));
}

// --- get_latest_build_info tests ---

#[test]
fn test_get_latest_build_info_missing_file() {
    let path = PathBuf::from("/tmp/nonexistent_build_info.json");
    let result = get_latest_build_info(&path).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_get_latest_build_info_valid_file() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    let path = temp.path().join("build_info.json");
    let json = r#"{
        "marmite_version": "0.3.3",
        "posts": [],
        "pages": [],
        "shortcodes": [],
        "generated_at": "2024-01-01T00:00:00",
        "timestamp": 1704067200,
        "elapsed_time": 1.0,
        "config": {}
    }"#;
    fs::write(&path, json).unwrap();

    let result = get_latest_build_info(&path).unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().marmite_version, "0.3.3");
}

#[test]
fn test_get_latest_build_info_invalid_json() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    let path = temp.path().join("build_info.json");
    fs::write(&path, "not valid json").unwrap();

    let result = get_latest_build_info(&path).unwrap();
    assert!(result.is_none());
}

// --- set_next_and_previous_links series tests ---

#[test]
fn test_next_previous_links_series_takes_precedence() {
    let mut site_data = Data::new("", Path::new("test.yaml"));

    let post1 = ContentBuilder::new()
        .title("Part 1".to_string())
        .slug("part-1".to_string())
        .stream("blog".to_string())
        .series("tutorial".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let post2 = ContentBuilder::new()
        .title("Part 2".to_string())
        .slug("part-2".to_string())
        .stream("blog".to_string())
        .series("tutorial".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 2)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let post3 = ContentBuilder::new()
        .title("Part 3".to_string())
        .slug("part-3".to_string())
        .stream("blog".to_string())
        .series("tutorial".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 3)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    site_data.posts.push(post1);
    site_data.posts.push(post2);
    site_data.posts.push(post3);
    site_data.sort_all();

    let mutex = Mutex::new(site_data);
    let mut guard = mutex.lock().unwrap();
    set_next_and_previous_links(&mut guard);

    let p1 = guard.posts.iter().find(|p| p.slug == "part-1").unwrap();
    assert!(p1.previous.is_none());
    assert_eq!(p1.next.as_ref().unwrap().slug, "part-2");

    let p2 = guard.posts.iter().find(|p| p.slug == "part-2").unwrap();
    assert_eq!(p2.previous.as_ref().unwrap().slug, "part-1");
    assert_eq!(p2.next.as_ref().unwrap().slug, "part-3");

    let p3 = guard.posts.iter().find(|p| p.slug == "part-3").unwrap();
    assert_eq!(p3.previous.as_ref().unwrap().slug, "part-2");
    assert!(p3.next.is_none());
}

// --- Data::collect_all_urls tests ---

#[test]
fn test_collect_all_urls_posts_and_pages() {
    let mut data = Data::new("", Path::new("test.yaml"));

    let post = ContentBuilder::new()
        .title("Post".to_string())
        .slug("my-post".to_string())
        .tags(vec!["rust".to_string()])
        .stream("blog".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let page = ContentBuilder::new()
        .title("About".to_string())
        .slug("about".to_string())
        .build();

    data.push_content(post);
    data.push_content(page);
    data.collect_all_urls();

    let all = data.generated_urls.get_all_urls();
    assert!(all.contains(&"my-post.html".to_string()));
    assert!(all.contains(&"about.html".to_string()));
    assert!(all.contains(&"tag-rust.html".to_string()));
    assert!(all.contains(&"blog.html".to_string()));
}

#[test]
fn test_build_language_index_groups_posts_by_language() {
    let mut data = Data::new("language: en", Path::new("marmite.yaml"));

    let en_post = ContentBuilder::new()
        .title("Hello World".to_string())
        .slug("hello-world".to_string())
        .language("en".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let pt_post = ContentBuilder::new()
        .title("Ola Mundo".to_string())
        .slug("pt-ola-mundo".to_string())
        .language("pt".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 2)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let es_post = ContentBuilder::new()
        .title("Hola Mundo".to_string())
        .slug("es-hola-mundo".to_string())
        .language("es".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 3)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    data.push_content(en_post);
    data.push_content(pt_post);
    data.push_content(es_post);
    build_language_index(&mut data);

    assert_eq!(data.language.map.len(), 3);
    assert!(data.language.map.contains_key("en"));
    assert!(data.language.map.contains_key("pt"));
    assert!(data.language.map.contains_key("es"));
    assert_eq!(data.language.map["en"].len(), 1);
    assert_eq!(data.language.map["pt"].len(), 1);
    assert_eq!(data.language.map["es"].len(), 1);
}

#[test]
fn test_build_language_index_defaults_to_site_language() {
    let mut data = Data::new("language: en", Path::new("marmite.yaml"));

    let post_with_lang = ContentBuilder::new()
        .title("Post With Lang".to_string())
        .slug("post-with-lang".to_string())
        .language("en".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    let post_without_lang = ContentBuilder::new()
        .title("Post Without Lang".to_string())
        .slug("post-without-lang".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 2)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    data.push_content(post_with_lang);
    data.push_content(post_without_lang);
    build_language_index(&mut data);

    assert_eq!(data.language.map.len(), 1);
    assert!(data.language.map.contains_key("en"));
    assert_eq!(data.language.map["en"].len(), 2);
}

#[test]
fn test_build_language_index_monolingual_site() {
    let mut data = Data::new("language: en", Path::new("marmite.yaml"));

    let post = ContentBuilder::new()
        .title("Only Post".to_string())
        .slug("only-post".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 6, 15)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    data.push_content(post);
    build_language_index(&mut data);

    assert_eq!(data.language.map.len(), 1);
    assert!(data.language.map.contains_key("en"));
    assert_eq!(data.language.map["en"].len(), 1);
}

#[test]
fn test_url_collection_includes_languages() {
    let mut data = Data::new("language: en", Path::new("marmite.yaml"));

    let post = ContentBuilder::new()
        .title("Test Post".to_string())
        .slug("test-post".to_string())
        .language("en".to_string())
        .date(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .build();

    data.push_content(post);
    build_language_index(&mut data);
    data.collect_all_urls();

    assert!(data
        .generated_urls
        .languages
        .contains(&"languages.html".to_string()));
    let all = data.generated_urls.get_all_urls();
    assert!(all.contains(&"languages.html".to_string()));
}
