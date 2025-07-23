---
tags: docs, markdown, features
description: Learn how to publish markdown source files alongside your HTML content with Marmite's new source publishing feature.
---

# Markdown Source Publishing

Marmite now supports publishing the original markdown source files alongside your HTML content, making it easy for readers to access the raw markdown files directly. This feature is perfect for technical blogs, documentation sites, and any content where you want to provide transparency about your sources.

## Why Publish Markdown Sources?

Publishing markdown sources alongside your HTML content offers several benefits:

- **Transparency**: Readers can see exactly how you've structured your content
- **Collaboration**: Others can easily suggest improvements or corrections
- **Learning**: New writers can learn from your markdown formatting and structure
- **Backup**: Provides an additional way to access your content
- **Version Control**: When linked to repositories, readers can see the full history of changes

## How It Works

Marmite provides two ways to share your markdown sources:

### 1. Local File Publishing

When you enable `publish_md: true`, Marmite copies all your markdown files to the output directory and adds "📄 View source" links to each post that point to the local markdown files.

```yaml
# marmite.yaml
publish_md: true
```

Or via command line:

```console
$ marmite myblog output/ --publish-md true
```

### 2. Repository Linking

If you store your content in a Git repository (like GitHub, GitLab, or Codeberg), you can link directly to the repository instead of copying local files:

```yaml
# marmite.yaml
publish_md: true
source_repository: https://github.com/user/repo/tree/main/content
```

Or via command line:

```console
$ marmite myblog output/ --publish-md true --source-repository "https://github.com/user/repo/tree/main/content"
```

## Configuration Options

### In Configuration File

Add these options to your `marmite.yaml`:

```yaml
# Enable markdown source publishing
publish_md: true

# Optional: Link to external repository
source_repository: https://github.com/user/repo/tree/main/content
```

### Via Command Line

Both options can be overridden from the command line:

```console
# Enable local publishing only
$ marmite myblog output/ --publish-md true

# Enable repository linking
$ marmite myblog output/ --publish-md true --source-repository "https://github.com/user/repo/tree/main/content"
```

## Behavior and Precedence

The feature follows these rules:

1. **Repository links take precedence**: If both `publish_md` and `source_repository` are configured, links will point to the repository
3. **Automatic detection**: Links are automatically added to the footer of each post
4. **Visual indicator**: Links include a 📄 emoji and "View source" text with `rel="nofollow"` attribute

## Visual Examples

When enabled, you'll see source links at the bottom of each post:

```html
<div class="content-source">
  <a href="./2025-07-18-markdown-source-publishing.md" rel="nofollow">📄 View source</a>
</div>
```

Or for repository links:

```html
<div class="content-source">
  <a href="https://github.com/user/repo/tree/main/content/2025-07-18-markdown-source-publishing.md" rel="nofollow">📄 View source</a>
</div>
```

## Use Cases

This feature is particularly useful for:

- **Technical documentation**: Allow readers to see the exact markdown formatting
- **Tutorial blogs**: Let learners study your content structure
- **Open source projects**: Provide easy access to source files for contributions
- **Academic writing**: Offer transparency in your writing process
- **Personal blogs**: Share your writing workflow with interested readers

## Template Customization

If you want to customize how source links appear, you can modify the `content.html` template:

```html
{% if content.source_path and site.publish_md or site.source_repository %}
<div class="content-source">
  {% set source_url = source_link(content=content) %}
  {% if source_url %}
    <a href="{{ source_url }}" rel="nofollow">📄 View source</a>
  {% endif %}
</div>
{% endif %}
```

## Getting Started

To start using markdown source publishing:

1. **Enable the feature** in your `marmite.yaml` or via CLI
2. **Choose your approach**: Local files or repository linking
3. **Rebuild your site** to see the source links appear
4. **Customize styling** if needed using CSS

The feature is completely optional and doesn't affect your existing content if not enabled.

---

*This feature was added in response to user feedback requesting better transparency and collaboration features. We hope it helps you build more open and accessible content!*
