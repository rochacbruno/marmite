---
title: Marmite 0.3.2 Release Notes
slug: marmite-0-3-2-release-notes
description: "Marmite 0.3.2 adds embedded agent skills, making marmite projects fully AI-agent-ready with a single command."
tags: [release-notes, marmite, features, announcement]
author: rochacbruno
pinned: true
stream: draft
---

We're excited to announce Marmite 0.3.2! This release introduces embedded agent skills - a self-documenting system that lets AI coding agents build, configure, and manage marmite sites with full context.

## New Feature: Embedded Agent Skills

Marmite now ships with a comprehensive [Agent Skill](https://agentskills.io) embedded directly in the binary. The skill follows the open agent-skills format, making marmite projects instantly usable by AI coding agents like Claude Code, Codex, Gemini CLI, Cursor, and others.

### What's Included

The embedded skill contains a main `SKILL.md` with workflow-based documentation and 10 reference files covering every aspect of marmite:

- **SKILL.md** - Workflows for project setup, content authoring, configuration, templates, themes, shortcodes, image optimization, comments, and deployment
- **cli-reference.md** - Every CLI flag and option
- **installation.md** - All installation methods
- **config-reference.md** - Complete `marmite.yaml` field reference
- **frontmatter.md** - Content frontmatter fields and formats
- **content-organization.md** - Directory structure, taxonomy, fragment files, and site organization strategies
- **markdown-format.md** - Markdown syntax, extensions, wikilinks, math, and diagrams
- **tera-templates.md** - Template system, variables, functions, and filters
- **shortcodes.md** - Built-in and custom shortcode reference
- **deployment-guide.md** - Platform-specific deployment guides (GitHub Pages, Netlify, Vercel, Cloudflare, Docker, Nginx, Apache)
- **comment-system.md** - Giscus, Utterances, Hatsu, and other comment system integration

### New CLI Commands

Three new flags, none of which require an input folder argument:

**`marmite --skill`** - Print the embedded SKILL.md to stdout.

```console
$ marmite --skill
---
name: marmite
description: Build and manage static sites with marmite...
---
# Marmite Static Site Generator
...
```

**`marmite --skill-install`** - Install the skill into `.agents/skills/marmite/` in the current directory. This is the standard [agent-skills.io](https://agentskills.io) location used by Codex, Gemini CLI, Cursor, and other agents.

```console
$ cd myproject
$ marmite --skill-install
[INFO] Generated .agents/skills/marmite/SKILL.md
[INFO] Generated .agents/skills/marmite/references/config-reference.md
...
```

**`marmite --skill-install-claude`** - Install the skill into `.claude/skills/marmite/` for Claude Code, which uses a different directory convention.

```console
$ marmite --skill-install-claude
[INFO] Generated .claude/skills/marmite/SKILL.md
...
```

Both install commands can be combined to set up all agents at once:

```console
$ marmite --skill-install --skill-install-claude
```

### How It Works

The skill files are compiled into the marmite binary at build time using `rust_embed`, the same approach used for the default templates and static assets. This means:

- No network requests needed - everything is available offline
- Skills always match the installed marmite version
- Zero configuration - just run the install command

### What Agents Can Do With It

With the skill installed, an AI coding agent has full context to handle requests like:

- "Create a new marmite blog about cooking with search and the dracula colorscheme"
- "Add a Python tutorial series with three parts"
- "Set up GitHub Pages deployment for this site"
- "Create a custom shortcode for embedding recipe cards"
- "Add Giscus comments to the blog"
- "Customize the homepage with a hero section and sidebar"
- "Create a new theme based on the default one"

The agent reads the SKILL.md for the workflow, then loads the relevant reference files for exact field names, default values, and code snippets.

## CLI Change: Optional Input Folder

The `--skill`, `--skill-install`, and `--skill-install-claude` commands do not require an input folder argument. They work like `--version` and `--help` - just run them from anywhere.

For all other commands, the input folder remains required and behaves exactly as before.

## New Feature: AT Protocol & standard.site Integration

Marmite now supports native integration with the **AT Protocol** and the **standard.site** specification. This allows you to publish your blog posts to the decentralized social web while keeping your Marmite site as the canonical source.

Key capabilities:
- **Fully optional** and opt-in.
- **CLI Subcommand**: All functionality is cleanly organized under the `marmite atproto` command (`marmite atproto auth` to authenticate and `marmite atproto publish` to push updates).
- **Decentralized Endpoint Resolution**: Respects the protocol's decentralized nature: resolves handles via DNS or `.well-known` endpoints directly, finds your PDS using the PLC directory (`plc.directory`), and interacts directly with your PDS.
- **Verification and Header Injection**: Automatically generates verification files under `/.well-known/site.standard.publication` and injects publication and document headers (`link` tags) inside HTML templates.
- **Change Detection**: Tracks content hashes and last-published state dynamically to only publish new or modified posts.

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

### Getting Started with Agent Skills

After upgrading, install the skill in your project:

```bash
cd myproject
marmite --skill-install --skill-install-claude
```

That's it. Your project is now AI-agent-ready.

---

For the complete changelog, see the [GitHub releases page](https://github.com/rochacbruno/marmite/releases/tag/0.3.2).
