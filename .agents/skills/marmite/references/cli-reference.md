# Marmite CLI Reference

```
marmite [OPTIONS] [INPUT_FOLDER] [OUTPUT_FOLDER]
```

`INPUT_FOLDER` is the directory containing markdown files. Required for most commands, but optional for `--skill`, `--skill-install`, and `--skill-install-claude`. `OUTPUT_FOLDER` defaults to `INPUT_FOLDER/site`.

## Commands

### Site Generation

```bash
# Build the site
marmite <input> [output]

# Build with verbose output
marmite <input> -v

# Force full rebuild (ignore cache)
marmite <input> --force

# Build and serve locally
marmite <input> --serve

# Build, serve, and watch for changes
marmite <input> --serve --watch

# Custom server address
marmite <input> --serve --bind 127.0.0.1:3000
```

### Project Initialization

```bash
# Scaffold a new project with sample content
marmite <folder> --init-site

# Scaffold with config overrides
marmite <folder> --init-site --name "My Blog" --tagline "Articles" --colorscheme nord

# Generate default marmite.yaml only
marmite <folder> --generate-config

# Generate config with overrides
marmite <folder> --generate-config --name "My Blog" --pagination 20 --url "https://myblog.com"
```

### Content Creation

```bash
# Create a new post (date auto-set)
marmite <folder> --new "Post Title"

# Create a new page (no date)
marmite <folder> --new "Page Title" -p

# Create with tags
marmite <folder> --new "Post Title" -t "tag1,tag2"

# Create and open in $EDITOR
marmite <folder> --new "Post Title" -e

# Combine all options
marmite <folder> --new "Tutorial: Getting Started" -t "tutorial,beginner" -e

# Create a post in a specific subfolder
marmite <folder> --new "Post Title" -d posts

# Create a page in the pages subfolder
marmite <folder> --new "About" -p -d pages

# Create in a topic subfolder (creates it if needed)
marmite <folder> --new "Python Basics" -d tutorials

# Create with a language code
marmite <folder> --new "Articulo" --lang es

# Create a translation of an existing post
marmite <folder> --new "Ola Mundo" --lang pt --translates hello-world

# In workspace mode, specify the target site
marmite <workspace> --new "Post Title" --site blog
```

The `--new` command outputs JSON with fields: `file`, `title`, `slug`, `date` (posts only), `tags`, `language`, `translates` (when applicable). Use `jq` to extract specific fields:

```bash
marmite <folder> --new "Title" | jq -r .file
```

When `--translates <slug>` is used:
- If the original content is in a subfolder, the translation is placed in the same folder with a language-code prefix (e.g. `pt-slug.md`)
- If the original is at root level, the translation goes to root with `translates` frontmatter

### Workspace (Multi-Site)

```bash
# Build all sites in a workspace (detected by marmite-workspace.yaml)
marmite <workspace> <output>

# Show URLs from all sites
marmite <workspace> --show-urls

# List shortcodes from all sites
marmite <workspace> --shortcodes

# Build and serve with live reload
marmite <workspace> <output> --serve --watch

# Create content in a specific site
marmite <workspace> --new "Post Title" --site blog
```

Workspace mode is activated when `marmite-workspace.yaml` is found in the input folder. Theme commands and `--init-site` are not supported in workspace mode - run them on individual sites.

### Templates and Themes

```bash
# Export default templates for customization
marmite <folder> --init-templates

# Create a new theme from built-in template
marmite <folder> --start-theme mytheme

# Install theme from GitHub
marmite <folder> --set-theme https://github.com/user/marmite-theme

# Install theme from GitLab
marmite <folder> --set-theme https://gitlab.com/user/theme

# Install theme from Codeberg
marmite <folder> --set-theme https://codeberg.org/user/theme

# Install theme from ZIP URL
marmite <folder> --set-theme https://example.com/theme.zip

# Set a local theme folder
marmite <folder> --set-theme mytheme

# Override theme for a single build
marmite <folder> --theme mytheme
```

### Information

```bash
# List available shortcodes
marmite <folder> --shortcodes

# Preview all site URLs (dry run, JSON output)
marmite <folder> --show-urls

# Preview with absolute URLs
marmite <folder> --show-urls --url "https://myblog.com"

# Print embedded agent skill document (no input folder needed)
marmite --skill

# Install skill to .agents/skills/ (standard agents, defaults to CWD)
marmite --skill-install

# Install skill to .claude/skills/ (Claude Code)
marmite --skill-install-claude

# Install for all agents at once
marmite --skill-install --skill-install-claude

# Show version
marmite --version

# Show help
marmite --help
```

### AT Protocol / standard.site

```bash
# Authenticate with your PDS (Personal Data Server)
# Requires ATPROTO_APP_PASSWORD env var and atproto.handle configured in marmite.yaml
marmite [site_folder] atproto auth

# Publish your blog posts to the PDS as site.standard.document records
marmite [site_folder] atproto publish

# Publish posts with options
marmite [site_folder] atproto publish --force
marmite [site_folder] atproto publish --dry-run
```

