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

- [ ] Increase unit test coverage for core logic.
  - [ ] `content.rs`: Test content parsing, frontmatter extraction, and content classification.
  - [ ] `parser.rs`: Test Markdown parsing and HTML generation.
  - [ ] `config.rs`: Test configuration loading and default values.
- [ ] Implement integration tests for site generation.
  - [ ] Create a minimal test site fixture.
  - [ ] Write a test that generates the site and verifies the output structure and content.
  - [ ] Use `tempfile` for filesystem isolation in tests.
- [ ] Add a `mask test` command to `maskfile.md` for a consistent testing interface.

## Phase 2: Refactoring and Performance

### 2.1. Refactoring

- [ ] Refactor large functions into smaller, more focused units.
  - [ ] `site.rs`: Break down `generate()` into smaller private functions (e.g., `setup_build`, `process_content`, `render_site`).
  - [ ] `site.rs`: Refactor `collect_all_urls()` to improve readability.

### 2.2. Performance Optimization

- [ ] Optimize the `_collect_back_links` function in `site.rs`.
  - [ ] Replace the O(n^2) nested loop with a more efficient `HashMap`-based approach.

## Phase 3: Build and Automation

### 3.1. Build System Enhancements

- [ ] Enhance the `mask coverage` command in `maskfile.md`.
  - [ ] Add an option to fail the build if test coverage drops below a defined threshold.

---

This plan will be executed incrementally. Progress will be tracked by checking off the items in this document.
