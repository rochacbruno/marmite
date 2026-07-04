---
title: Folder-Level Frontmatter Defaults
slug: folder-level-frontmatter-defaults
description: "Organize content in subfolders with shared frontmatter defaults - reduce repetition by inheriting stream, tags, date, and other metadata from a folder-level configuration file."
tags: [content, organization, frontmatter, streams, features]
date: 2026-07-04 12:00:00
author: rochacbruno
---

When multiple posts share the same metadata - stream, tags, date, authors, or extra fields - you can avoid repeating it in every file by placing a `frontmatter.yaml` in the folder. All `.md` files in that folder inherit the values as defaults, while still being able to override any field individually.

## How it works

Place a `frontmatter.yaml` file in any content subfolder. Its key-value pairs become the default frontmatter for all markdown files in that folder.

```
content/
  python/
    frontmatter.yaml      # defaults for all files in this folder
    databases.md
    classes.md
    web-frameworks.md
```

The `frontmatter.yaml` is a plain YAML file (no `---` delimiters needed):

```yaml
date: 2026-01-01
stream: python
tags:
  - python
  - programming
extra:
  mermaid: true
```

With this setup, `databases.md` only needs its own title:

```markdown
---
title: Python Databases
---

# Working with Databases in Python

Content here...
```

It automatically gets `date: 2026-01-01`, `stream: python`, the python/programming tags, and `extra.mermaid: true` from the folder defaults.

## Override any field

Per-file frontmatter always takes precedence over folder defaults. If `classes.md` needs different tags:

```markdown
---
title: Python Classes
tags:
  - oop
  - beginner
---
```

This file gets `tags: [oop, beginner]` instead of the folder defaults, while still inheriting `date`, `stream`, and `extra` from `frontmatter.yaml`.

The `title` and `slug` fields are never inherited from folder defaults - they are always per-file.

## Root-level defaults

The `content/` directory itself can also have a `frontmatter.yaml` to set defaults for all content across the entire site:

```
content/
  frontmatter.yaml        # applies to ALL content
  python/
    frontmatter.yaml      # applies to python/ files
    intro.md
  2026-01-15-standalone.md # inherits from root only
```

When both root and subfolder defaults exist, they layer: root defaults apply first, then subfolder defaults override, then per-file frontmatter overrides last.

## Merge priority

From lowest to highest priority:

1. Root `content/frontmatter.yaml`
2. Subfolder `content/{folder}/frontmatter.yaml`
3. Filename conventions (date, stream, and language detection from the filename)
4. The markdown file's own frontmatter block

## Subfolder rendering rules

Not all subfolders render their content. A subfolder's `.md` files are only processed when at least one of these is true:

- The subfolder contains a `frontmatter.yaml` (even an empty one)
- The subfolder is named `pages` (always rendered for undated page content)
- The subfolder is a translation group (contains files with language-prefixed names like `pt-ola.md`)

Subfolders that don't match any of these conditions are ignored. This prevents accidental rendering of files in arbitrary subdirectories.

To enable a subfolder for rendering without setting any defaults, create an empty `frontmatter.yaml`:

```bash
touch content/notes/frontmatter.yaml
```

## Use with language streams

Folder-level defaults work alongside translation groups. A subfolder can be both a translation group and have shared defaults:

```
content/
  hello/
    frontmatter.yaml      # shared defaults
    en-hello.md            # English version
    pt-ola.md              # Portuguese version
```

Both translations inherit the folder defaults while still being detected and linked as translations.