## All Flags and Options

### Build Control

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--verbose` | `-v` | 0 (warn) | Verbosity: `-v` info, `-vv` debug, `-vvv` trace, `-vvvv` trace all |
| `--watch` | `-w` | false | Auto-rebuild on file changes |
| `--serve` | | false | Start built-in HTTP server |
| `--bind <ADDR>` | | `0.0.0.0:8000` | Server bind address (requires `--serve`) |
| `--config <FILE>` | `-c` | `marmite.yaml` | Path to config file |
| `--force` | | false | Force full rebuild |

### Initialization

| Flag | Description |
|------|-------------|
| `--init-site` | Create new project with sample content and config |
| `--init-templates` | Export default templates to `templates/` |
| `--start-theme <NAME>` | Create new theme directory from built-in template |
| `--set-theme <SOURCE>` | Download and install theme from URL or local path |
| `--generate-config` | Generate default `marmite.yaml` |

### Content Creation

| Flag | Short | Description |
|------|-------|-------------|
| `--new <TITLE>` | | Create new markdown file with given title (outputs JSON) |
| `-e` | | Open in `$EDITOR` (requires `--new`) |
| `-p` | | Create as page instead of post (requires `--new`) |
| `-t <TAGS>` | | Set comma-separated tags (requires `--new`) |
| `-d` | | Directory within content folder to create the file |
| `--lang <CODE>` | | ISO 639-1 language code for the new content (requires `--new`) |
| `--translates <SLUG>` | | Slug of original content this translates (requires `--new` and `--lang`) |

### Information

| Flag | Description |
|------|-------------|
| `--shortcodes` | List available shortcodes with examples |
| `--show-urls` | Preview all site URLs as JSON (dry run) |
| `--skill` | Print embedded SKILL.md to stdout (no input folder needed) |
| `--skill-install` | Install skill to `.agents/skills/` (defaults to CWD) |
| `--skill-install-claude` | Install skill to `.claude/skills/` for Claude Code |
| `--version` | Print version |
| `--help` | Print help |

### Configuration Overrides

These flags override values from `marmite.yaml` for a single build:

| Flag | Type | Default |
|------|------|---------|
| `--name <NAME>` | String | `"Home"` |
| `--tagline <TAGLINE>` | String | `""` |
| `--url <URL>` | String | `""` |
| `--https <BOOL>` | bool | false |
| `--footer <FOOTER>` | String | Marmite credit |
| `--language <CODE>` | String | `"en"` |
| `--pagination <N>` | int | 10 |
| `--enable-search <BOOL>` | bool | false |
| `--search-show-matches <BOOL>` | bool | false |
| `--search-match-count <N>` | int | 3 |
| `--enable-related-content <BOOL>` | bool | true |
| `--show-next-prev-links <BOOL>` | bool | true |
| `--content-path <PATH>` | String | `"content"` |
| `--templates-path <PATH>` | String | `"templates"` |
| `--static-path <PATH>` | String | `"static"` |
| `--media-path <PATH>` | String | `"media"` |
| `--default-date-format <FMT>` | String | `"%b %e, %Y"` |
| `--colorscheme <NAME>` | String | `"default"` |
| `--toc <BOOL>` | bool | false |
| `--json-feed <BOOL>` | bool | false |
| `--publish-md <BOOL>` | bool | false |
| `--source-repository <URL>` | String | none |
| `--image-provider <NAME>` | String | none (`picsum`) |
| `--theme <NAME>` | String | none |
| `--build-sitemap <BOOL>` | bool | true |
| `--publish-urls-json <BOOL>` | bool | true |
| `--enable-shortcodes <BOOL>` | bool | true |
| `--shortcode-pattern <REGEX>` | String | HTML comment pattern |
| `--skip-image-resize <BOOL>` | bool | false |

## Show URLs Output

`--show-urls` outputs JSON with these sections:

```bash
marmite <folder> --show-urls | jq '.summary'
```

Categories: `posts`, `pages`, `tags`, `authors`, `series`, `streams`, `archives`, `feeds`, `pagination`, `file_mappings`, `misc`.

Useful with `jq`:
```bash
# Count total URLs
marmite <folder> --show-urls | jq '.summary.total'

# List post URLs
marmite <folder> --show-urls | jq -r '.posts[]'

# List feed URLs
marmite <folder> --show-urls | jq -r '.feeds[]'
```

## Automatic Image Download

When `--image-provider picsum` is set, marmite downloads banner images for posts that don't have a `banner_image` in frontmatter. Images are saved as `{slug}.banner.jpg` in the media folder. The image is deterministic based on site name, slug, and tags.

```bash
marmite <folder> --image-provider picsum
```

Only applies to posts (dated content), not pages.
