---
date: 2024-10-16
tags: docs, hosting
---
## Hosting

Marmite genetates a static site, so you can host it in any web server.

Examples:

- Github pages
- Gitlab pages
- Netlify
- Vercel
- Nginx
- Apache

### Github Pages

This is the easiest and cheapest option to publish your static blog,
you need a **Github Repository** containing a `content` directory and a `marmite.yaml`


Fork this template repository https://github.com/rochacbruno/make-me-a-blog and give a meaninful name like `blog` to
the forked repo.

Or start from scratch! it is very simple!

---

Create a new repository and the following files.

```bash
.
|_ marmite.yaml
|_ content
  |_ 2024-10-22-my-first-post.md
  |_ about.md
```

Edit `marmite.yaml` to add your preferences.

```yaml
name: My Blog
tagline: Poems, Essays and Articles
url: https://YOURUSER.github.io/REPONAME
menu:
  - ["About", "about.html"]
  - ["Pages", "pages.html"]
  - ["Tags", "tags.html"]
  - ["Follow me", "https://mastodon.social/@YOURUSER"]
  - ["Github", "https://github.com/rochacbruno"]
```

Now you need to tell github actions to go inside your repo root and run

```
marmite . site
```

And then publish the `site/` directory as the github page for your repo.

You can automate that:

First access https://github.com/YOURUSER/REPONAME/settings/pages and set the
pages source to **Github Actions**

Then add a workflow to your repository.

`.github/workflows/main.yaml`
```yaml
name: GH Pages Deploy

on:
  push:
    branches: [main]

  # Allows you to run this workflow manually from the Actions tab
  workflow_dispatch:

# Sets permissions of the GITHUB_TOKEN to allow deployment to GitHub Pages
permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: "pages"
  cancel-in-progress: false

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout 🛎
        uses: actions/checkout@v4

      - name: Install marmite 🫙
        run: cargo install marmite

      - name: Build site 🏗️
        run: marmite . site --debug

      - name: Setup Pages
        uses: actions/configure-pages@v5

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: 'site'

      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

Now commit and push to the main branch and wait for your blog to be published at

https://YOURUSER.github.io/REPONAME


Read [Customizing Templates](./customizing-templates.html) to learn how 
to customize the look and feel of your blog.
