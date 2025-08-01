---
title: "Shortcodes Demo"
tags: ["features", "shortcodes", "documentation"]
authors: ["marmite"]
---

# Shortcodes Demo

> [!IMPORTANT]
> This is a **Beta** feature currently available only on the main branch. It has not been released in a stable version yet.

This post demonstrates the new shortcodes feature in Marmite.

## YouTube Video

Here's a video about static site generators:

```
<!-- .youtube id=dQw4w9WgXcQ -->
```

<!-- .youtube id=dQw4w9WgXcQ -->

You can also specify custom dimensions:

```
<!-- .youtube id=dQw4w9WgXcQ width=400 height=300 -->
```

<!-- .youtube id=dQw4w9WgXcQ width=400 height=300 -->

## Table of Contents

This page's table of contents:

```
<!-- .toc -->
```

<!-- .toc -->

## Authors List

All authors on this site:

```
<!-- .authors -->
```
<!-- .authors -->

## Streams List

Available content streams:

```
<!-- .streams ord=desc items=5 -->
```

<!-- .streams ord=desc items=5 -->

## Series List

All content series:

```
<!-- .series -->
```

<!-- .series -->

With parameters:

```
<!-- .series ord=desc items=3 -->
```

<!-- .series ord=desc items=3 -->

## Posts List

Recent posts:

```
<!-- .posts -->
```
<!-- .posts -->

With custom parameters:

```
<!-- .posts ord=asc items=5 -->
```
<!-- .posts ord=asc items=5 -->


## Pages List

All pages:

```
<!-- .pages -->
```

<!-- .pages -->

## Tags List

All tags with post counts:

```
<!-- .tags -->
```

<!-- .tags -->

Limited tags:

```
<!-- .tags ord=desc items=5 -->
```

<!-- .tags ord=desc items=5 -->


## Social Networks

```
<!-- .socials -->
```

<!-- .socials -->

## Card Display

Card shortcodes allow you to display content previews with image, title and description:

```
<!-- .card slug=tag-markdown -->
```

<!-- .card slug=tag-markdown -->

You can also display cards for other content types:

```
<!-- .card slug=getting-started -->
```

<!-- .card slug=getting-started -->

Card for author:

```
<!-- .card slug=author-rochacbruno -->
```

<!-- .card slug=author-rochacbruno -->

Card for stream:

```
<!-- .card slug=stream-alt -->
```

<!-- .card slug=stream-alt -->

Card for series:

```
<!-- .card slug=series-python-tutorial -->
```

<!-- .card slug=series-python-tutorial -->

Card for page:

```
<!-- .card slug=about -->
```

<!-- .card slug=about -->


## Conclusion

Shortcodes make it easy to add dynamic content to your static site! With 8 builtin shortcodes, you can create rich, interactive content while keeping your markdown clean and readable.