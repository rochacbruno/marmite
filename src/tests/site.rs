use super::*;
use crate::content::ContentBuilder;
use chrono::NaiveDate;
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
