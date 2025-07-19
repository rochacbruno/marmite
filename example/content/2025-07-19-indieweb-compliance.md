---
tags: indieweb, microformats, standards, web-standards
description: Marmite now fully supports IndieWeb microformats, making your blog more discoverable and interoperable with the decentralized web ecosystem.
---

# IndieWeb Compliance: Marmite Joins the Decentralized Web

We're excited to announce that Marmite now fully supports **IndieWeb microformats**, making your static blog a first-class citizen of the decentralized web. This means your content is now more discoverable, machine-readable, and interoperable with a growing ecosystem of IndieWeb tools and services.

## What is the IndieWeb?

The [IndieWeb](https://indieweb.org) is a movement towards creating a more decentralized and user-controlled web. Instead of relying solely on social media platforms, IndieWeb principles encourage owning your content on your own domain while still enabling social interactions across the web.

Key IndieWeb principles include:

- **Own your data**: Publish on your own domain
- **Use visible data**: Make your content machine-readable
- **Make tools for yourself**: Build what you need
- **Document your stuff**: Share your knowledge
- **Open source your stuff**: Enable others to build upon your work

## Microformats: The Foundation

At the heart of IndieWeb compatibility are **microformats** - a simple way to add semantic markup to HTML that makes your content machine-readable without requiring separate APIs or databases.

Marmite now implements the core microformats:

### h-card (Personal Identity)
```html
<div class="h-card">
  <span class="p-name">John Doe</span>
  <img class="u-photo" src="avatar.jpg" alt="John">
  <a class="u-url" href="https://johndoe.com">Website</a>
  <p class="p-note">Software developer and blogger</p>
</div>
```

### h-entry (Blog Posts)
```html
<article class="h-entry">
  <h1 class="p-name">Post Title</h1>
  <time class="dt-published" datetime="2025-07-19">July 19, 2025</time>
  <div class="e-content">Post content here...</div>
  <a class="p-category" href="/tag/example">example</a>
</article>
```

### h-feed (Content Collections)
```html
<div class="h-feed">
  <h1 class="p-name">My Blog</h1>
  <article class="h-entry">...</article>
  <article class="h-entry">...</article>
</div>
```

## What This Means for Your Blog

With IndieWeb microformats, your Marmite-powered blog now supports:

### Enhanced Discoverability
- **Feed readers** can automatically discover and parse your content
- **Search engines** better understand your content structure
- **Social media platforms** can extract rich preview information
- **IndieWeb tools** can interact with your content programmatically

### Improved SEO and Metadata
- Search engines can better categorize your posts and author information
- Social sharing creates richer preview cards
- Author attribution is more clearly defined
- Content relationships (tags, dates, categories) are machine-readable

### Future-Proof Architecture
- Your content works with current and future IndieWeb tools
- No vendor lock-in - the markup is standards-based HTML
- Backward compatible with existing browsers and tools
- Forward compatible with emerging web technologies

## Features Enabled by Microformats

### Author Recognition
Every blog post now includes proper author attribution with h-card microformats, linking to author pages with full profile information including avatars, bios, and social links.

### Content Syndication
Your RSS and JSON feeds are enhanced with microformat data, making them more useful for feed readers and aggregation services.

### Tag and Category Intelligence
Tags are now marked up as `p-category` properties, making it easier for tools to understand your content taxonomy and suggest related content.

### Temporal Context
All dates use proper `dt-published` microformat properties with ISO 8601 datetime stamps, enabling precise chronological ordering and filtering.

### Social Interactions
Your content is now ready for IndieWeb social features like:
- **Webmentions**: Decentralized comments and reactions
- **Backlinks**: Automatic cross-site link discovery
- **POSSE**: Publish (on your) Own Site, Syndicate Elsewhere

## Implementation Details

Marmite's IndieWeb compliance includes:

### Automatic Microformat Injection
All templates now include appropriate microformat classes without requiring any configuration. The system intelligently applies:

- `h-card` for author profiles and site identity
- `h-entry` for individual blog posts and list items
- `h-feed` for content collections and archives
- `p-category` for tags and categories
- `dt-published` for publication dates
- `u-url` for canonical URLs

### Invisible Data Elements
Some microformat data is included invisibly to avoid visual clutter while providing complete machine-readable information:

```html
<data class="p-name" value="Post Title"></data>
<time class="dt-published" datetime="2025-07-19T10:00:00Z" style="display: none;">
```

### Nested Microformats
Author information within posts uses nested h-card microformats:

```html
<article class="h-entry">
  <div class="h-card p-author">
    <span class="p-name">Author Name</span>
    <img class="u-photo" src="avatar.jpg">
  </div>
</article>
```

## Testing Your IndieWeb Markup

You can validate your site's microformat implementation using these tools:

1. **[Microformats Parser](https://microformats.io)** - Parse any URL and see the extracted microformat data
2. **[IndieWebify.Me](https://indiewebify.me)** - Step-by-step IndieWeb validation
3. **[Microformats Wiki](https://microformats.org/wiki)** - Complete specification and examples

Example validation for a typical Marmite blog post:
```json
{
  "type": ["h-entry"],
  "properties": {
    "name": ["Post Title"],
    "published": ["2025-07-19T10:00:00Z"],
    "content": [{"html": "Post content...", "value": "Post content..."}],
    "category": ["tag1", "tag2"],
    "author": [{
      "type": ["h-card"],
      "properties": {
        "name": ["Author Name"],
        "photo": ["avatar.jpg"],
        "url": ["author-page.html"]
      }
    }]
  }
}
```

## Getting Started with IndieWeb

Your Marmite blog is now IndieWeb-ready out of the box! To fully embrace the IndieWeb ecosystem:

### 1. Verify Your Domain
Add `rel="me"` links to your social profiles and verify them back to your domain:

```yaml
# In marmite.yaml
extra:
  fediverse_verification: "https://mastodon.social/@username"
```

### 2. Enable Webmentions
Consider adding [Webmention](https://webmention.net) support to enable decentralized comments and interactions.

### 3. Join the Community
Connect with the IndieWeb community:
- [IndieWeb Chat](https://chat.indieweb.org)
- [IndieWeb Events](https://indieweb.org/events)
- [Getting Started Guide](https://indieweb.org/Getting_Started)

## Why This Matters

The web is increasingly centralized around a few major platforms. IndieWeb represents a movement back to the original vision of the web: a decentralized network of interconnected sites owned and controlled by their creators.

By supporting IndieWeb standards, Marmite helps you:

- **Own your content** on your own domain
- **Control your data** without platform dependencies  
- **Participate in the decentralized web** while maintaining full ownership
- **Future-proof your content** with open standards
- **Enable richer interactions** without sacrificing independence

## Technical Benefits

Beyond philosophical alignment, IndieWeb compliance provides concrete technical benefits:

### Better SEO
Search engines better understand your content structure, author information, and content relationships, potentially improving search rankings and rich snippets.

### Enhanced Social Sharing
When your content is shared on social platforms, the microformat data enables richer preview cards with proper titles, descriptions, and author attribution.

### Improved Accessibility
Semantic markup improves accessibility for screen readers and other assistive technologies.

### Developer-Friendly
Other developers can more easily build tools that work with your content, from feed readers to analytics to content management systems.

## Backward Compatibility

All IndieWeb microformat additions are purely additive - they don't break existing functionality or change your site's appearance. Your existing:

- CSS styles continue to work unchanged
- RSS and JSON feeds remain valid
- SEO and social sharing continue to function
- Custom templates remain compatible

The microformat classes are simply added alongside your existing classes, making the transition seamless.

## What's Next

Marmite's IndieWeb compliance opens the door to future features:

- **Webmention support** for decentralized comments
- **POSSE integration** for cross-posting to social platforms
- **Microsub reader compatibility** for feed consumption
- **IndieAuth integration** for decentralized authentication
- **Enhanced content discovery** through IndieWeb directories

## Conclusion

The web is more than social media platforms. With IndieWeb microformats, your Marmite blog becomes part of a growing ecosystem of independent, interconnected websites that prioritize user control and open standards.

Your content is now ready for the decentralized web while remaining fully under your control. Welcome to the IndieWeb!

---

*Learn more about the IndieWeb at [indieweb.org](https://indieweb.org) and test your site's microformat compliance at [microformats.io](https://microformats.io).*