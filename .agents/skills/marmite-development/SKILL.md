---
name: marmite-development
description: Guidelines and workflows for contributing to the marmite codebase - covers code quality, testing, architecture patterns, and contribution checklists
---

# Marmite Development Guide

This skill is for developers contributing to the marmite codebase. For building sites with marmite, use the `marmite` skill instead.

- Repository: https://github.com/rochacbruno/marmite
- Site: https://marmite.blog
- License: AGPL-3.0-or-later
- Language: Rust (edition 2021)

## Code Quality Requirements

### Formatting and Linting

Every change must pass formatting and linting before it can be committed.

```bash
# Format code (always run first)
mask fmt

# Check formatting + clippy lints
mask check

# Pedantic clippy (run before opening a PR)
mask pedantic
```

`mask check` runs `cargo fmt -- --check` followed by `cargo clippy`. Both must pass clean. `mask pedantic` adds `-W clippy::pedantic` for stricter analysis - fix pedantic issues only after confirming with the project maintainer.

Do not suppress clippy warnings with `#[allow(...)]` unless there is a justified reason. If a suppression is needed, use the narrowest possible scope (on the item, not the module).

### Safe Rust Practices

- **No `unwrap()` in non-test code.** Use `Result` and `Option` with proper error propagation (`?` operator), `unwrap_or`, `unwrap_or_else`, `unwrap_or_default`, or pattern matching. Reserve `unwrap()` for test code and cases where the value is guaranteed at compile time (e.g., embedded assets loaded via `rust_embed`).
- **No `panic!` in library code.** Return `Result` or `Option` instead. Use `error!` from the `log` crate to report problems, then return an error or a sensible default.
- **No unsafe code** unless absolutely necessary and approved by the maintainer.
- **Handle all `Result` types.** Do not silently discard errors with `let _ = ...` unless the discard is intentional and documented.
- **Prefer `&str` over `String`** in function parameters when ownership is not needed.
- **Use `Arc` for shared ownership** across threads. The project uses `Arc<PathBuf>` and `Arc<Cli>` for passing shared state.
- **Avoid `.clone()` when a borrow suffices.** Cloning is acceptable when the borrow checker requires it or when the data needs to outlive the current scope, but do not clone as a reflex.
- **Use `LazyLock` for static initialization** instead of `lazy_static!` in new code. The project is transitioning to `std::sync::LazyLock`.

### Error Handling Patterns

The project uses the `log` crate for user-facing messages:

```rust
use log::{info, warn, error, debug, trace};

// User-facing progress
info!("Processing {} files", count);

// Recoverable problems the user should know about
warn!("Skipping file with invalid frontmatter: {}", path.display());

// Fatal or important errors
error!("Failed to write output: {}", err);

// Developer debugging (visible with -vv)
debug!("Resolved slug: {}", slug);

// Detailed tracing (visible with -vvv)
trace!("Template context: {:?}", context);
```

Functions that can fail should return `Result<T, E>` or `Option<T>`. Use `process::exit(1)` only in `main.rs` for top-level CLI errors.

### Naming Conventions

- `snake_case` for functions, methods, variables, and modules
- `PascalCase` for types, structs, enums, and traits
- `SCREAMING_SNAKE_CASE` for constants and static variables
- CLI flags: use `#[arg(long)]` with descriptive names (e.g., `--skip-image-resize`, `--init-templates`). Clap automatically converts underscores to hyphens.
- Config fields: use `snake_case` matching the YAML key name. Always add `#[serde(default)]` with a default function.

## Testing

Every change needs tests. The project has two testing layers:

### Unit Tests (`src/tests/`)

Unit tests call marmite code directly. Each module has a corresponding test file.

```
src/tests/
  content.rs         # Content parsing, slug generation, streams
  shortcodes.rs      # Shortcode regex matching and expansion
  parser.rs          # Markdown to HTML conversion
  tera_functions.rs  # Custom Tera template functions
  tera_filter.rs     # Custom Tera filters
  feed.rs            # RSS feed generation
  gallery.rs         # Image gallery processing
  site.rs            # Site generation logic
  templates.rs       # Template initialization
  embedded.rs        # Embedded asset loading
  server.rs          # HTTP server
  image_resize.rs    # Image resizing
  image_provider.rs  # Image download
  theme_manager.rs   # Theme installation
```

Unit test pattern:

```rust
use super::*;
use crate::config::Marmite;

#[test]
fn test_descriptive_name_of_what_is_tested() {
    // Setup
    let config = Marmite::default();

    // Act
    let result = function_under_test(&config);

    // Assert
    assert_eq!(result, expected_value);
}
```

