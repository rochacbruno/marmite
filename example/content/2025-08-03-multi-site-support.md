---
title: Multi-Site Support in Marmite
slug: multi-site-support
tags: features, docs, content-organization
author: rochacbruno
---

# Multi-Site Support in Marmite

Marmite now supports generating multiple independent sites from a single project! This feature allows you to create subsites with their own themes, configurations, and search indexes.

## How it works

Any directory inside your `content` folder that contains a `site.yaml` file will be processed as a subsite. Each subsite:

- Has its own independent configuration
- Can use a different theme
- Maintains a separate search index  
- Is served at its own URL path (e.g., `/site1/`)

## Creating a subsite

1. Create a directory in your content folder:
   ```bash
   mkdir content/site1
   ```

2. Add a `site.yaml` configuration file:
   ```yaml
   name: My Subsite
   theme: theme_template
   enable_search: true
   ```

3. Add content files (markdown) to the subsite directory

4. Build your site as usual - Marmite will automatically detect and process the subsite

## Example structure

```
example/
├── marmite.yaml          # Main site config
├── content/
│   ├── posts.md         # Main site content
│   └── site1/
│       ├── site.yaml    # Subsite config
│       ├── about.md     # Subsite content
│       └── posts.md     # More subsite content
└── site/                # Generated output
    ├── index.html       # Main site
    └── site1/
        └── index.html   # Subsite at /site1/
```

## Configuration inheritance

Subsites inherit certain configuration values from the parent site if not specified:

- Theme selection
- Date format settings
- Other default values

## Use cases

- Documentation sites with versioned content
- Multi-language sites
- Separate blogs or sections with different themes
- Project portfolios with independent sections

This feature enables complex site architectures while maintaining Marmite's simplicity and performance.