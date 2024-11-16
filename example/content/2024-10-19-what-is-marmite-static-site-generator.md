---
title: What is marmite
tags: docs
authors: rochacbruno
---

**Marmite** is a simple, easy and opinionated static site generator, 
probably the easiest and simple to use.

**Marmite** is written in **Rust** so it is very fast and everything is included
in a single binary.

You can use it to generate a static blog, starting with the built-in **theme**
and then fully customize if you want a more personalized website.

To generate your static site the only thing you need is a folder with some
markdown files and `marmite`

assuming you have a folder called `mycontent` containing files with `.md` extension
such as `about.md,first-post.md,second-post.md`

```console
$ marmite mycontent mysite

Generated /mysite/about.html
Generated /mysite/first-post.html
Generated /mysite/second-post.html
...

Site generated at: /mysite
```

That is all you need to have a blog generated, now you just need to take the 
`mysite` folder and publish it in a webserver, read more on [hosting](./hosting.html).

## Layout

By default the site is generated using marmite embedded theme (this one you are reading right now)
it is based on picocss and supports ligh/dark themes.

<details>
<summary> <strong>CLICK HERE</strong> TO SEE SOME SCREENSHOTS </summary>

The following screenshots are using the default embedded
templates (from [/example](https://github.com/rochacbruno/marmite/blob/main/example) folder)

**Light Mode**

Index:

![Index Light](https://github.com/rochacbruno/marmite/raw/main/assets/screenshots/index-light.png)

Content:

![Post Light](https://github.com/rochacbruno/marmite/raw/main/assets/screenshots/post-light.png)

**Dark mode**

Index:

![Index Dark](https://github.com/rochacbruno/marmite/raw/main/assets/screenshots/index-dark.png)

Content:

![Post Dark](https://github.com/rochacbruno/marmite/raw/main/assets/screenshots/post-dark.png)

</details>

You can of course, customize your own look and feel by adding `templates` and `static` files to 
your `mycontent` folder, read more on [customizing templates](./customizing-templates.html).

---

## Content Types

Marmite separates content in two kinds, **posts** and **pages**.

An **opinionated** decision of marmite is how it makes the distinction.

### Posts

If content has a **date** it is a **Post**!

If the `file.md` has a **FrontMatter** (metadata on its first lines) defining a
`date: YYYY-MM-DD` field, or the date field is extracted from the file name `YYYY-MM-DD-file.md`
then marmite will consider it as a **post**.

Posts are shown on `index.html` page sorted by date, and also shown on `tag/{tag}.html` page,
and included on the `RSS` and `JSON` **feeds**.

### Pages

If the markdown file doesn't define a date, then `marmite` can't list it on index or feeds, because
it doesn't know where to include it in the chronological order, so it makes sense to render this content
as a page named `{slug}.html` and make it accessible only via the link directly, so it can optionally
added to the main menu or linked in other content.

## Menu

By default marmite includes 3 items in the main menu:

**Pages** -> `pages.html`

  : List of pages in alphabetical order.

**Tags** -> `tags.html`

  :List of tags and a link to each tag group page.

**Archive** -> `archive.html`

  :List of YEAR and link to each year group page.

Menu can be optionally customized in the configuration file, it is possible
to add any **post**, **page** or external **link** to the menu.

## Authors 

A page named `authors.html` [[authors]] is also rendered and can be included in the menu,
it groups all content for authors.

## Streams

Streams are a way to have separate index on the site, you can check all available
streams on `streams.html` page [[streams]].
  
## Metadata

On each markdown file it is possible (and optional) to define metadata on the **FrontMatter**,
the first lines of the file separated by `---`.

```markdown
---
field: value
---

# title

Content
```

`marmite` supports 5 fields:

**title**

  : str: Title of the post  
  **default**: extracted from the first line of markdown.

**description**

  : str: Description text (for listing and RSS)  
  **default**: extracted from the content.

**slug** 

  : str: this-is-the-post-slug    
  **default**: slugfied `title` or `filename`.

**date**

  : str: `YYYY-MM-DD`  
  **formats** `YYYY-MM-DD`, `YYYY-MM-DD HH:MM`, `YYYY-MM-DD HH:MM:SS`  
  **default**: extracted from filename or null.

**tags** 

  : Comma separated list of tags, or YAML list of tags  
  **formats**  
    ```yaml
    tags: tag1, tag2, tag3
    tags:
      - tag1
      - tag2
    ```
    **default** empty
  
**authors** 

  : Single author, comma separated list of authors, or YAML list of authors  
  **formats**  
    ```yaml
    authors: username1
    authors: username1, username2, username3
    authors:
      - username1
      - username2
    ```
    **default** empty  
    **important** authors are extracted from the username, and if there is a mathing author in the config file then the data is used to build the `author-{username}.html` page.

**stream** 

  : str: something  
  **default**: index  
  **important**: Stream is used to define a separate index for the content, `something.html` will be the list of contents for a stream.

**card_image**

  : Image url to use as social card image `og:image`  
  **format**  `./media/file.png` or `https://path/to/img.jpg`  
  **default** banner_image or first image extracted from html content or config card_image, or None.

**banner_image**

  : Image url to use the top banner on content page  
  **format**  `./media/file.png` or `https://path/to/img.jpg`  
  **default** empty

**extra**

  : arbitrary extra `key:value` pair in YAML format (for template customization)  
  **format**
    ```yaml
    extra:
      math: true
      mermaid: true
    ```
    **important**: the above example shows the keys supported by the default theme.  



## Media

Images can be added using the normal markdown tag, marmite doesn't have shortcodes yet.

For local images you have to put the files in a folder named `media` in the content folder.

```markdown
# content with media

![Image here](media/subfolder/image.png)
```

Marmite will copy your `media` folder to the output site, it is recommended to use `./media` as
the URL for relative media.
  
## Site Config
  
Optionally, a file named `marmite.yaml` inside your content folder (together with your .md files)
can be used to customize configuration.

> `--config file.yaml` can also be passed directly to the CLI.

example:

```yaml
name: My Blog
tagline: Poems, Essays and Articles
url: https://mysite.com/blog
menu:
  - ["About", "about.html"]
  - ["Projects", "projects.html"]
  - ["Contact", "contact.html"]
  - ["Github", "https://github.com/rochacbruno"]
```

Other options are available and can be viewed on [repository](https://github.com/rochacbruno/marmite/blob/main/example/marmite.yaml)

Use `--generate-config` in the CLI to create a default config file.

## Theme customization

The embedded templates are created with [picocss.com](https://picocss.com/) and 
it is easy to customize, just put a `custom.css` and a `custom.js` 
in your root folder and use anything that pico supports or just be creative with css.

If customizing the css and js is not enough then you can create your own theme.

## Creating a new Theme

To create a new theme is very simple, you just need to add to your content folder
the `templates` and `static` directories and then customize in the way you like.

Use `--start-theme` on the CLI to start a new theme from the built-in theme.

To learn more about how to create a new theme check this post:

[Customizing Templates](customizing-templates.html)

## More features

There are more to come, marmite will include soon support for the most simple and 
popular comment systems.

Also, on of the goals is to integrate with **ActivityPub** via the JSON feed and
**Hatsu**.

If you have ideas please open issues on the repository.

That's all!

[Read the Docs](tag-docs.html)

