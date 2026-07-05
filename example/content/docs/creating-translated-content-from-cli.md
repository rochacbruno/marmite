---
title: Creating Translated Content from the CLI
slug: creating-translated-content-from-cli
tags: docs,i18n
toc: true
---

Marmite's `--new` command now supports creating translations of existing content directly from the command line, and outputs structured JSON for easy scripting.

## JSON Output

The `--new` command outputs JSON instead of a plain file path. This makes it easy to pipe results into other tools:

```console
$ marmite myblog --new "Hello World"
{"file":"myblog/content/2024-11-26-12-34-27-hello-world.md","title":"Hello World","slug":"hello-world","date":"2024-11-26"}
```

Use `jq` to extract specific fields:

```console
$ marmite myblog --new "Hello World" | jq -r .file
myblog/content/2024-11-26-12-34-27-hello-world.md
```

The JSON output includes all applicable fields: `file`, `title`, `slug`, `date` (posts only), `tags`, `language`, and `translates`.

## Setting a Language

Use `--lang` with a two-letter ISO 639-1 code to set the `language` frontmatter field:

```console
$ marmite myblog --new "Articulo en Espanol" --lang es
```

This creates a file with `language: es` in its frontmatter, which tells marmite to assign the content to the corresponding language stream.

## Creating Translations

Use `--translates` along with `--lang` to create a translation of an existing post. Pass the slug of the original content:

```console
$ marmite myblog --new "Ola Mundo" --lang pt --translates hello-world
```

### Subfolder-based translations

When the original content is inside a subfolder, the translation file is automatically placed in the same folder with a language-code prefix:

```
content/
  hello-world/
    hello-world.md           # original (created with -d hello-world)
    pt-ola-mundo.md          # translation (auto-placed here)
```

This follows the subfolder translation convention. Marmite detects the language from the `pt-` filename prefix and automatically links the translations during site generation.

### Root-level translations

When the original content is at the root of the content directory, the translation is also created at root level. The file includes `language` and `translates` fields in its frontmatter so marmite can link them:

```yaml
---
language: pt
translates: hello-world
---
# Ola Mundo
```

## Workflow Example

A typical workflow for creating a post and its translations:

```bash
# Create the original post in a subfolder
marmite myblog --new "Getting Started Guide" -p -d getting-started

# Add a Portuguese translation
marmite myblog --new "Guia de Introducao" --lang pt --translates getting-started-guide

# Add a Spanish translation
marmite myblog --new "Guia de Inicio" --lang es --translates getting-started-guide
```

The resulting structure:

```
content/
  getting-started/
    getting-started-guide.md
    pt-guia-de-introducao.md
    es-guia-de-inicio.md
```

All three files are automatically grouped as translations during site generation, with bidirectional links rendered in templates.

## Flag Reference

| Flag | Requires | Description |
|------|----------|-------------|
| `--lang <CODE>` | `--new` | ISO 639-1 language code for the new content |
| `--translates <SLUG>` | `--new`, `--lang` | Slug of the original content being translated |

Note: `--translates` conflicts with `-d` (directory), since the translation's directory is determined automatically from the original content's location.
