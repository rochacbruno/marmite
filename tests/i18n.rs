use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_language_streams_subfolder() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content").join("hello")).unwrap();

    let config = r#"
name: Test Blog
language: pt
languages:
  pt:
    name: "Portugues"
  en:
    name: "English"
  es:
    name: "Espanol"
"#;
    fs::write(input_dir.join("marmite.yaml"), config).unwrap();

    // Base content (default language pt) in subfolder
    let base_post = "---\ndate: 2024-01-01\ntitle: Ola Mundo\nslug: hello\n---\n# Ola Mundo\n";
    fs::write(
        input_dir.join("content").join("hello").join("hello.md"),
        base_post,
    )
    .unwrap();

    // English translation
    let en_post = "---\ndate: 2024-01-01\ntitle: Hello World\n---\n# Hello World\n";
    fs::write(
        input_dir
            .join("content")
            .join("hello")
            .join("en-hello-world.md"),
        en_post,
    )
    .unwrap();

    // Spanish translation
    let es_post = "---\ndate: 2024-01-01\ntitle: Hola Mundo\n---\n# Hola Mundo\n";
    fs::write(
        input_dir
            .join("content")
            .join("hello")
            .join("es-hola-mundo.md"),
        es_post,
    )
    .unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute marmite");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Base content page should exist
    assert!(
        output_dir.join("hello.html").exists(),
        "hello.html should exist"
    );

    // Translation pages should exist with language prefix in slug
    assert!(
        output_dir.join("en-hello-world.html").exists(),
        "en-hello-world.html should exist"
    );
    assert!(
        output_dir.join("es-hola-mundo.html").exists(),
        "es-hola-mundo.html should exist"
    );

    // Language stream pages should exist
    assert!(
        output_dir.join("en.html").exists(),
        "en.html stream page should exist"
    );
    assert!(
        output_dir.join("es.html").exists(),
        "es.html stream page should exist"
    );

    // Base content should have translation links
    let base_html = fs::read_to_string(output_dir.join("hello.html")).unwrap();
    assert!(
        base_html.contains("Also available in"),
        "Base content should show translation links"
    );
    assert!(
        base_html.contains("English"),
        "Base content should link to English translation"
    );
    assert!(
        base_html.contains("en-hello-world.html"),
        "Base content should link to English slug"
    );

    // English translation should have hreflang tags
    let en_html = fs::read_to_string(output_dir.join("en-hello-world.html")).unwrap();
    assert!(
        en_html.contains("hreflang"),
        "English page should have hreflang tags"
    );

    // Base content should be on index (default language)
    let index_html = fs::read_to_string(output_dir.join("index.html")).unwrap();
    assert!(
        index_html.contains("Ola Mundo"),
        "Index should contain default language content"
    );
}

#[test]
fn test_language_streams_mixed_flat_subfolder() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content").join("hello")).unwrap();

    let config = r#"
name: Test Blog
language: en
languages:
  en:
    name: "English"
  pt:
    name: "Portugues"
"#;
    fs::write(input_dir.join("marmite.yaml"), config).unwrap();

    // Flat file (existing site pattern)
    let base_post = "---\ndate: 2024-01-01\ntitle: Hello\nslug: hello\n---\n# Hello\n";
    fs::write(input_dir.join("content").join("hello.md"), base_post).unwrap();

    // Translation in subfolder matching slug
    let pt_post = "---\ndate: 2024-01-01\ntitle: Ola\n---\n# Ola\n";
    fs::write(
        input_dir.join("content").join("hello").join("pt-ola.md"),
        pt_post,
    )
    .unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute marmite");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_dir.join("hello.html").exists());
    assert!(output_dir.join("pt-ola.html").exists());

    // Both should have translation links
    let base_html = fs::read_to_string(output_dir.join("hello.html")).unwrap();
    assert!(
        base_html.contains("Portugues"),
        "Base should link to Portuguese translation"
    );

    let pt_html = fs::read_to_string(output_dir.join("pt-ola.html")).unwrap();
    assert!(
        pt_html.contains("English"),
        "Portuguese page should link to English original"
    );
}

#[test]
fn test_language_streams_frontmatter_only() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();

    let config = r#"
name: Test Blog
language: en
languages:
  en:
    name: "English"
  pt:
    name: "Portugues"
"#;
    fs::write(input_dir.join("marmite.yaml"), config).unwrap();

    // English post with explicit translation reference
    let en_post =
        "---\ndate: 2024-01-01\ntitle: Hello\nslug: hello\ntranslations:\n  - pt-ola\n---\n# Hello\n";
    fs::write(input_dir.join("content").join("hello.md"), en_post).unwrap();

    // Portuguese post using stream marker
    let pt_post = "---\ndate: 2024-01-01\ntitle: Ola\n---\n# Ola\n";
    fs::write(input_dir.join("content").join("pt-S-ola.md"), pt_post).unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute marmite");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_dir.join("hello.html").exists());
    assert!(output_dir.join("pt-ola.html").exists());
    assert!(output_dir.join("pt.html").exists());

    // English post should have translation link
    let en_html = fs::read_to_string(output_dir.join("hello.html")).unwrap();
    assert!(
        en_html.contains("Portugues") || en_html.contains("pt-ola.html"),
        "English post should reference Portuguese translation"
    );

    // Portuguese post should have bidirectional link back
    let pt_html = fs::read_to_string(output_dir.join("pt-ola.html")).unwrap();
    assert!(
        pt_html.contains("English") || pt_html.contains("hello.html"),
        "Portuguese post should reference English original"
    );
}

#[test]
fn test_no_languages_backward_compat() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();

    // No languages configured
    fs::write(input_dir.join("marmite.yaml"), "name: Test Blog").unwrap();

    let post = "---\ndate: 2024-01-01\ntitle: Hello\n---\n# Hello\n";
    fs::write(input_dir.join("content").join("hello.md"), post).unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute marmite");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let html = fs::read_to_string(output_dir.join("hello.html")).unwrap();
    // No translation links should appear
    assert!(
        !html.contains("Also available in"),
        "No translation links without languages config"
    );
    // No hreflang tags
    assert!(
        !html.contains("hreflang"),
        "No hreflang tags without languages config"
    );
}
