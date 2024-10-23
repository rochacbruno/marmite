# Contributing to Marmite Static Site Generator

Thank you for considering contributing to the Marmite Site Generator project! Contributions are what make this project strong, and any help you can offer is highly appreciated. Below are the guidelines for contributing to the project.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Prerequisites](#prerequisites)
3. [How to Contribute](#how-to-contribute)
4. [Pull Requests](#pull-requests)
5. [Commit Messages](#commit-messages)
6. [Code Quality](#code-quality)

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

- All code submissions are done through pull requests against the `develop` branch.
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

2. Apply clippy fixes **optional**

```
cargo clippy
```
or
```
cargo clippy -- -W clippy:pedantic
```

> **hint**: you can add `--fix` for clippy to try to apply fixes.

#### Just

There is a `justfile` in the root of repo, you can use it for checkings.

```bash
cargo install just
just check

# Ensure your changes are committed before running.
just fix
```
