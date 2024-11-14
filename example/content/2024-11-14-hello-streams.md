---
stream: alt
---

# Hello Streams

This content does not show on index page  
and also does not show on main rss feed.

Because this content defines a custom stream on the `frontmatter`

```markdown
---
stream: alt
---
# Alternative Stream

This will show only on alt.html  
and will be listed only on `alt.rss`

```

You can name `stream:` anything you want, it is useful to have a separate index to add to the menu  

This article will show up on [[alt]] stream.

All streams available on the site will be listed on [[streams]] group page.

> ![IMPORTANT]  
> Streams are rendered before the content
> So if you have a conflicting slug on your content, it will override the stream page.
