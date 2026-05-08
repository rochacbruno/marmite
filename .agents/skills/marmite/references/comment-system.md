# Marmite Comment System Reference

Marmite is a static site generator and does not include a built-in commenting system. Comments are added by integrating external services that inject JavaScript-based comment widgets.

## Supported Comment Systems

| System | Backend | Auth Required | Free | Self-hosted |
|--------|---------|---------------|------|-------------|
| **Giscus** | GitHub Discussions | GitHub login | Yes | No |
| **Utterances** | GitHub Issues | GitHub login | Yes | No |
| **Hatsu** | ActivityPub/Fediverse | Fediverse account | Yes | Yes |
| **Disqus** | Disqus platform | Email/social | Freemium | No |
| **Commento** | Commento platform | Email | Paid | Optional |

## Setup Methods

There are three ways to add comments to a marmite site, in order of simplicity:

### Method 1: Fragment File (Recommended)

Create `content/_comments.md` with the comment script:

```markdown
##### Comments

<script src="https://giscus.app/client.js"
    data-repo="youruser/yourrepo"
    data-repo-id="YOUR_REPO_ID"
    data-category="Comments"
    data-category-id="YOUR_CATEGORY_ID"
    data-mapping="pathname"
    data-strict="0"
    data-reactions-enabled="1"
    data-emit-metadata="0"
    data-input-position="bottom"
    data-theme="preferred_color_scheme"
    data-lang="en"
    data-loading="lazy"
    crossorigin="anonymous"
    async>
</script>
```

The `_comments.md` fragment supports Tera templating with the global context, so you can use template expressions if needed.

### Method 2: Config File

Add to `marmite.yaml`:

```yaml
extra:
  comments:
    title: "Comments"
    source: |
      <script src="https://giscus.app/client.js"
        data-repo="youruser/yourrepo"
        data-repo-id="YOUR_REPO_ID"
        data-category="Comments"
        data-category-id="YOUR_CATEGORY_ID"
        data-mapping="pathname"
        data-strict="0"
        data-reactions-enabled="1"
        data-emit-metadata="0"
        data-input-position="bottom"
        data-theme="preferred_color_scheme"
        data-lang="en"
        data-loading="lazy"
        crossorigin="anonymous"
        async>
      </script>
```

### Method 3: Custom Template

Create `templates/comments.html`:

```html
<article>
  <header>{{ site.extra.comments.title | default(value="Comments") }}</header>
  {{ site.extra.comments.source }}
</article>
```

This gives full control over the comments section HTML structure.

## Giscus Setup

Giscus uses GitHub Discussions as the comment backend.

### Prerequisites

1. A **public** GitHub repository
2. The [Giscus app](https://github.com/apps/giscus) installed on the repository
3. [Discussions enabled](https://docs.github.com/en/github/administering-a-repository/managing-repository-settings/enabling-or-disabling-github-discussions-for-a-repository) on the repository

### Configuration

1. Go to https://giscus.app/
2. Enter your repository name
3. Choose the discussion category (create an "Announcements" or "Comments" category)
4. Select your preferred options (mapping, reactions, theme, etc.)
5. Copy the generated `<script>` tag
6. Add it to `content/_comments.md` or `marmite.yaml`

### Recommended Giscus Settings

- **Mapping**: `pathname` - maps comments to the page URL path
- **Theme**: `preferred_color_scheme` - respects the visitor's system theme
- **Reactions**: enabled
- **Input position**: `bottom` - comment box below existing comments
- **Loading**: `lazy` - load comments on scroll for better performance

## Utterances Setup

Utterances uses GitHub Issues as the comment backend.

### Prerequisites

1. A **public** GitHub repository
2. The [Utterances app](https://github.com/apps/utterances) installed on the repository

### Configuration

Add to `content/_comments.md`:

```markdown
##### Comments

<script src="https://utteranc.es/client.js"
    repo="youruser/yourrepo"
    issue-term="pathname"
    theme="preferred-color-scheme"
    crossorigin="anonymous"
    async>
</script>
```

## Hatsu Setup (ActivityPub/Fediverse)

Hatsu bridges your static site to the Fediverse via ActivityPub.

### Prerequisites

1. A running Hatsu instance
2. Visitors need a Fediverse account to comment

### Configuration

Follow the Hatsu documentation to set up the instance and generate the embed script, then add it to `content/_comments.md`.

## Other Comment Systems

Any JavaScript-based comment system can be integrated. The general pattern:

```markdown
##### Comments

<div id="comment-container"></div>
<script src="https://commentsystem.example.com/embed.js"
    data-site-id="YOUR_SITE_ID"
    async>
</script>
```

## Disabling Comments Per Post

Comments appear on all posts by default when configured. To disable comments on a specific post:

```yaml
---
title: "Post Without Comments"
comments: false
---
```

## Comment System Considerations

### Giscus vs Utterances

| Feature | Giscus | Utterances |
|---------|--------|------------|
| Backend | GitHub Discussions | GitHub Issues |
| Threading | Yes | No |
| Reactions | Emoji reactions | Issue reactions |
| Search | Discussions search | Issues search |
| Admin | Discussion moderation | Issue management |

Giscus is generally recommended as it uses Discussions (purpose-built for conversations) rather than Issues.

### Privacy and Self-hosting

- **Giscus/Utterances**: Data stored on GitHub. Visitors need GitHub accounts.
- **Hatsu**: Self-hosted, federated. Visitors need Fediverse accounts.
- **Disqus**: Third-party hosted, includes tracking/ads on free tier.
- **Commento**: Privacy-focused, paid or self-hosted.

### Performance

All comment systems use `async` or `lazy` loading, so they do not block page rendering. The `data-loading="lazy"` option (Giscus) defers loading until the comments section scrolls into view.
