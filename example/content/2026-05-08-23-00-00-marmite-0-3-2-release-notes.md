---
title: Marmite 0.3.2 Release Notes
slug: marmite-0-3-2-release-notes
description: "Marmite 0.3.2 adds embedded AI agent skills, AT Protocol / standard.site publishing, and the Marmite Playground."
tags: [release-notes, marmite, features, announcement]
author: rochacbruno
pinned: true
stream: draft
---

We're excited to announce Marmite 0.3.2! This release introduces embedded AI agent skills, native AT Protocol publishing via standard.site, and the Marmite Playground - an interactive web app for trying marmite in the browser.

## New Features

### Embedded AI Agent Skills

Marmite now ships with a comprehensive [Agent Skill](https://agentskills.io) embedded directly in the binary. The skill follows the open agent-skills format, making marmite projects instantly usable by AI coding agents like Claude Code, Codex, Gemini CLI, Cursor, and others. (#438)

The embedded skill contains a main `SKILL.md` with workflow-based documentation and 10 reference files covering every aspect of marmite - from CLI flags and configuration to templates, shortcodes, and deployment guides.

**New CLI flags:**

- **`marmite --skill`** - Print the embedded SKILL.md to stdout
- **`marmite --skill-install`** - Install the skill into `.agents/skills/marmite/` (standard agent-skills location)
- **`marmite --skill-install-claude`** - Install the skill into `.claude/skills/marmite/` (Claude Code convention)

```console
$ cd myproject
$ marmite --skill-install --skill-install-claude
```

The skill files are compiled into the binary at build time using `rust_embed`, so they are always available offline and always match the installed marmite version. With the skill installed, an AI coding agent has full context to create sites, add content, configure templates, set up deployment, and more.

**Getting started:** After installing or upgrading marmite, run the following in your project directory:

```bash
cd myproject
marmite --skill-install --skill-install-claude
```

That's it. Your project is now AI-agent-ready.

### AT Protocol & standard.site Integration

Marmite now supports native integration with the [AT Protocol](https://atproto.com) and the [standard.site](https://standard.site) specification. This allows you to publish your blog posts to the decentralized social web (e.g., discoverable inside [Bluesky](https://bsky.app)) while keeping your Marmite site as the canonical source. (#448)

This feature is **fully optional** and opt-in. Key capabilities:

- **Authentication** - Resolves your handle via DNS or `.well-known` endpoints, finds your PDS using the PLC directory, and authenticates with an app password
- **Publishing** - Publishes posts as `site.standard.document` records to your PDS, with change detection to only push new or modified content
- **Verification** - Automatically generates `/.well-known/site.standard.publication` and injects `<link>` tags for publication and document discovery
- **Dry-run support** - Preview what would be published without making changes (`--dry-run`)

Configuration in `marmite.yaml`:

```yaml
atproto:
  handle: "myhandle.bsky.social"
  publication_uri: "at://did:plc:.../site.standard.publication/..."
  publish_content: true
```

See the [[AT Protocol standard.site]] documentation for setup and usage details.

Thanks to **Eduardo Cuducos** for this contribution!

### Marmite Playground

A new interactive web application that lets you try Marmite directly in the browser without installing anything. The playground supports live editing of content and configuration, auto-complete, and auto-navigation between pages. (#359)

Try it at [play.marmite.blog](https://play.marmite.blog). The playground is a separate application in the `playground/` directory, deployable via Docker.

## CLI Changes

### Optional Input Folder for Skill Commands

The `--skill`, `--skill-install`, and `--skill-install-claude` flags do not require an input folder argument. They work like `--version` and `--help` - just run them from anywhere.

For all other commands, the input folder remains required and behaves exactly as before.

### AT Protocol Subcommands

Two new subcommands for AT Protocol publishing:

- **`marmite <folder> auth`** - Authenticate with your atproto PDS using an app password
- **`marmite <folder> publish`** - Publish posts to your PDS as `site.standard.document` records

The `publish` subcommand supports `--force` (re-publish all posts, ignoring change detection) and `--dry-run` / `-n` (preview what would be published without making changes).

## Breaking Changes

_None in this release._

## Fixes

_No bug fixes in this release (see 0.3.1 for recent fixes)._

## Dependency Updates

- `arborium` 2.16.0 -> 2.18.0
- `rss` 2.0.12 -> 2.0.13
- `log` 0.4.29 -> 0.4.30
- `serde_json` 1.0.149 -> 1.0.150

New dependencies for AT Protocol support: `ureq` (with JSON feature), `shrike`, `sha2`, `dirs`.

## Upgrading

To upgrade to Marmite 0.3.2:

```bash
# If installed via cargo
cargo install marmite --force

# If installed via pip
pip install --upgrade marmite

# Or use the install script
curl -sSL https://marmite.blog/install.sh | bash
```

---

For the complete changelog, see the [GitHub releases page](https://github.com/rochacbruno/marmite/releases/tag/0.3.2).
