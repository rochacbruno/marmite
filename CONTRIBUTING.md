# Contributing to Marmite Static Site Generator

Thank you for considering contributing to the Marmite Site Generator project! Contributions are what make this project strong, and any help you can offer is highly appreciated. Below are the guidelines for contributing to the project.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Prerequisites](#prerequisites)
3. [How to Contribute](#how-to-contribute)
4. [Pull Requests](#pull-requests)
5. [Commit Messages](#commit-messages)
6. [Code Quality](#code-quality)
7. [Releasing](#releasing)

## Code of Conduct

As contributors, maintainers, and participants in this project, we pledge to foster an open, inclusive, and respectful environment. We are committed to ensuring that everyone who participates in the project, whether through reporting issues, submitting code, or engaging in discussions, feels safe and welcome. We are dedicated to making participation in this project harassment-free for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, nationality, race, religion, sexual orientation, or any other attribute of diversity. Examples of behavior that contribute to creating a positive environment include, but are not limited to:

- Showing empathy and kindness towards others
- Being respectful of differing opinions, experiences, and viewpoints
- Offering and accepting constructive feedback graciously
- Owning up to mistakes, apologizing when necessary, and learning from them
- Focusing on what is best for the community as a whole
- Using inclusive and welcoming language

Examples of unacceptable behavior include, but are not limited to:

- The use of sexualized language or imagery
- Personal attacks, trolling, insulting, or derogatory comments
- Public or private harassment in any form
- Publishing others’ private information without explicit permission
- Violence, threats of violence, or encouraging violent behavior
- Unwelcome physical or sexual attention
- Stalking or following someone without consent
- Any other behavior which would be deemed inappropriate in a professional setting

Project maintainers have the right and responsibility to remove, edit, or reject comments, commits, code, issues, and other contributions that do not align with this Code of Conduct. Violators may be temporarily or permanently banned from the project based on the severity of the infraction.

## Prerequisites

Before contributing, please ensure that you meet the following prerequisites:

1. Rust Installed: Make sure you have Rust installed on your machine, as the project is written in Rust. Run the following command to check your installation:

```bash
rustc --version
```

Visit the rust page for more information [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

## How to Contribute

Things to know prior to submitting code:

- All code submissions are done through pull requests against the `main` branch.
- Take care to make sure no merge commits are in the submission, and use git rebase vs git merge for this reason.

There are several ways to contribute to this project:

- **Improving the code**
- **Reporting bugs**
- **Suggesting new features**
- **Improving documentation**
- **Submitting patches**

When contributing code, it’s always a good idea to open an issue first to discuss the changes you'd like to make. It helps maintainers and other contributors align and provide feedback early. Access [Good first issue](https://github.com/rochacbruno/marmite/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) which are easy problems to solve for anyone who wants to start collaborating with the project.


## Pull Requests

1. Fork the repository and create your branch from `main`.
2. If you've added code that should be tested, add tests.
3. Ensure your code follows the existing code style.
4. Submit your pull request, linking it to the related issue if applicable.

### Commit Messages

Your commit messages should be descriptive and concise. Use the following format:

```bash
fix: Corrected YAML parsing error when loading the configuration
feat: Added support for multiple markdown templates
```

### Code Quality

Before pushing your changes ensure it meets the minimal code quality.

1. Format the code **Required**

```bash
cargo fmt
```

2. Check clippy suggestions **Required**

```bash
cargo clippy
```

2. Apply clippy fixes **optional**

```bash
cargo clippy --fix
```

#### Mask

There is a `maskfile.md` in the root of repo.
You can use it for running checks with [mask](https://crates.io/crates/mask).

```bash
cargo install mask
mask check

# Ensure your changes are committed before running.
mask fix

# If you have free time :)
mask pedantic 
mask pedantic_fix
```

## Releasing

Marmite is published to three registries: **GitHub Releases** (pre-built binaries), **crates.io** (Rust crate), and **PyPI** (Python package). The version in `Cargo.toml` is the single source of truth — `pyproject.toml` reads from it dynamically.

### Prerequisites for Releasing

- [mask](https://crates.io/crates/mask) task runner installed (`cargo install mask`)
- [cargo-edit](https://crates.io/crates/cargo-edit) installed (`cargo install cargo-edit`) for `cargo set-version`
- Push access to the repository
- For crates.io: `cargo login` with a valid API token
- For PyPI: the `PYPI_API_TOKEN` secret configured in the repository settings

### Step 1: Bump the Version and Tag

The `mask publish` command handles version bumping and tagging in one step:

```bash
mask publish 0.3.0
```

This runs two sub-commands:

1. **`mask bumpversion 0.3.0`** — Updates the version in `Cargo.toml`, regenerates `Cargo.lock`, runs `cargo fmt`, and commits the change.
2. **`mask pushtag 0.3.0`** — Creates an annotated git tag and pushes it to the remote.

You can also run these separately if you need more control:

```bash
mask bumpversion 0.3.0
# review the commit, make adjustments if needed
mask pushtag 0.3.0
```

### Step 2: GitHub Releases (automatic)

Pushing any tag triggers the [`build-release.yml`](.github/workflows/build-release.yml) workflow, which:

- Builds release binaries for 5 platforms (macOS ARM64, macOS x86_64, Linux musl, Linux gnu, Windows)
- Generates SHA256 checksums
- Creates a GitHub Release with all binaries attached
- Tags ending in `-pre` (e.g. `0.3.0-pre`) are marked as pre-releases

The [`container.yml`](.github/workflows/container.yml) workflow also triggers on tag push, building and publishing a Docker image to `ghcr.io`.

### Step 3: Publish to crates.io (manual)

Publishing to crates.io is done manually after the tag is pushed:

```bash
cargo publish --locked
```

You can verify what would be published first with:

```bash
cargo publish --dry-run --locked
```

### Step 4: Publish to PyPI (automatic)

Pushing a tag prefixed with `py` (e.g. `py0.3.0`) triggers the [`release-python.yml`](.github/workflows/release-python.yml) workflow, which:

- Runs tests, `cargo fmt`, and `cargo clippy`
- Builds Python wheels for 5 platforms using [maturin](https://github.com/PyO3/maturin) (with zig for cross-compilation)
- Builds a source distribution (`sdist`)
- Publishes wheels and sdist to PyPI via `twine`

To trigger a PyPI release after the Rust release:

```bash
git tag -a "py0.3.0" -m "chore: push py0.3.0"
git push origin py0.3.0
```

The workflow can also be triggered manually via `workflow_dispatch` with options for dry-run mode and skipping PyPI or crates.io publishing.

### Fixing a Bad Release

If you need to re-tag a release (e.g. a last-minute fix):

```bash
mask retag 0.3.0
```

This deletes the remote tag, amends the current commit, force-pushes the branch, and re-creates the tag. Use with caution as it rewrites history.

### Release Checklist

1. Ensure all tests pass: `mask test`
2. Ensure code quality: `mask check`
3. Bump version and tag: `mask publish <version>`
4. Verify the GitHub Release is created with all binaries
5. Publish to crates.io: `cargo publish --locked`
6. Tag and push for PyPI: `git tag -a "py<version>" -m "chore: push py<version>" && git push origin py<version>`
7. Verify the package is available on [crates.io](https://crates.io/crates/marmite) and [PyPI](https://pypi.org/project/marmite/)
