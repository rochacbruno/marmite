use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_subfolder_with_frontmatter_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(input_dir.join("content").join("python")).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Blog").unwrap();

    fs::write(
        input_dir
            .join("content")
            .join("python")
            .join("frontmatter.yaml"),
        "date: 2026-01-01\nstream: python\ntags:\n  - python\n  - programming\n",
    )
    .unwrap();

    fs::write(
        input_dir
            .join("content")
            .join("python")
            .join("databases.md"),
        "---\ntitle: Databases\n---\n# Databases\n\nUsing databases with Python.\n",
    )
    .unwrap();

    fs::write(
        input_dir.join("content").join("python").join("classes.md"),
        "---\ntitle: Classes\n---\n# Classes\n\nPython classes tutorial.\n",
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

    assert!(
        output_dir.join("python-databases.html").exists(),
        "databases page should be rendered with python stream prefix"
    );
    assert!(
        output_dir.join("python-classes.html").exists(),
        "classes page should be rendered with python stream prefix"
    );
    assert!(
        output_dir.join("python.html").exists(),
        "python stream page should exist"
    );

    let databases_html = fs::read_to_string(output_dir.join("python-databases.html")).unwrap();
    assert!(
        databases_html.contains("Databases"),
        "databases page should contain its title"
    );
}

#[test]
fn test_subfolder_file_overrides_folder_defaults() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(input_dir.join("content").join("python")).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Blog").unwrap();

    fs::write(
        input_dir
            .join("content")
            .join("python")
            .join("frontmatter.yaml"),
        "date: 2026-01-01\nstream: python\n",
    )
    .unwrap();

    fs::write(
        input_dir.join("content").join("python").join("special.md"),
        "---\ntitle: Special\nstream: advanced\n---\n# Special\n\nAdvanced topic.\n",
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

    assert!(
        output_dir.join("advanced-special.html").exists(),
        "file should use its own stream override, not the folder default"
    );
    assert!(
        !output_dir.join("python-special.html").exists(),
        "file should not use the folder default stream"
    );
}

#[test]
fn test_root_frontmatter_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(input_dir.join("content")).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Blog").unwrap();

    fs::write(
        input_dir.join("content").join("frontmatter.yaml"),
        "authors:\n  - admin\n",
    )
    .unwrap();

    fs::write(
        input_dir.join("content").join("2026-01-01-post.md"),
        "---\ntitle: A Post\n---\n# A Post\n\nContent.\n",
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

    assert!(
        output_dir.join("a-post.html").exists(),
        "post should be rendered"
    );
    assert!(
        output_dir.join("author-admin.html").exists(),
        "author page should be generated from root frontmatter defaults"
    );
}

#[test]
fn test_root_and_subfolder_frontmatter_layering() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(input_dir.join("content").join("python")).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Blog").unwrap();

    fs::write(
        input_dir.join("content").join("frontmatter.yaml"),
        "authors:\n  - admin\ndate: 2026-01-01\n",
    )
    .unwrap();

    fs::write(
        input_dir
            .join("content")
            .join("python")
            .join("frontmatter.yaml"),
        "stream: python\ntags:\n  - python\n",
    )
    .unwrap();

    fs::write(
        input_dir.join("content").join("python").join("intro.md"),
        "---\ntitle: Python Intro\n---\n# Python Intro\n\nIntro content.\n",
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

    assert!(
        output_dir.join("python-python-intro.html").exists(),
        "post should use python stream from subfolder frontmatter"
    );
    assert!(
        output_dir.join("python.html").exists(),
        "python stream page should exist"
    );
    assert!(
        output_dir.join("author-admin.html").exists(),
        "author from root frontmatter should be inherited through subfolder"
    );
}

#[test]
fn test_translation_group_without_frontmatter_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(input_dir.join("content").join("hello")).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Blog\nlanguage: en\n").unwrap();

    fs::write(
        input_dir.join("content").join("hello").join("en-hello.md"),
        "---\ntitle: Hello\ndate: 2026-01-01\n---\n# Hello\n\nHello world.\n",
    )
    .unwrap();

    fs::write(
        input_dir.join("content").join("hello").join("pt-ola.md"),
        "---\ntitle: Ola\ndate: 2026-01-01\n---\n# Ola\n\nOla mundo.\n",
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

    let has_en =
        output_dir.join("en-hello.html").exists() || output_dir.join("hello.html").exists();
    let has_pt = output_dir.join("pt-ola.html").exists() || output_dir.join("ola.html").exists();

    assert!(
        has_en,
        "English translation should be rendered even without frontmatter.yaml"
    );
    assert!(
        has_pt,
        "Portuguese translation should be rendered even without frontmatter.yaml"
    );
}

#[test]
fn test_empty_frontmatter_yaml_enables_subfolder() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(input_dir.join("content").join("notes")).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Blog").unwrap();

    fs::write(
        input_dir
            .join("content")
            .join("notes")
            .join("frontmatter.yaml"),
        "",
    )
    .unwrap();

    fs::write(
        input_dir
            .join("content")
            .join("notes")
            .join("2026-01-01-first-note.md"),
        "---\ntitle: First Note\n---\n# First Note\n\nA note.\n",
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

    assert!(
        output_dir.join("first-note.html").exists(),
        "empty frontmatter.yaml should still enable the subfolder for rendering"
    );
}

