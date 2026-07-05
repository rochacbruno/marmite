use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_language_detection_without_config() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();

    // No languages: in config at all
    fs::write(
        input_dir.join("marmite.yaml"),
        "name: Test Blog\nlanguage: en",
    )
    .unwrap();

    // Post with language: pt frontmatter
    let pt_post =
        "---\ndate: 2024-01-01\ntitle: Conteudo\nlanguage: pt\n---\n# Conteudo em Portugues\n";
    fs::write(input_dir.join("content").join("conteudo.md"), pt_post).unwrap();

    // Default language post
    let en_post = "---\ndate: 2024-01-01\ntitle: Content\n---\n# English Content\n";
    fs::write(input_dir.join("content").join("content.md"), en_post).unwrap();

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

    // Portuguese post should be on pt stream even without languages: config
    assert!(
        output_dir.join("pt-conteudo.html").exists(),
        "pt-conteudo.html should exist"
    );
    assert!(
        output_dir.join("pt.html").exists(),
        "pt.html stream page should exist"
    );

    // English post stays on index
    let index = fs::read_to_string(output_dir.join("index.html")).unwrap();
    assert!(
        index.contains("Content"),
        "English post should be on index.html"
    );
    assert!(
        !index.contains("pt-conteudo"),
        "Portuguese post should NOT be on index.html"
    );
}

#[test]
fn test_translates_frontmatter_manual_linking() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();

    fs::write(
        input_dir.join("marmite.yaml"),
        "name: Test Blog\nlanguage: en",
    )
    .unwrap();

    // Original English post
    let en_post = "---\ndate: 2024-01-01\ntitle: Hello World\nslug: hello\n---\n# Hello\n";
    fs::write(input_dir.join("content").join("hello.md"), en_post).unwrap();

    // Portuguese translation using translates:
    let pt_post = "---\ndate: 2024-01-01\ntitle: Ola Mundo\nslug: ola\nlanguage: pt\ntranslates: hello\n---\n# Ola\n";
    fs::write(input_dir.join("content").join("ola.md"), pt_post).unwrap();

    // Spanish translation using translates:
    let es_post = "---\ndate: 2024-01-01\ntitle: Hola Mundo\nslug: hola\nlanguage: es\ntranslates: hello\n---\n# Hola\n";
    fs::write(input_dir.join("content").join("hola.md"), es_post).unwrap();

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

    // All content pages should exist
    assert!(output_dir.join("hello.html").exists());
    assert!(output_dir.join("pt-ola.html").exists());
    assert!(output_dir.join("es-hola.html").exists());

    // Original should have translation links to both
    let en_html = fs::read_to_string(output_dir.join("hello.html")).unwrap();
    assert!(
        en_html.contains("Also available in"),
        "English post should show translation links"
    );
    assert!(
        en_html.contains("pt-ola.html"),
        "English post should link to Portuguese"
    );
    assert!(
        en_html.contains("es-hola.html"),
        "English post should link to Spanish"
    );

    // Portuguese should link back to English and Spanish
    let pt_html = fs::read_to_string(output_dir.join("pt-ola.html")).unwrap();
    assert!(
        pt_html.contains("hello.html"),
        "Portuguese post should link to English original"
    );
}

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
fn test_language_frontmatter_implies_stream() {
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

    // Post with language: pt but no stream - should go to pt.html
    let pt_post =
        "---\ndate: 2024-01-01\ntitle: Conteudo\nlanguage: pt\n---\n# Conteudo em Portugues\n";
    fs::write(input_dir.join("content").join("conteudo.md"), pt_post).unwrap();

    // Default language post stays on index
    let en_post = "---\ndate: 2024-01-01\ntitle: Content\n---\n# English Content\n";
    fs::write(input_dir.join("content").join("content.md"), en_post).unwrap();

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

    // Portuguese post should be on pt.html, not index
    assert!(
        output_dir.join("pt-conteudo.html").exists(),
        "pt-conteudo.html should exist (language implies stream prefix)"
    );
    assert!(
        output_dir.join("pt.html").exists(),
        "pt.html stream page should exist"
    );

    let pt_stream = fs::read_to_string(output_dir.join("pt.html")).unwrap();
    assert!(
        pt_stream.contains("Conteudo"),
        "Portuguese post should be listed on pt.html"
    );

    let index = fs::read_to_string(output_dir.join("index.html")).unwrap();
    assert!(
        !index.contains("pt-conteudo"),
        "Portuguese post should NOT be on index.html"
    );
    assert!(
        index.contains("Content"),
        "English post should be on index.html"
    );

    // HTML lang attribute should be correct
    let pt_html = fs::read_to_string(output_dir.join("pt-conteudo.html")).unwrap();
    assert!(
        pt_html.contains("lang=\"pt\""),
        "Portuguese page should have lang=pt"
    );
}

