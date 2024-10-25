---
title: Getting started
slug: getting-started
tags: docs
extra:
  mermaid: true
---
Learn how to create your blog with Marmite in minutes, you can start with zero-config 
and then customize gradually later.

## Quick Start

### Installation

Marmite is written in Rust :crab: so if you have Rust in your system you 
can use cargo to install it.

```bash
cargo install marmite
```

If you don't have Rust, then you can download the pre-build binary from
the [Github Releases page](https://github.com/rochacbruno/marmite/releases)

### Adding Content

For a simple website marmite doesn't require any specific directory structure,
you just need any folder containing markdown files.

So start by creating a new directory for your content.

```bash
mkdir myblog
```

On this example lets add an **about** page and a **hello world** post.  

`myblog/about.md`
```markdown
# About me

I am a person that loves simplicity and writing content in 
Markdown format.

Email me at <me@example.com>
Follow me on <https://social.example.com/@me>
```

`myblog/hello-world.md`
```markdown
---
date: 2024-10-25
tags: hello,world
---
# Hello World

Hello, this is my first post on my new blog generated
by Marmite[1], a very simple to use blog generator.

I can write full CommonMark and GFM Markdown.

:smile:

[1]: https://github.com/rochacbruno/marmite
```

The main difference from those 2 files is the fact that the first one **about.md**
doesn't have a `date:` metadata and since marmite can't detect its date, it considers
it a **page**. 

The second, **hello-world.md** has a `date: ...` on its **FrontMatter** so
marmite considers it a **chronological** content, a **post** and will list it 
on your blog front-page.

### Generating the site

>>>
That looks really nice!  
I'm a big user of other SSGs but it is frequently frustrating that it takes so much setup to get started.  
Just having a directory of markdown files and running a single command sounds really useful.  
&mdash; Michael, marmite user.
>>>

As said above, you just need to run marmite and point your content directory,
the usage is `marmite [input_folder] [output_folder] [options]`

So do:

```bash
marmite myblog site
```
```bash
Config loaded from defaults
Generated site/index.html
Generated site/pages.html
Generated site/tags.html
Generated site/archive.html
Generated site/about.html
Generated site/hello-world.html
Generated site/404.html
Generated site/feed.rss
Generated site/static
Site generated at: site/
```

And that's all! you have full working blog on the `site` folder.

To see your site you can open the `index.html` in your browser, type `Ctrl + O`
on your keyboard and your browser will let you navigate to the folder and then
select the `index.html` file.

### Automatic re-generation when files changes

Instead of manually executing the command every time you change your
markdown files you can tell marmite to detect the changes and regenerate.

```
marmite myblog site --watch
...
Watching for changes in folder: myblog
```

### Serving the site

Marmite comes with a built-in server, this server is not meant to use in
production, when publishing your site you are probably going to use a
webserver such as **Apache** or **Nginx**, or most probably use a free
static hosting service such as **Github pages**, **Netlify** or **CLoudflare**.

However, during the content writing you want to check how the website looks,
so the built-in server comes handy, just add `--serve`

```
marmite myblog site --watch --serve
...
Watching for changes in folder: myblog
Starting built-in HTTP server
Server started at http://localhost:8000/ - Type ^C to stop.
```

Open your browser and check http://localhost:8000/ to see your running blog

if you want to share your site with others in the same network, just 
pass `--bind "0.0.0.0:8000` and then share your local IP address.

## Configuring

marmite is designed to be **zero** config to get started, just like you
did above!

But you probably want to customize couple of things in your website to give 
your personal touch, change the look and feel, or even completely customize
its front-end if you know some **CSS/JS** magic.

### Adding your blog information

If found, marmite will load configuration from a file named `marmite.yaml`
located in the root of your input_folder, in this case:

`myblog/marmite.yaml`
```yaml
name: My Personal Blog
tagline: Things I like to share
pagination: 10
menu:
  - ["about", "about.html"]
  - ["pages", "pages.html"]
  - ["tags", "tags.html"]
  - ["archive", "archive.html"]
```

Now regenerate your site with 
```
marmite myblog site --watch --serve
```
And refresh your browser to take a look at the customizations.

You can see all available options on the Marmite [Docs]

### Enabling Comments

Most of blogs will have a comment box to receive feedback from readers,
marmite doesn't come with one as it is a static site and doesn't have the 
ability to handle dynamic data, instead, marmite allows you to plug external
commenting system providers such as **disqus** or **github**.

`myblog/marmite.yaml`
```yaml
extra:
  comments:
    title: Comments
    source: |
      <div id="#comment-box"></div>
      <script> ... </script>
```

The content of the `source` text depends on which commenting system you choose
and how it is configured.

Read more on [Enabling Comments] page to learn how to enable **Gisqus**, a
commenting system based on Github discussions.


## Layout Customization

### Easy mode

#### Custom CSS
#### Custom JS

### Advanced mode

#### Custom templates
#### Custom theme


𐚮


[docs]: <https://rochacbruno.github.io/marmite/tag/docs.html> "Marmite Docs"
[Enabling Comments]: <https://rochacbruno.github.io/marmite/enabling-comments.html> "Enabling Comments"
