# Marmite Development Guide

This document is for AI agents and contributors working on the marmite codebase itself. For building sites with marmite, see the embedded skill (`marmite --skill`).

## About

Marmite (**Mar**kdown **M**akes S**ite**s) is a minimal, fast static site generator written in Rust. It takes a folder of Markdown files, combines them with Tera templates, and produces a static HTML website. Single binary, zero runtime dependencies, zero-config by default.

- Repository: https://github.com/rochacbruno/marmite
- Site: https://marmite.blog

## Tech Stack

- **Language:** Rust
- **Templating:** [Tera](https://keats.github.io/tera/docs/) (Jinja2-like syntax)
- **Markdown:** [Comrak](https://docs.rs/comrak/) (CommonMark + GFM extensions)
- **CLI:** [Clap](https://docs.rs/clap/) (derive macros)
- **Syntax Highlighting:** [Arborium](https://docs.rs/arborium/) (tree-sitter based, build-time)
- **Dev Server:** [tiny_http](https://docs.rs/tiny_http/) with live reload via [tungstenite](https://docs.rs/tungstenite/) (WebSocket)
- **Task Runner:** [Mask](https://github.com/jacobdeichert/mask) (tasks defined in `maskfile.md`)
- **Image Processing:** [image](https://docs.rs/image/) crate with [rayon](https://docs.rs/rayon/) for parallel resizing
- **Embedded Assets:** [rust_embed](https://docs.rs/rust-embed/) (templates, static files, shortcodes, agent skills compiled into binary)

## Project Structure

```
src/
  main.rs             Entry point, CLI command routing
  cli.rs              Clap argument definitions
  config.rs           Marmite struct (marmite.yaml deserialization)
  content.rs          Content struct, frontmatter parsing, slug generation
  embedded.rs         Embedded assets (templates, static, shortcodes, agent skills)
  site.rs             Core site generation logic (~2000 lines)
  templates.rs        Template initialization and theme setup
  tera_functions.rs   Custom Tera functions (url_for, group, get_posts, etc.)
  tera_filter.rs      Custom Tera filters (default_date_format, remove_draft)
  shortcodes.rs       Shortcode processing with regex pattern matching
  parser.rs           Markdown to HTML conversion with comrak
  feed.rs             RSS feed generation
  gallery.rs          Image gallery processing
  highlight.rs        Build-time syntax highlighting
  image_provider.rs   Automatic banner image download (picsum)
  image_resize.rs     Parallel image resizing with incremental builds
  server.rs           Built-in HTTP server with WebSocket live reload
  theme_manager.rs    Remote theme download and installation
  re.rs               Shared regex patterns
  tests/              Unit tests (one file per module, calls code directly)

tests/                Integration tests (runs marmite as a subprocess via process::Command)

example/              Complete working example site - primary dev/test target
  marmite.yaml        Example configuration
  content/            Markdown files (posts, pages, _ prefixed fragments)
  templates/          Tera HTML templates
  static/             CSS, JS, fonts, colorschemes
  shortcodes/         Built-in shortcode definitions (Tera macros)
  theme_template/     Default theme scaffold used by --start-theme
  ai/llms.txt         LLM-readable documentation index

.agents/              Embedded agent skill files (compiled into binary via rust_embed)
  skills/marmite/
    SKILL.md           Main skill document with workflows
    references/        Detailed reference files (config, CLI, templates, etc.)
```

## Development Workflow

### First-time setup

```bash
# Install mask task runner (if not already installed)
cargo install mask

# Install pre-commit hook (runs mask pedantic before every commit)
mask install_hook

# Build and serve the example site with full trace logging and live reload
mask serve
```

The site runs at http://localhost:8000 with auto-rebuild on file changes. Edit source in `src/` or content in `example/` and it rebuilds automatically.

### Everyday commands

| Command | What it does |
|---------|-------------|
| `mask serve` | Build and serve the example site with live reload and full trace logging |
| `mask watch` | Watch for changes, rebuild without serving |
| `mask fmt` | Format code with `cargo fmt` |
| `mask check` | Check formatting + run clippy |
| `mask test` | Run all tests (unit + integration) |
| `mask test_unit` | Run unit tests only (`cargo test --bin marmite`) |
| `mask test_integration` | Run integration tests only (`cargo test --test '*'`) |
| `mask build` | Build release binary |
| `mask pedantic` | Run clippy with pedantic warnings |
| `mask install_hook` | Install pre-commit hook that runs `mask pedantic` |
| `mask fix` | Auto-apply clippy fixes |
| `mask pedantic_fix` | Auto-apply clippy pedantic fixes |

### Running specific tests

```bash
# A specific unit test by name
cargo test --bin marmite test_embedded_agent_skills

# A specific integration test file
cargo test --test basic_functionality

# All tests with output
cargo test -- --nocapture
```

### Serving with a theme

```bash
# Serve with the theme_template theme
mask serve_theme

# Serve the actual marmite.blog site locally
mask serve_site
```

## Code Conventions

- Follow `rustfmt` defaults. Always run `mask fmt` before committing.
- Follow `clippy` defaults. Always run `mask check` to verify.
- Use `Result` and `Option` for error handling. Avoid `unwrap()` in non-test code.
- Use `log` macros (`info!`, `error!`, `warn!`) for user-facing output.
- Standard Rust naming: `snake_case` for functions/variables, `PascalCase` for types.
- CLI flags use `#[arg(long)]` with doc comments that become help text.
- Configuration fields go in the `Marmite` struct in `config.rs` with `#[serde(default)]` and a default function.
- New embedded assets follow the `rust_embed` derive pattern in `embedded.rs`.

## Checklists

### After any code change

1. `mask fmt`
2. `mask check`
3. `mask test`

### After implementing a new feature

1. **Format and lint:** `mask fmt` and `mask check`.
2. **Add tests:** Unit tests in `src/tests/` for module logic, integration tests in `tests/` for CLI/end-to-end behavior.
3. **Update example content:** If the feature adds or changes user-facing behavior, update or add the relevant docs in `example/content/*.md`. Follow existing patterns - dated files for feature guides, undated files for reference pages.
4. **Update the CLI docs:** If CLI flags were added or changed, update `example/content/2024-11-26-marmite-command-line-interface.md` - both the feature section and the `--help` output block at the bottom.
5. **Update llms.txt:** If the feature is significant (new CLI flag, new config option, new content capability), add an entry to `example/ai/llms.txt` under the appropriate section.
6. **Update the draft release notes:** Find the latest release notes file with `stream: draft` in `example/content/` and add the feature. If no draft exists, create one following the pattern `YYYY-MM-DD-HH-MM-SS-marmite-X-Y-Z-release-notes.md`.
7. **Update embedded skill references:** If the feature changes config options, CLI flags, frontmatter fields, template variables, or shortcodes, update the corresponding reference file in `.agents/skills/marmite/references/`. These are compiled into the binary.

### After implementing a bug fix

1. `mask fmt` and `mask check`.
2. Add or update tests that cover the fixed behavior.
3. Update the draft release notes with a description of what was broken and how it's fixed.

### Before opening a PR

1. `mask fmt` and `mask check` - must pass clean.
2. `mask test` - all tests must pass.
3. `mask pedantic` - runs clippy with pedantic warnings. Ask the user if they want to fix pedantic issues before proceeding. Do not auto-fix pedantic issues without confirmation.
4. Verify the example site builds and looks correct: `mask serve`, then check in a browser.
5. Ensure no unrelated changes are staged.

## Key Architectural Patterns

### CLI command routing

`main()` parses args with `cli::Cli::parse()`, then `run_cli()` checks flags in order:

1. `--skill` (no input folder needed, print and exit)
2. `--skill-install` / `--skill-install-claude` (defaults to CWD)
3. Resolve `input_folder` (required for all remaining commands)
4. `--init-site`, `--new`, `--init-templates`, `--start-theme`, `--set-theme`, `--generate-config`, `--shortcodes`, `--show-urls`
5. Default: `site::generate()` with optional `--serve` and `--watch`

Each handler returns early after completing its task.

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

### Embedded assets pattern

Templates, static files, shortcodes, and agent skills are compiled into the binary via `rust_embed`:

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

### Adding a new config field

1. Add the field to `Marmite` struct in `config.rs` with `#[serde(default)]` (and a default function if the default is not the type's `Default`).
2. Add a CLI override flag in the `Configuration` struct in `cli.rs`.
3. Add the override mapping in `Marmite::override_from_cli_args()` in `config.rs`.
4. Use the field in the relevant module.
5. Update `example/marmite.yaml` with a commented-out example.
6. Update `.agents/skills/marmite/references/config-reference.md`.

### Adding a new CLI command

1. Add the flag to `Cli` struct in `cli.rs` with `#[arg(long)]`.
2. Add the handler in `run_cli()` in `main.rs` at the appropriate position in the chain.
3. If the command doesn't need `input_folder`, handle it before the `input_folder` resolution.
4. Update `determine_verbosity()` if the command should auto-bump verbosity.
5. Update the CLI docs in `example/content/2024-11-26-marmite-command-line-interface.md`.

### Adding a new template function

1. Implement the function struct in `tera_functions.rs` (implement `tera::Function`).
2. Register it in `site.rs` where other functions are registered.
3. Update `.agents/skills/marmite/references/tera-templates.md`.

## Version Management

```bash
# Bump to release version
mask bumpversion 0.3.2

# Tag and push
mask pushtag 0.3.2

# Or do both
mask publish 0.3.2

# Then bump to next dev version
mask bumpversion 0.3.3-dev
```

## Code Coverage

```bash
# Generate HTML coverage report (requires cargo-llvm-cov)
mask coverage_llvm

# Generate cobertura.xml (requires cargo-tarpaulin on nightly)
mask coverage
```

## Python Package

Marmite is also published on PyPI via maturin:

```bash
mask build_python         # Build wheel
mask python_dev_install   # Install in dev mode
```
