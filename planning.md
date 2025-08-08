# Marmite Improvement Plan

This document outlines the plan for improving the Marmite codebase based on the analysis performed.

## Phase 1: Foundational Improvements

### 1.1. Error Handling

- [x] Replace all `.unwrap()` and `.expect()` calls with robust error handling (`?` operator or `match` statements).
  - [x] `site.rs`
  - [x] `main.rs`
  - [x] `content.rs`
  - [x] `parser.rs`
  - [x] Other modules as needed (server.rs, config.rs).
  - [x] Applied clippy pedantic fixes for better code quality.

### 1.2. Testing Strategy

- [x] Separate unit tests from source files, create a src/tests module with test files martching each of the source files
- [x] Increase unit test coverage for core logic.
  - [x] `content.rs`: Test content parsing, frontmatter extraction, and content classification.
  - [x] `parser.rs`: Test Markdown parsing and HTML generation.
  - [x] `config.rs`: Test configuration loading and default values.
- [x] Implement integration tests on tests/ for site generation.
  - [x] Create a minimal test site fixture.
  - [x] Write a test that generates the site and verifies the output structure and content.
  - [x] Use `tempfile` for filesystem isolation in tests.
- [x] Add a `mask test` command to `maskfile.md` for a consistent testing interface.
- [x] Adapt the commented out integration tests on tests/site_generation.rs to use the subprocess approach

### 1.2.1 More integration tests

- [x] Write more integration tests based on features you discover reading the documentation on example/content/*.md


## Phase 2: Refactoring and Performance

### 2.1. Refactoring

- [ ] Refactor large functions into smaller, more focused units.
  - [ ] `site.rs`: Break down `generate()` into smaller private functions (e.g., `setup_build`, `process_content`, `render_site`).
  - [ ] `site.rs`: Refactor `collect_all_urls()` to improve readability.

### 2.2. Performance Optimization

- [ ] Optimize the `_collect_back_links` function in `site.rs`.
  - [ ] Replace the O(n^2) nested loop with a more efficient `HashMap`-based approach.

---

This plan will be executed incrementally. Progress will be tracked by checking off the items in this document.
