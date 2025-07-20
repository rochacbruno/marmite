---
title: "How to Use Draft Posts in Marmite"
description: "Learn how to create draft posts that won't appear in feeds or search results"
tags: ["documentation", "draft", "guide"]
authors: ["marmite"]
---

# How to Use Draft Posts in Marmite

Marmite supports draft posts that allow you to work on content without making it publicly discoverable through feeds or search. This guide explains how to create and manage draft posts.

## Creating a Draft Post

To mark a post as a draft, simply add `stream: draft` to your post's frontmatter:

```markdown
---
title: "My Work in Progress Post"
description: "This post is still being written"
tags: ["example"]
stream: draft
authors: ["yourname"]
---

# My Draft Post

This content is still being worked on and won't appear in feeds.
```

## What Happens to Draft Posts

When you set a post's stream to "draft", the following behavior occurs:

### ✅ What Still Works
- **Individual HTML pages are generated** - You can still view the post directly by URL
- **Draft stream page is created** - A `draft.html` page lists all draft posts
- **All normal post processing** - Markdown rendering, template processing, etc.

### ❌ What Gets Excluded
- **RSS feeds** - Draft posts won't appear in `index.rss` or any other RSS feeds
- **JSON feeds** - Draft posts won't appear in `index.json` or any other JSON feeds  
- **Search index** - Draft posts won't be included in `search_index.json`
- **Feed files for draft stream** - No `draft.rss` or `draft.json` files are generated

## Use Cases for Draft Posts

Draft posts are useful for:

- **Work in progress content** - Write and preview posts before publishing
- **Internal documentation** - Create content for your team that shouldn't be public
- **Scheduled content** - Prepare posts in advance (remember to change the stream when ready to publish)
- **Experimental content** - Test ideas without affecting your public feeds

## Publishing a Draft Post

To publish a draft post, simply:

1. Remove the `stream: draft` line from the frontmatter, OR
2. Change it to a different stream like `stream: blog` or `stream: news`

The post will then appear in all feeds and search results on the next build.

## Viewing Draft Posts

Draft posts are still accessible if you know the URL:
- Individual post: `yoursite.com/draft-post-title.html` 
- Draft stream page: `yoursite.com/draft.html` (lists all draft posts)

The draft stream page can be useful for reviewing all your unpublished content.

## Security Note

Remember that draft posts still generate HTML files that are publicly accessible if someone knows the URL. If you need truly private content, consider using a separate private repository or local development environment.