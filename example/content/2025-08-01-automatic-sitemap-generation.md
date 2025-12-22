---
title: "Automatic Sitemap Generation"
tags: ["docs", "features", "seo"]
description: "Learn how Marmite automatically generates sitemaps for better SEO"
authors: ["rochacbruno"]
---

# Automatic Sitemap Generation

> [!NOTE]
> This feature was added in version **0.2.7**.

Marmite now automatically generates a `sitemap.xml` file for your website. This feature is enabled by default and helps search engines discover and index your content more effectively.

## What is a Sitemap?

A sitemap is an XML file that lists all the URLs on your website. It helps search engines like Google, Bing, and others to:
- Discover all pages on your site
- Understand your site structure
- Know when pages were last updated
- Prioritize crawling of important pages

## How It Works

When you build your site, Marmite automatically:
1. Collects all generated URLs (posts, pages, tags, authors, series, etc.)
2. Creates a standard XML sitemap following the [sitemaps.org](https://www.sitemaps.org/) protocol
3. Saves it as `sitemap.xml` in your site's root directory

## Configuration

The sitemap generation is enabled by default. You can disable it in your `marmite.yaml`:

```yaml
build_sitemap: false
```

## URL Format

The URLs in your sitemap depend on whether you have configured a base URL:

### With Base URL

When you have a `url` configured in your `marmite.yaml`:

```yaml
url: https://example.com
```

The sitemap will contain absolute URLs:
```xml
<url>
  <loc>https://example.com/getting-started.html</loc>
</url>
```

### Without Base URL

If no base URL is configured, the sitemap will contain relative URLs:
```xml
<url>
  <loc>/getting-started.html</loc>
</url>
```

> [!NOTE]
> For better SEO, it's recommended to configure a base URL so your sitemap contains absolute URLs.

## What's Included

The sitemap includes URLs for:
- Homepage (`index.html`)
- All published posts
- All pages
- Tag archive pages
- Author pages
- Series pages
- Stream pages (except draft)
- Archive pages by year
- Index pages for tags, authors, series, streams, and archives

## What's Excluded

The following are NOT included in the sitemap:
- Draft posts (content in the draft stream)
- RSS/JSON feed URLs
- Static assets (CSS, JS, images)
- Source markdown files

## Verifying Your Sitemap

After building your site, you can verify the sitemap:

```bash
# Build your site
marmite input_folder output_folder

# Check if sitemap was created
ls output_folder/sitemap.xml

# View the first few entries
head -20 output_folder/sitemap.xml
```

## Submitting to Search Engines

Once your site is deployed, you can submit your sitemap to search engines:

### Google Search Console
1. Go to [Google Search Console](https://search.google.com/search-console)
2. Select your property
3. Go to "Sitemaps" in the sidebar
4. Enter `sitemap.xml` and submit

### Bing Webmaster Tools
1. Go to [Bing Webmaster Tools](https://www.bing.com/webmasters)
2. Select your site
3. Go to "Sitemaps" in the sidebar
4. Submit your sitemap URL

### robots.txt

You can also add your sitemap to your `robots.txt` file:

```
User-agent: *
Allow: /

Sitemap: https://example.com/sitemap.xml
```

## Customizing the Sitemap

If you need to customize the sitemap format, you can override the default template by creating a `templates/sitemap.xml` file in your project:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{%- for url in sitemap_urls %}
  <url>
    <loc>{{ url }}</loc>
    <changefreq>weekly</changefreq>
    <priority>0.8</priority>
  </url>
{%- endfor %}
</urlset>
```

## Example Output

Here's what a typical sitemap looks like:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://example.com/index.html</loc>
  </url>
  <url>
    <loc>https://example.com/getting-started.html</loc>
  </url>
  <url>
    <loc>https://example.com/about.html</loc>
  </url>
  <!-- ... more URLs ... -->
</urlset>
```

## Integration with File Mapping

If you need more control over your sitemap, you can disable automatic generation and use the [[file-mapping-feature|File Mapping Feature]] to copy a custom sitemap:

```yaml
build_sitemap: false
file_mapping:
  - source: custom-sitemap.xml
    dest: sitemap.xml
```

---

The automatic sitemap generation feature makes it easy to improve your site's SEO without any extra configuration or manual work.