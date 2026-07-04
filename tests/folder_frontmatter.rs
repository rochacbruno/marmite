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