Run unit tests:

```bash
mask test_unit
# or a specific test
cargo test --bin marmite test_name
```

### Integration Tests (`tests/`)

Integration tests run marmite as a subprocess via `std::process::Command` and assert on the output files. They test end-to-end behavior.

```
tests/
  basic_functionality.rs       # CLI help, version, minimal site generation
  content_generation.rs        # Content processing end-to-end
  features.rs                  # Feature-specific integration tests
  streams.rs                   # Stream content type
  wikilinks_integration.rs     # Wikilink resolution
  highlight.rs                 # Syntax highlighting
  image_resize_integration.rs  # Image resizing pipeline
  themes.rs                    # Theme loading and application
```

Integration test pattern:

```rust
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_feature_name_end_to_end() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    fs::create_dir_all(input_dir.join("content")).unwrap();
    fs::write(
        input_dir.join("marmite.yaml"),
        "name: Test Site\ntagline: Test",
    ).unwrap();
    fs::write(
        input_dir.join("content").join("test.md"),
        "# Test Page\n\nContent here.",
    ).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--",
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap()])
        .output()
        .expect("Failed to execute marmite");

    assert!(output.status.success());
    // Assert on generated files
    let html = fs::read_to_string(output_dir.join("test.html")).unwrap();
    assert!(html.contains("Test Page"));
}
```

Run integration tests:

```bash
mask test_integration
# or a specific file
cargo test --test basic_functionality
```

Run all tests:

```bash
mask test
```

### What to Test

- **New features:** Both unit tests for the logic and integration tests for CLI/end-to-end behavior.
- **Bug fixes:** A test that reproduces the bug and verifies the fix. The test should fail without the fix applied.
- **Config options:** Test default values, custom values, and edge cases.
- **CLI flags:** Integration test that passes the flag and checks the result.

## Avoiding Breaking Changes

Marmite is used by people who depend on its current behavior. Follow these principles:

- **Prefer feature flags over behavior changes.** If a change alters existing behavior, add a config option to opt in. Make the current behavior the default so existing sites are unaffected.
- **New config fields must have sensible defaults.** Use `#[serde(default = "default_function")]` so that existing `marmite.yaml` files without the new field continue to work.
- **Do not remove or rename existing config fields.** If a field needs to be superseded, deprecate it (log a warning when it is used) and support both the old and new field.
- **Do not remove or rename existing CLI flags.** Use `#[arg(hide = true)]` to hide deprecated flags from `--help` while still accepting them.
- **Do not change the default output structure** (file names, directory layout) without a migration path or a feature flag.
- **Template variable changes must be backward-compatible.** Adding new variables is fine; removing or renaming existing ones breaks custom templates.

## Documentation Requirements

Every new feature must be documented in multiple places. The marmite website is generated from `example/content/`, so documentation and the site are the same thing.

### Blog post for new features

Create a new markdown file in `example/content/` documenting the feature. Follow existing patterns:

- Use a dated filename: `YYYY-MM-DD-feature-name.md`
- Include frontmatter with title, description, and relevant tags
- Write a practical guide showing how to use the feature, with examples
- If the feature adds CLI flags, also update `example/content/2024-11-26-marmite-command-line-interface.md`

### LLM documentation (`example/ai/llms.txt`)

Add an entry to `example/ai/llms.txt` under the appropriate section for any significant change: new CLI flag, new config option, new content capability, new template variable, or new shortcode. This file is served at `marmite.blog/llms.txt` and is used by AI agents to understand marmite's capabilities.

### Agent skill references (`.agents/skills/marmite/references/`)

Update the corresponding reference file when the feature changes:

| What changed | File to update |
|-------------|---------------|
| Config options | `references/config-reference.md` |
| CLI flags | `references/cli-reference.md` |
| Frontmatter fields | `references/frontmatter.md` |
| Template variables or functions | `references/tera-templates.md` |
| Shortcodes | `references/shortcodes.md` |
| Content organization (streams, series, fragments) | `references/content-organization.md` |
| Deployment or hosting | `references/deployment-guide.md` |
| Comment systems | `references/comment-system.md` |

These files are compiled into the binary via `rust_embed` and installed with `--skill-install`. They must stay accurate.

### Release notes

Every new feature and bug fix must be added to the current draft release notes. Find the latest release notes file with `stream: draft` in frontmatter under `example/content/`. The filename pattern is `YYYY-MM-DD-HH-MM-SS-marmite-X-Y-Z-release-notes.md`.