#[test]
fn test_dated_subfolder_extracts_date() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(
        input_dir
            .join("content")
            .join("2024-06-15-dated-folder-test"),
    )
    .unwrap();

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

    // Base post - no date in frontmatter, should get it from folder
    let base = "---\ntitle: Dated Folder Test\n---\n# Test\n";
    fs::write(
        input_dir
            .join("content")
            .join("2024-06-15-dated-folder-test")
            .join("dated-folder-test.md"),
        base,
    )
    .unwrap();

    // Translation - also gets date from folder
    let pt = "---\ntitle: Teste de Pasta com Data\n---\n# Teste\n";
    fs::write(
        input_dir
            .join("content")
            .join("2024-06-15-dated-folder-test")
            .join("pt-teste-pasta-data.md"),
        pt,
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

    // Both should be posts (have dates from folder) and thus generate HTML
    assert!(
        output_dir.join("dated-folder-test.html").exists(),
        "Base post should exist"
    );
    assert!(
        output_dir.join("pt-teste-de-pasta-com-data.html").exists()
            || output_dir.join("pt-teste-pasta-data.html").exists(),
        "Portuguese translation should exist"
    );

    // Base should have date rendered (confirming date extraction worked)
    let base_html = fs::read_to_string(output_dir.join("dated-folder-test.html")).unwrap();
    assert!(
        base_html.contains("2024"),
        "Date from folder should appear in rendered content"
    );

    // Should have translation links (both are in the same subfolder)
    assert!(
        base_html.contains("Portugues"),
        "Base should link to Portuguese translation"
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

#[test]
fn test_lang_prefixed_file_without_original_is_not_translation_group() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    let myfolder = input_dir.join("content").join("myfolder");
    fs::create_dir_all(&myfolder).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Blog\nlanguage: en\n").unwrap();

    fs::write(
        myfolder.join("firstpost.md"),
        "---\ntitle: First Post\ndate: 2024-01-01\n---\n# First Post\n",
    )
    .unwrap();
    fs::write(
        myfolder.join("secondpost.md"),
        "---\ntitle: Second Post\ndate: 2024-01-02\n---\n# Second Post\n",
    )
    .unwrap();
    // Has a pt- prefix but no original matching the folder name "myfolder"
    fs::write(
        myfolder.join("pt-forthpost.md"),
        "---\ntitle: Forth Post\ndate: 2024-01-03\n---\n# Forth Post\n",
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

    // None of these should have translation links
    let first = fs::read_to_string(output_dir.join("first-post.html")).unwrap();
    assert!(
        !first.contains("content-translations"),
        "first-post should not be in a translation group"
    );

    let forth = fs::read_to_string(output_dir.join("pt-forth-post.html")).unwrap();
    assert!(
        !forth.contains("content-translations"),
        "pt-forth-post should not be in a translation group without a matching original"
    );
}

#[test]
fn test_translation_group_requires_slug_matching_folder() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    let things = input_dir.join("content").join("things");
    fs::create_dir_all(&things).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Blog\nlanguage: en\n").unwrap();

    // File slug "things" matches folder name "things" -> valid original
    fs::write(
        things.join("things.md"),
        "---\ntitle: Things\ndate: 2024-01-01\n---\n# Things\n",
    )
    .unwrap();
    fs::write(
        things.join("pt-coisas.md"),
        "---\ntitle: Coisas\ndate: 2024-01-01\n---\n# Coisas\n",
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

    let things_html = fs::read_to_string(output_dir.join("things.html")).unwrap();
    assert!(
        things_html.contains("content-translations"),
        "things.html should have translations (slug matches folder)"
    );
    assert!(
        things_html.contains("hreflang=\"pt\""),
        "things.html should link to Portuguese translation"
    );

    let coisas_html = fs::read_to_string(output_dir.join("pt-coisas.html")).unwrap();
    assert!(
        coisas_html.contains("content-translations"),
        "pt-coisas.html should have translations"
    );
    assert!(
        coisas_html.contains("hreflang=\"en\""),
        "pt-coisas.html should link back to English original"
    );
}
