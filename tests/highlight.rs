use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn run_marmite(input: &std::path::Path, output: &std::path::Path) {
    let out = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            input.to_str().unwrap(),
            output.to_str().unwrap(),
        ])
        .output()
        .expect("failed to invoke marmite");
    assert!(
        out.status.success(),
        "marmite run failed.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

/// Builds a fixture with the default highlight settings.
fn build_fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("input");
    fs::create_dir_all(input.join("content")).unwrap();
    fs::write(
        input.join("marmite.yaml"),
        "name: Highlight Test\ntagline: Test",
    )
    .unwrap();
    fs::write(
        input.join("content").join("post.md"),
        r##"# Code Demo

A Rust snippet:

```rust
fn main() {
    println!("hi");
}
```

An unknown-language snippet:

```foobarlang
not a real language
```

A fence with no info string:

```
plain text with <angles> & ampersands
```
"##,
    )
    .unwrap();
    dir
}

#[test]
fn rust_fence_gets_arborium_highlighting() {
    let dir = build_fixture();
    let input = dir.path().join("input");
    let output = dir.path().join("out");
    run_marmite(&input, &output);

    let html = fs::read_to_string(output.join("post.html")).expect("post.html not generated");
    assert!(
        html.contains("<a-"),
        "expected arborium custom elements in rendered HTML, got: {html}"
    );
}

#[test]
fn arborium_css_emitted_into_static_dir() {
    let dir = build_fixture();
    let input = dir.path().join("input");
    let output = dir.path().join("out");
    run_marmite(&input, &output);

    let css_path = output.join("static").join("arborium.css");
    let css = fs::read_to_string(&css_path)
        .unwrap_or_else(|e| panic!("arborium.css missing at {}: {e}", css_path.display()));
    assert!(!css.is_empty(), "arborium.css is empty");
    assert!(
        css.contains("html[data-theme=\"light\"]"),
        "light-theme scope missing"
    );
    assert!(
        css.contains("html[data-theme=\"dark\"]"),
        "dark-theme scope missing"
    );
    assert!(
        css.contains("prefers-color-scheme"),
        "OS-preference fallback missing"
    );
}

#[test]
fn unknown_language_falls_back_to_plain() {
    let dir = build_fixture();
    let input = dir.path().join("input");
    let output = dir.path().join("out");
    run_marmite(&input, &output);

    let html = fs::read_to_string(output.join("post.html")).unwrap();
    // The fallback path escapes the source and lets comrak wrap it in <pre><code>.
    assert!(
        html.contains("not a real language"),
        "unknown-language body should be preserved verbatim"
    );

    // HTML special chars from the plain text fence must be escaped, not highlighted.
    assert!(
        html.contains("&lt;angles&gt;") && html.contains("&amp;"),
        "plain-text fence should be HTML-escaped"
    );
}

/// Builds a fixture with syntax highlighting disabled.
fn build_fixture_highlight_disabled() -> TempDir {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("input");
    fs::create_dir_all(input.join("content")).unwrap();
    fs::write(
        input.join("marmite.yaml"),
        "name: Highlight Test\ntagline: Test\ncode_highlight:\n  enabled: false",
    )
    .unwrap();
    fs::write(
        input.join("content").join("post.md"),
        "# No Highlight\n\n```rust\nfn main() {}\n```\n",
    )
    .unwrap();
    dir
}

#[test]
fn highlight_disabled_omits_css() {
    let dir = build_fixture_highlight_disabled();
    let input = dir.path().join("input");
    let output = dir.path().join("out");
    run_marmite(&input, &output);

    let html = fs::read_to_string(output.join("post.html")).expect("post.html not generated");
    assert!(
        !html.contains("arborium.css"),
        "arborium.css link should not appear when highlighting is disabled, got: {html}"
    );
    assert!(
        !output.join("static").join("arborium.css").exists(),
        "arborium.css file should not be generated when highlighting is disabled"
    );
}

#[test]
fn highlight_enabled_includes_css() {
    let dir = build_fixture();
    let input = dir.path().join("input");
    let output = dir.path().join("out");
    run_marmite(&input, &output);

    let html = fs::read_to_string(output.join("post.html")).expect("post.html not generated");
    assert!(
        html.contains("arborium.css"),
        "arborium.css link should be present when highlighting is enabled (default)"
    );
    assert!(
        output.join("static").join("arborium.css").exists(),
        "arborium.css file should be generated when highlighting is enabled (or not explicitly disabled)"
    );
}