- For new features, add a section describing what the feature does and how to use it.
- For bug fixes, describe what was broken and how it is fixed.
- If no draft release notes file exists, create one following the same pattern, using the next expected version number.

## Template Changes

Marmite has two sets of templates that must be kept in sync:

### Main theme (`example/templates/` and `example/static/`)

This is the default theme used by marmite when no custom theme is set. Changes to templates, CSS, JavaScript, or static assets go here first.

- `example/templates/` - Tera HTML templates (`base.html`, `content.html`, `list.html`, `group.html`)
- `example/static/` - CSS, JavaScript, fonts, colorschemes

### Alternative theme - `theme_template` (`example/theme_template/`)

This is the scaffolding theme used when users run `marmite <folder> --start-theme <name>`. It provides a starting point for new custom themes. It has its own copies of templates and static assets:

- `example/theme_template/templates/` - Template files
- `example/theme_template/static/` - Static assets (CSS, JS, fonts, colorschemes)

**When you change any template or static asset in the main theme, you must also apply the corresponding change to `theme_template`.** The two themes can differ in styling and layout details, but they must both support the same template variables, blocks, and structural features. If a new template block or variable is added in the main theme, `theme_template` must also include it so that users who scaffold a new theme get a working starting point.

After making template changes, verify both themes work:

```bash
# Test with the main theme
mask serve

# Test with theme_template
mask serve_theme
```

## Adding New Features

### Every config parameter needs a CLI argument

When you add a new field to `marmite.yaml`, it must also be settable via a CLI flag. This allows users to override config without editing files, and is essential for CI/CD and scripting.

The full process:

1. **Add the config field** in `config.rs`:

```rust
#[serde(default = "default_my_feature")]
pub my_feature: bool,
```

Add the default function:

```rust
fn default_my_feature() -> bool {
    false
}
```

2. **Add the CLI flag** in `cli.rs` inside the `Configuration` struct:

```rust
/// Enable my feature
#[arg(long)]
pub my_feature: bool,
```

3. **Wire the override** in `config.rs` inside `Marmite::override_from_cli_args()`:

```rust
if cli_args.configuration.my_feature {
    self.my_feature = true;
}
```

4. **Document it:**
   - Add a commented-out example in `example/marmite.yaml`
   - Update `.agents/skills/marmite/references/config-reference.md`
   - Update `example/content/2024-11-26-marmite-command-line-interface.md` if it is a CLI flag

### Adding a new CLI command

1. Add the flag to `Cli` in `cli.rs` with `#[arg(long)]` and a doc comment (becomes `--help` text).
2. Add the handler in `run_cli()` in `main.rs` at the correct position in the command chain.
3. If the command does not need `input_folder`, handle it before `input_folder` resolution.
4. Update `determine_verbosity()` if the command should auto-set verbosity.
5. Update the CLI docs.

### Adding a new template function

1. Implement the function struct in `tera_functions.rs` (implement `tera::Function`).
2. Register it in `site.rs` where other functions are registered.
3. Update `.agents/skills/marmite/references/tera-templates.md`.

### Adding embedded assets

New embedded assets follow the `rust_embed` pattern in `embedded.rs`:

```rust
#[derive(Embed, Debug)]
#[folder = "$CARGO_MANIFEST_DIR/path/to/folder/"]
pub struct MyAssets;

pub static EMBEDDED_MY_ASSETS: LazyLock<Vec<(String, Vec<u8>)>> = LazyLock::new(|| {
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    for name in MyAssets::iter() {
        let file = MyAssets::get(name.as_ref())
            .expect("Failed to get embedded asset - this is a build-time error");
        files.push((name.clone().to_string(), file.data.clone().to_vec()));
    }
    files
});
```

## Dependency Management

- **Minimize new dependencies.** Marmite ships as a single binary. Every dependency adds compile time and binary size. Prefer the standard library when possible.
- **Pin major versions** in `Cargo.toml` (e.g., `serde = "1.0"`). Dependabot handles patch and minor updates.
- **Check for security advisories** with `cargo audit` before adding new crates.
- **Feature-gate heavy optional dependencies.** If a feature requires a large crate, consider making it optional behind a Cargo feature flag.

## Project Architecture

### Content processing pipeline

