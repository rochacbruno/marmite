# This is a page with no front matter

The page must work even without `frontmatter` specified, the `slug` to build the
 `url` is extracted from the filename and page `title` is extracted from the
first line of the markdown.

By default this page doesn't show in menu, unless explititly added to the `menu:`
in the configuration file `marmite.yaml`

```yaml
menu:
  - ["My Page", "no-metadata.html"]

```

If the menu is not specified, than `marmite` will add `tags`, `archive` and `pages`
listing to the menu.