#[test]
fn test_nested_subfolder_frontmatter_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(input_dir.join("content").join("poetry").join("love")).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Blog").unwrap();

    fs::write(
        input_dir
            .join("content")
            .join("poetry")
            .join("love")
            .join("frontmatter.yaml"),
        "date: 2026-01-01\nstream: poetry\ntags:\n  - love\n  - poetry\n",
    )
    .unwrap();

    fs::write(
        input_dir
            .join("content")
            .join("poetry")
            .join("love")
            .join("roses.md"),
        "---\ntitle: Roses\n---\n# Roses\n\nRoses are red.\n",
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

    assert!(
        output_dir.join("poetry-roses.html").exists(),
        "nested subfolder content should render with inherited stream"
    );
}

#[test]
fn test_nested_frontmatter_inheritance_from_ancestor() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(input_dir.join("content").join("poetry").join("love")).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Blog").unwrap();

    // Only parent has frontmatter.yaml, not the nested subfolder
    fs::write(
        input_dir
            .join("content")
            .join("poetry")
            .join("frontmatter.yaml"),
        "date: 2026-01-01\nstream: poetry\n",
    )
    .unwrap();

    fs::write(
        input_dir
            .join("content")
            .join("poetry")
            .join("love")
            .join("sonnet.md"),
        "---\ntitle: Sonnet\n---\n# Sonnet\n\nA love sonnet.\n",
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

    assert!(
        output_dir.join("poetry-sonnet.html").exists(),
        "file in nested subfolder should inherit from ancestor frontmatter.yaml"
    );
}

#[test]
fn test_three_level_frontmatter_layering() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(input_dir.join("content").join("tutorials").join("python")).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Blog").unwrap();

    // Root defaults
    fs::write(
        input_dir.join("content").join("frontmatter.yaml"),
        "authors:\n  - admin\ndate: 2026-01-01\n",
    )
    .unwrap();

    // Level 1 adds stream
    fs::write(
        input_dir
            .join("content")
            .join("tutorials")
            .join("frontmatter.yaml"),
        "stream: tutorial\n",
    )
    .unwrap();

    // Level 2 adds tags
    fs::write(
        input_dir
            .join("content")
            .join("tutorials")
            .join("python")
            .join("frontmatter.yaml"),
        "tags:\n  - python\n",
    )
    .unwrap();

    fs::write(
        input_dir
            .join("content")
            .join("tutorials")
            .join("python")
            .join("basics.md"),
        "---\ntitle: Python Basics\n---\n# Python Basics\n\nLearn Python.\n",
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

    // Stream from level 1 should be inherited through level 2
    assert!(
        output_dir.join("tutorial-python-basics.html").exists(),
        "three-level layering should work: root (author/date) + tutorials (stream) + python (tags)"
    );
    // Author from root should be inherited through all levels
    assert!(
        output_dir.join("author-admin.html").exists(),
        "root author should be inherited through nested frontmatter layers"
    );
}

#[test]
fn test_nested_translation_group() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(input_dir.join("content").join("poetry").join("love")).unwrap();
    fs::create_dir_all(input_dir.join("content").join("poetry").join("nature")).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Blog\nlanguage: en\n").unwrap();

    // Love subfolder: translation group
    // Default language (en) file has no prefix
    fs::write(
        input_dir
            .join("content")
            .join("poetry")
            .join("love")
            .join("love-poem.md"),
        "---\ntitle: Love Poem\ndate: 2026-01-01\n---\n# Love Poem\n\nA love poem.\n",
    )
    .unwrap();

    fs::write(
        input_dir
            .join("content")
            .join("poetry")
            .join("love")
            .join("pt-poema-amor.md"),
        "---\ntitle: Poema de Amor\ndate: 2026-01-01\n---\n# Poema de Amor\n\nUm poema de amor.\n",
    )
    .unwrap();

    // Nature subfolder: separate translation group
    fs::write(
        input_dir
            .join("content")
            .join("poetry")
            .join("nature")
            .join("nature-poem.md"),
        "---\ntitle: Nature Poem\ndate: 2026-01-01\n---\n# Nature Poem\n\nA nature poem.\n",
    )
    .unwrap();

    fs::write(
        input_dir
            .join("content")
            .join("poetry")
            .join("nature")
            .join("pt-poema-natureza.md"),
        "---\ntitle: Poema da Natureza\ndate: 2026-01-01\n---\n# Poema da Natureza\n\nUm poema da natureza.\n",
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

    // Default language (en) renders without prefix, translations with lang prefix
    let love_en = output_dir.join("love-poem.html").exists();
    let love_pt = output_dir.join("pt-poema-de-amor.html").exists();
    let nature_en = output_dir.join("nature-poem.html").exists();
    let nature_pt = output_dir.join("pt-poema-da-natureza.html").exists();

    assert!(
        love_en,
        "English love poem should be rendered without lang prefix"
    );
    assert!(love_pt, "Portuguese love poem should be rendered");
    assert!(
        nature_en,
        "English nature poem should be rendered without lang prefix"
    );
    assert!(nature_pt, "Portuguese nature poem should be rendered");
}
