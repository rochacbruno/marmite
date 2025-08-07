## Marmite Development

This document outlines the essential commands and guidelines for developing on the Marmite static site generator.

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
