## Content Types and Taxonomy

Marmite separates content in two kinds, **posts** and **pages**.

An **opinionated** decision of marmite is how it makes the distinction.

### Posts

If content has a **date** it is a **Post**!

Instead of having to mark if a content is a page via metadata, **Marmite** takes the 
simplicity to just decided based on if the content is chronological or static.

#### Date discovery

There are 2 ways to tell **Marmite** that your content is a **Post**:

- Add `date: YYYY-MM-DD` to the **frontmatter**
    ```markdown
    ---
    date: 2024-10-20
    ---
    # Hello
    ```
- Name your file with `YYYY-MM-DD-` prefix
    ```console
    $ ls mycontent
    2024-10-20-hello-world.md
    2024-10-19-another-post.md
    ```

> The date format on frontmatter can be any of `YYYY-MM-DD`, `YYYY-MM-DD H:M` or `YYYY-MM-DD H:M:S`, the filename only supports `YYYY-MM-DD-`.

#### Where posts are listed?

- By default on `/index.html` ordered by **date** (newest first)
  - If the content defines a custom stream, then the post is not listed on index, 
    and appears only on the custom stream.
      `content.md`
      ```
      ---
      stream: notes
      ---
      # my first note
      ```
      This post will not show on `index.html` but will on `/notes.html`  
      This post will not show on `index.rss` but will on `/notes.rss`
- Taxonomies
  - `/tags.html` will list content grouped by tags
    - `/tag-{name}.html` will list content for the `name` tag.
  - `/archive.html` will list content grouped by year
    - `/archive-{year}.html` will list content for the `year`
  - `/authors.html` will list content grouped by author
    - `/author-{username}.html` will list content by author `username`

#### Formatting

**Marmite** parses the markdown content using **CommonMark** and **Github Flavoured Markdown**, it allows raw HTML and adds some extensions to enable Wikilinks, Obsidian Links, Backlinks, Spoilers, Footnotes and the built-in theme supports **mermaid** and **math** syntax.

You can see examples on [[markdown-format]]

#### Metadata

On each markdown file it is possible (and optional) to define metadata on the **FrontMatter**, the first lines of the file separated by `---`.

```markdown
---
field: value
---

# title

Content
```

`marmite` supports the following fields:

**title**

  : str: Title of the post  
  **default**: extracted from the first line of markdown.

**description**

  : str: Description text (for listing and RSS)  
  **default**: extracted from the content first sentence.

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
    author: username1
    authors: username1, username2, username3
    authors:
      - username1
      - username2
    ```
    **default** empty  
    **important** authors are identified by the username, and if there is a matching author in the config file then the data is used to build the profile page on `author-{username}.html` with avatar, bio and links.

**stream** 

  : str: something  
  **default**: index  
  **important**: Stream is used to define a separate index for the content, `something.html` will be the list of contents for a stream.

**card_image**

  : Image url to use as social card image `og:image`, this is the image that shows when you share your link on mastodon, bluesky, linkedin etc.   
  **format**  `media/file.png` or `https://path/to/img.jpg`  
  **default** banner_image or first image extracted from html content or config card_image, or None.

**banner_image**

  : Image url to use the top banner on content page  
  **format**  `media/file.png` or `https://path/to/img.jpg`  
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

**pinned**

  : Boolean `true` or `false` indicating if content is pinned on top
    of its stream page.
  **default** false


### Pages

If the markdown file doesn't define a date, then **Marmite** can't list it on index or feeds, because it doesn't know where to include it in the chronological order, so it makes sense to render this content as a page named `{slug}.html` and make it accessible only via the link directly, so it can optionally added to the main menu or linked in other content.

### Menu

By default marmite includes items in the main menu:

**Tags** -> `tags.html`

  :Content grouped by tags

**Archive** -> `archive.html`

  :Content grouped by year

**Authors** -> `authors.html`

  :Content grouped by author

**Streams** -> `streams.html`

  :Content grouped by stream

**Pages** -> `pages.html`

  : List of pages in alphabetical order.

#### Customizing the menu

The menu is defined on `marmite.yaml` config file and is a list of tuples 
containing name and URL

`marmite.yaml`
```yaml
menu:
  - - About
    - about.html
  - - Github
    - https://github.com/me
```

You can add anything you want to the menu and the name allows HTML.

>>>
When `search_enabled: true` is on your config marmite will also add an icon to open the search spotlight on the menu, also the light/dark mode switch.
>>>

### Taxonomies and groups
 
Taxonomy is a way to group or separate content based on marks, **Marmite** allows you to add the following marks to the content.

#### Group content together

##### Authors 

Add `author: username` or `authors: username, other` to the content.

A page named `authors.html` [[authors]] is rendered and can be included in the menu,
it groups all content for authors.

##### Tags

Add `tags: one,two` to the content.

A page named `tags.html` [[tags]] is rendered and can be included in the menu, or linked in any other page, this goups content by tags.

##### Archive

Add `data: YYYY-MM-DD` to the content and a page named `archive.html` [[archive]] is rendered grouping content by year.

Each of the pages above will also have its equivalent `.rss` feed.

#### Separating content

##### Streams

Add `stream: name` to the content.

Streams are a way to have separate index on the site, you can check all available
streams on `streams.html` page [[streams]].

Each stream have its separate RSS feed and listing page.

---


### Media

Images can be added using the normal markdown tag, marmite doesn't have shortcodes yet.

For local images you have to put the files in a folder named `media` in the content folder.

```markdown
# content with media

![Image here](media/subfolder/image.png)
```

Marmite will copy your `media` folder to the output site/
  
### Site Config
  
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
extra:
  colorscheme: gruvbox
  fediverse_verification: https://mastodon.social/@me
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

---

[Read the Docs](tag-docs.html)
