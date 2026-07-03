---
title: "Media Organization with Slug-Based Subfolders"
tags: ["docs", "features", "media", "images"]
description: "Organize media files in per-content subfolders and reference them with the @/ shorthand in markdown"
authors: ["rochacbruno"]
---

# Media Organization with Slug-Based Subfolders

Marmite supports organizing media files in subfolders named after your content's slug. This keeps media tidy for sites with many images per post and provides a convenient `@/` shorthand for referencing those files.

## Media Subfolder Discovery

Banner and card images can live as flat files in `media/` with the slug as a filename prefix (e.g., `media/my-post.banner.jpg`), in a subfolder named after the slug inside `media/`, or in a `media/` directory inside a content subfolder:

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

Or, when using content subfolders (e.g., for translations), media can live alongside the content:

```
content/
  my-post/
    my-post.md
    pt-meu-post.md
    media/
      banner.jpg          # Shared by all files in the subfolder
      card.png
```

Marmite checks for `banner.{ext}` and `card.{ext}` files automatically. The lookup order is:

1. `media/{slug}.banner.{ext}` (flat file in global media)
2. `content/{slug}/media/banner.{ext}` (content subfolder media - takes precedence over global)
3. `content/media/{slug}/banner.{ext}` (global media subfolder)
4. `content/{subfolder}/media/banner.{ext}` (generic fallback - shared by all files in the subfolder)

Content subfolder media takes precedence over global media. A generic `banner.jpg` in a content subfolder's media directory is shared by all `.md` files in that subfolder, so translations automatically inherit the base content's banner without needing their own copy.

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

## Media with Content Subfolders

When using content subfolders (for example, for multilingual content), you can place a `media/` directory inside the content subfolder instead of using the global `content/media/` folder:

```
content/
  travel-photos/
    travel-photos.md
    pt-fotos-de-viagem.md
    es-fotos-de-viaje.md
    media/
      banner.jpg
      card.png
      photo1.jpg
```

This keeps media files close to the content that uses them. The `media/` directory inside a content subfolder is automatically copied to `output/media/{subfolder_name}/` during the build.

A generic `banner.jpg` or `card.png` (without a slug prefix) in the subfolder's media directory is shared by all `.md` files in that subfolder. This is useful for translations - all language versions of a post inherit the same banner image without needing separate copies or frontmatter overrides.

Any individual file can still override the shared media by setting `banner_image` or `card_image` in its own frontmatter, or by having a slug-specific file like `media/{slug}.banner.jpg`.
