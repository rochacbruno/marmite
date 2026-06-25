---
title: "Media Organization with Slug-Based Subfolders"
tags: ["docs", "features", "media", "images"]
description: "Organize media files in per-content subfolders and reference them with the @/ shorthand in markdown"
authors: ["rochacbruno"]
---

# Media Organization with Slug-Based Subfolders

Marmite supports organizing media files in subfolders named after your content's slug. This keeps media tidy for sites with many images per post and provides a convenient `@/` shorthand for referencing those files.

## Media Subfolder Discovery

Previously, banner and card images had to live as flat files in `media/` with the slug as a filename prefix (e.g., `media/my-post.banner.jpg`). Now you can also place them in a subfolder named after the slug:

```
content/
  media/
    my-post/
      banner.jpg
      card.png
      diagram.svg
      photo.png
  2024-01-15-my-post.md
```

Marmite checks the subfolder for `banner.{ext}` and `card.{ext}` files automatically. The lookup order is:

1. `media/{slug}.banner.{ext}` (flat file - existing behavior, takes precedence)
2. `media/{slug}/{kind}.{ext}` (subfolder - new)

Existing sites using flat files are unaffected.

## The `@/` Shorthand

Inside markdown content, use `@/` to reference files in your content's media subfolder. Marmite replaces `@/` in image and link attributes with `media/{slug}/` in the rendered HTML.

```markdown
---
title: My Post
slug: my-post
---

Here is a photo from the trip:

![Sunset](@/sunset.jpg)

Download the [full resolution version](@/sunset-full.jpg).
```

The rendered HTML will contain `src="media/my-post/sunset.jpg"` and `href="media/my-post/sunset-full.jpg"`.

### What gets replaced

The `@/` replacement only applies to `src` and `href` attributes in the final HTML. It does **not** affect:

- **Plain text** - Writing `@/` in a paragraph leaves it as-is
- **Code blocks** - Documenting the feature with `` `@/example.png` `` or fenced code blocks works correctly
- **Fragment files** - Files prefixed with `_` (like `_hero.md`) do not get `@/` replacement since they are shared across content and have no slug context

### Custom media path

If you configure a custom `media_path` in `marmite.yaml`, the `@/` shorthand uses it:

```yaml
media_path: assets
```

With this config, `@/photo.png` in a post with slug `my-post` becomes `assets/my-post/photo.png`.

## Putting It Together

A typical workflow for a media-heavy post:

1. Create your post: `content/2024-06-15-travel-photos.md`
2. Create the media subfolder: `content/media/travel-photos/`
3. Place your images there: `banner.jpg`, `card.png`, `photo1.jpg`, `photo2.jpg`
4. Reference images in markdown with `@/`:

```markdown
---
title: Travel Photos
slug: travel-photos
date: 2024-06-15
---

![Beach at sunset](@/photo1.jpg)

![Mountain view](@/photo2.jpg)
```

Marmite will:
- Automatically discover `media/travel-photos/banner.jpg` as the banner image
- Automatically discover `media/travel-photos/card.png` as the card image
- Replace `@/photo1.jpg` with `media/travel-photos/photo1.jpg` in the rendered HTML
- Copy the entire `media/travel-photos/` folder to the output directory
