# Project Context: Marmite

This document outlines the essential commands and guidelines for developing on the Marmite static site generator.

## About the Project

Marmite is a minimal, fast, and flexible static site generator written in Rust. It takes Markdown files as input, combines them with Tera templates, and generates a static HTML website. It includes a live-reloading development server for a smooth workflow.

## Tech Stack

- **Language:** Rust
- **Templating Engine:** [Tera](https://tera.netlify.app/) (Jinja2-like syntax)
- **Markdown Parsing:** [Comrak](https://docs.rs/comrak/latest/comrak/)
- **CLI Framework:** [Clap](https://crates.io/crates/clap)
- **Development Server:** [tiny_http](https://crates.io/crates/tiny_http)
- **Task Runner:** [Mask](https://github.com/jacobdeichert/mask) (via `maskfile.md`)

## Project Structure

- `src/`: Contains all the core Rust source code for the generator.
  - `main.rs`: Application entry point and CLI command handling.
  - `cli.rs`: Defines the command-line interface structure using Clap.
  - `config.rs`: Handles the configuration from the `marmite.yaml` file.
  - `content.rs`: Handles parsing and management of Markdown content.
  - `embedded.rs`: Manages embedded assets like templates and static files.
  - `feed.rs`: Generates the RSS feed.
  - `parser.rs`: Contains the logic for parsing Markdown files and frontmatter.
  - `server.rs`: Implements the live-reloading development server.
  - `site.rs`: Core logic for site generation, processing content and templates.
  - `templates.rs`: Manages Tera templating.
  - `tera_filter.rs`: Defines custom filters for the Tera templating engine.
  - `tera_functions.rs`: Defines custom functions for the Tera templating engine.
- `example/`: A complete, working example of a Marmite site. This is the primary directory for testing and development.
  - `marmite.yaml`: The main configuration file for a Marmite project.
  - `content/`: Source Markdown files for the site.
  - `templates/`: Tera HTML templates.
  - `static/`: Static assets like CSS, JavaScript, and images.
- `maskfile.md`: Defines common development tasks and commands, acting as a Makefile.

## Key Commands

The project uses `mask` as a command runner.

- `mask --help`: Lists all available commands.
- `mask build`: Compiles the `marmite` binary in release mode.
- `mask serve`: Builds and runs the example site with a development server at `http://127.0.0.1:8000`. This is the most common command for development.
- `mask check`: Runs `cargo fmt -- --check` and `cargo clippy` to check for style and correctness issues.
- `mask fmt`: Formats the code using `cargo fmt`.
- `mask fix`: Applies clippy's automatic fixes.
- `mask watch`: Watches for changes and rebuilds the site.
- `mask pedantic`: Runs clippy with pedantic warnings.
- `mask pedantic_fix`: Applies clippy's automatic fixes with pedantic warnings.
- `mask bumpversion (tag)`: Bumps the version in `Cargo.toml`.
- `mask pushtag (tag)`: Pushes a new tag to the repository.
- `mask publish (tag)`: Publishes a new version by running `bumpversion` and `pushtag`.

To work on the project, first run `mask watch` in a terminal. Then, you can modify the source code in `src/` or the example site in `example/`. The server will automatically rebuild and reload the site on changes, open the site on a browser from `example/site` or use any local webserver such as the Live Server on your IDE.

### Development Commands

Marmite uses `mask` as a task runner.

- `mask serve`: Build and serve the example site with live reload at `http://localhost:8000`. This is the primary command for development.
- `mask watch`: Watch for changes and rebuild the example site without serving.
- `mask check`: Run code formatting checks and clippy (Rust linter).
- `mask fmt`: Format the code using `cargo fmt`.
- `mask fix`: Automatically apply clippy's lint suggestions.
- `mask build`: Build the release binary.
- `cargo test`: Run the test suite. (Inferred, as no specific test command is in `maskfile.md`)

To run a single test, use: `cargo test --test <test_name>`

### Code Style Guidelines

- **Formatting**: Adhere to the default Rust formatting (`rustfmt`). Run `mask fmt` before committing.
- **Imports**: Organize imports according to Rust conventions. Unused imports should be removed.
- **Types**: Use static typing and leverage Rust's type system for safety and clarity.
- **Naming**: Follow Rust's naming conventions (e.g., `snake_case` for variables and functions, `PascalCase` for types).
- **Error Handling**: Use `Result` and `Option` for error handling. Avoid `unwrap()` in production code.
- **Templates**: Use Tera templates for HTML generation. See `CLAUDE.md` for specific Tera syntax rules.
