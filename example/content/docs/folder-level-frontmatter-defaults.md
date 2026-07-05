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

## Nested subfolders

Frontmatter defaults work at any nesting depth. Each level inherits from its parent and can add or override values:

```
content/
  frontmatter.yaml                  # authors: [admin]
  tutorials/
    frontmatter.yaml                # stream: tutorial (inherits authors from root)
    tutorials/python/
      frontmatter.yaml              # tags: [python] (inherits authors + stream)
      basics.md                     # gets all three: authors, stream, tags
```

Files in nested subfolders without their own `frontmatter.yaml` inherit from the nearest ancestor that has one. A file at `content/tutorials/python/advanced/decorators.md` would inherit from `content/tutorials/python/frontmatter.yaml`.

## Merge priority

From lowest to highest priority:

1. Root `content/frontmatter.yaml`
2. Parent subfolder `frontmatter.yaml` files (layered from shallowest to deepest)
3. Filename conventions (date, stream, and language detection from the filename)
4. The markdown file's own frontmatter block

## Use with language streams

Folder-level defaults work alongside translation groups. A subfolder can be both a translation group and have shared defaults:

```
content/
  hello/
    frontmatter.yaml      # shared defaults
    hello.md               # default language version
    pt-ola.md              # Portuguese version
```

Both translations inherit the folder defaults while still being detected and linked as translations.

Translation groups work at any nesting depth. Each subfolder forms its own independent group:

```
content/
  poetry/
    love/
      love-poem.md         # default language
      pt-poema-amor.md     # Portuguese translation of love poem
    nature/
      nature-poem.md       # default language (separate group from love/)
      pt-poema-natureza.md # Portuguese translation of nature poem
```

The `love/` and `nature/` subfolders are treated as separate translation groups - translations in one subfolder are not mixed with translations in the other.
