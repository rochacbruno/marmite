---
date: 2024-10-15
tags: docs, comments, customization
---

# Enabling comments

Marmite as a static site generator, doesn't have commenting features,
but there are various options of external commenting systems to integrate.

Utterances

  : Use github issues as comment system  
  Requires user to login to github

**Giscus**

  : Use Github discussions as comment system  
  requires user to login to github
 
Hatsu

  : Federated to ActivityPub   
  Requires a running instance  
  Requires user to comment from a fediverse account.
  

For now the easiest system is Giscus.

## Setting up Giscus

1. First you need to have a github repository
    - if you used the starting [repo](https://github.com/rochacbruno/make-me-a-blog) you already got one.
    - You can create a repository solely for comments, the blog doesn't need to be hosted on the same repo.
2. The repository must be public, otherwise visitors will not be able to view the discussion.
3. Install [giscus app](https://github.com/apps/giscus) on your repo, otherwise visitors will not be able to comment and react.
4. The Discussions feature must be [enabled](https://docs.github.com/en/github/administering-a-repository/managing-repository-settings/enabling-or-disabling-github-discussions-for-a-repository) on your repository.


Now go to https://giscus.app/ and find the **configuration** section.

Copy the `<script ... /script>` that was presented and put it in your `marmite.yaml`

```yaml
data:
  comments:
    title: Comments
    source: |
        <script src="https://giscus.app/client.js"
        data-repo="yourrepo/blog"
        data-repo-id="sdsdsd"
        data-category="Announcements"
        data-category-id="dfsffsdfsdfsdfsdf"
        data-mapping="pathname"
        data-strict="0"
        data-reactions-enabled="1"
        data-emit-metadata="0"
        data-input-position="top"
        data-theme="preferred_color_scheme"
        data-lang="en"
        data-loading="lazy"
        crossorigin="anonymous"
        async>
        </script>
```

## Setting up other comment systems

The process will be very similar, you just need to grab the required `script` and tags.

```yaml
data:
  comments:
    title: Comments
    source: |
        <div id="comment-system"></div>
        <script src="https://commentsystem.app/foo/bar/dothethings.js"></script>
```

## Customizing the HTML template directly

Add `templates/comments.html` to your project.

```html
<article>
<header>{{site.data.comments.title | default(value="Comments") }}</header>
{{site.data.comments.source}}
</article>
```
