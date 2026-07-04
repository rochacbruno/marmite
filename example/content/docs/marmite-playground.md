---
title: Introducing the Marmite Playground
slug: marmite-playground
description: "Try marmite directly in your browser. The Marmite Playground is a live editor where you can write markdown, tweak settings, and preview your site in real time - no installation required."
tags: announcement, marmite, playground, features
authors: rochacbruno
pinned: true
date: 2026-05-30
---

The **Marmite Playground** is now live at [play.marmite.blog](https://play.marmite.blog).

It is an interactive web application where you can try marmite without installing
anything. Open the playground, write some markdown, and see your site rendered
instantly in the preview panel.

Whether you are evaluating marmite for the first time or want a quick scratchpad
to test a feature, the playground is the fastest way to get started.

## How It Works

The playground gives you a split-pane interface: a code editor on the left and a
live site preview on the right. You get a full marmite project out of the box
with sample content, configuration, and a working site.

Every edit you make triggers an automatic rebuild. Change a markdown file, tweak
the `marmite.yaml` configuration, or add a new CSS file, and the preview updates
within seconds.

## What You Can Do

**Edit content** - The editor opens with a getting started guide that covers
marmite's content model (posts, pages, and fragments), frontmatter fields,
and markdown extensions like math, mermaid diagrams, alerts, and task lists.
You can create new content files, rename them, or delete them.

**Tweak configuration** - The seed `marmite.yaml` includes every configuration
field as a commented reference. Uncomment what you need, change the site name,
enable search, switch colorschemes, or configure authors and streams.

**Customize templates and styles** - Add files to the `templates/` and `static/`
folders to override the default theme. The new file dialog includes helpful
references with links to the template and content documentation.

**Switch editor themes** - The status bar at the bottom of the editor panel has a
theme selector with light and dark options (Dracula, Solarized Light, Cobalt,
and more). This is independent from the marmite site colorscheme, which you
control via `marmite.yaml`.

**Download your work** - The Download button lets you export either the source
project (your markdown, config, and templates as a `.tar.gz`) or the rendered
site (the generated HTML, ready to deploy). Upload a `.tar.gz` or `.zip` to
import an existing marmite project into the playground.

**Share your session** - Every playground session has a unique URL. Share it
with someone and they can view your site and browse your files in read-only mode.
They can also clone the session to get their own editable copy.

## Session Lifetime

Each session lives for one hour after the last file edit. After that, the session
and all its files are automatically removed. If you want to keep your work,
use the Download button to export the source project before the session expires.

## Try It Now

Open [play.marmite.blog](https://play.marmite.blog) and start building. The
playground comes pre-loaded with example content that demonstrates marmite's
features, including mermaid diagrams, KaTeX math, GitHub-style alerts, and
shortcodes.

If you already have a marmite project, upload it to the playground to preview
it or test it against the latest version of marmite.

For the full documentation, visit [marmite.blog/docs](https://marmite.blog/docs.html).
