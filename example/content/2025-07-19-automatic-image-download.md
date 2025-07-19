---
title: Automatic Image Download for Posts
date: 2025-07-19 18:30:00
slug: automatic-image-download
description: Learn how to enable automatic banner image generation for your blog posts using Marmite's built-in image provider feature.
tags: docs, features, images, configuration
authors: rochacbruno
---

Marmite now supports automatic banner image download for your blog posts! This feature helps you maintain a consistent visual appearance across your site without manually creating images for every post.

## How It Works

When enabled, Marmite automatically downloads banner images for posts (content with dates) that don't already have one. The feature only activates for posts, not for pages, ensuring your blog content gets beautiful banner images while keeping pages unaffected.

## Configuration

To enable automatic image download, add the following to your `marmite.yaml` configuration file:

```yaml
# Image provider for automatic image download
image_provider: picsum
```

Currently, `picsum` is the only available provider, which uses [Lorem Picsum](https://picsum.photos/) to generate beautiful, deterministic placeholder images.

## Image Generation Logic

Marmite follows this logic when processing posts:

1. **Check frontmatter**: If the post already has a `banner_image` field set, no download occurs
2. **Check existing file**: If `media/{slug}.banner.jpg` already exists, no download occurs  
3. **Generate image**: Downloads a 1200x300 image from `https://picsum.photos/seed/{site-name}-{post-slug}-{tags}/1200/300`
4. **Save locally**: Saves the image as `media/{slug}.banner.jpg`

The images are deterministic - the same post slug and tags combination will always generate the same image, ensuring consistency across rebuilds.

## Customizing Your Images

If you don't like the automatically generated image for a post, you have several options:

### Option 1: Replace the Image File

Simply replace the generated image file in your `media/` directory:

```bash
# Replace with your custom image
cp my-custom-banner.jpg media/my-post-slug.banner.jpg
```

### Option 2: Set Custom Banner in Frontmatter

Add a `banner_image` field to your post's frontmatter:

```yaml
---
title: My Post Title
date: 2025-07-19 12:00:00
banner_image: /media/my-custom-banner.jpg
# ... other frontmatter
---
```

### Option 3: Change the Post Slug

If you want a different generated image, you can change the post's slug to get a new random image:

```yaml
---
title: My Post Title
date: 2025-07-19 12:00:00
slug: my-new-slug  # This will generate a different image
# ... other frontmatter
---
```

### Option 4: Change the Post Tags

Since tags are included in the image seed, you can change your post's tags to generate a different image:

```yaml
---
title: My Post Title
date: 2025-07-19 12:00:00
tags: tutorial, guide, new-tag  # Different tags = different image
# ... other frontmatter
---
```

**Note**: To regenerate an image with new tags or slug, you must first delete the existing banner image file from the `media/` directory. Marmite will only download a new image if the file doesn't already exist.

## Benefits

- **Consistent visual appearance** across all your posts
- **Zero maintenance** - images are generated automatically
- **Deterministic results** - same slug always generates the same image
- **Flexible override options** when you want custom images
- **Bandwidth efficient** - images are only downloaded once

## Example

Here's what happens when you create a new post:

```markdown
---
title: My Amazing New Post
date: 2025-07-19 15:00:00
slug: amazing-new-post
tags: tutorial, guide
---

Your post content here...
```

With `image_provider: picsum` enabled, Marmite will:
1. Generate the URL: `https://picsum.photos/seed/my-blog-amazing-new-post-tutorial-guide/1200/300`
2. Download the image
3. Save it as `media/amazing-new-post.banner.jpg`
4. Use it as the banner image for your post

This feature makes it easier than ever to maintain a professional-looking blog with minimal effort!