1. Walk the content directory (`walkdir`)
2. For each `.md` file, call `Content::from_markdown()`:
   - Extract frontmatter (`frontmatter_gen` - supports YAML/TOML/JSON)
   - Determine post vs page (date presence)
   - Detect stream from frontmatter or filename prefix
   - Generate slug from frontmatter, title, or filename
   - Convert markdown to HTML via comrak
   - Process shortcodes if enabled
3. Build taxonomy indexes (tags, authors, archive, streams, series)
4. Resolve backlinks and related content
5. Render Tera templates with content and site data
6. Write HTML output, copy static/media, resize images

### CLI command routing

`main()` parses args with `cli::Cli::parse()`, then `run_cli()` checks flags in priority order. Each handler returns early after completing its task. Commands that do not need `input_folder` are handled before the folder resolution.

### Key modules

| Module | Responsibility |
|--------|---------------|
| `cli.rs` | Clap argument definitions |
| `config.rs` | `Marmite` struct, YAML deserialization, CLI overrides |
| `content.rs` | `Content` struct, frontmatter parsing, slug generation |
| `site.rs` | Core site generation (~2000 lines), template rendering, taxonomy building |
| `templates.rs` | Template initialization, theme loading |
| `tera_functions.rs` | Custom Tera functions (`url_for`, `group`, `get_posts`, etc.) |
| `tera_filter.rs` | Custom Tera filters (`default_date_format`, `remove_draft`) |
| `shortcodes.rs` | Shortcode processing with regex pattern matching |
| `parser.rs` | Markdown to HTML conversion with comrak |
| `feed.rs` | RSS feed generation |
| `embedded.rs` | Embedded assets via `rust_embed` |
| `server.rs` | Built-in HTTP server with WebSocket live reload |
| `image_resize.rs` | Parallel image resizing with rayon |

## Contribution Checklist

### After any code change

1. `mask fmt`
2. `mask check`
3. `mask test`

### After implementing a new feature

1. Format and lint: `mask fmt` and `mask check`
2. Add tests: unit tests in `src/tests/` for module logic, integration tests in `tests/` for CLI/end-to-end behavior
3. Write a new blog post in `example/content/` documenting the feature (see "Documentation Requirements")
4. Update CLI docs in `example/content/2024-11-26-marmite-command-line-interface.md` if CLI flags changed
5. Add an entry to `example/ai/llms.txt` under the appropriate section
6. Update the draft release notes - find the file with `stream: draft` in `example/content/` and add the feature
7. Update the relevant reference files in `.agents/skills/marmite/references/` (see table in "Documentation Requirements")
8. If templates or static assets changed, apply the same changes to `example/theme_template/` (see "Template Changes")

### After implementing a bug fix

1. `mask fmt` and `mask check`
2. Add or update tests that cover the fixed behavior
3. Update the draft release notes - find the file with `stream: draft` in `example/content/` and describe what was broken and how it is fixed

### Before opening a PR

1. `mask fmt` and `mask check` - must pass clean
2. `mask test` - all tests must pass
3. `mask pedantic` - run and discuss pedantic issues with the maintainer before fixing
4. Verify the example site builds correctly: `mask serve`, check in a browser
5. If templates changed, also verify with `mask serve_theme` to test the `theme_template` variant
6. Ensure documentation is complete: blog post, llms.txt entry, skill references, draft release notes
7. Ensure no unrelated changes are staged

## Development Environment

### First-time setup

```bash
# Install mask task runner
cargo install mask

# Build and serve the example site with live reload
mask serve
```

The site runs at http://localhost:8000 with auto-rebuild on file changes.

### Useful commands

| Command | What it does |
|---------|-------------|
| `mask serve` | Build and serve example site with live reload and trace logging |
| `mask watch` | Watch for changes, rebuild without serving |
| `mask fmt` | Format code with `cargo fmt` |
| `mask check` | Check formatting + run clippy |
| `mask test` | Run all tests (unit + integration) |
| `mask test_unit` | Run unit tests only |
| `mask test_integration` | Run integration tests only |
| `mask build` | Build release binary |
| `mask pedantic` | Run clippy with pedantic warnings |
| `mask fix` | Auto-apply clippy fixes |
| `mask serve_theme` | Serve with the `theme_template` theme |
| `mask serve_site` | Serve the actual marmite.blog site locally |
| `mask coverage_llvm` | Generate HTML coverage report (requires `cargo-llvm-cov`) |

### Running specific tests

```bash
# A specific unit test by name
cargo test --bin marmite test_name_here

# A specific integration test file
cargo test --test basic_functionality

# All tests with output
cargo test -- --nocapture
```
