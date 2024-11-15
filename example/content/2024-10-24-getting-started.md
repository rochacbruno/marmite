---
title: Getting started
slug: getting-started
tags: docs
banner_image: media/getting_started.jpg
authors: rochacbruno
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
you just need a directory containing markdown files.

So start by creating a new directory for your content.

```bash
mkdir myblog
```

On this example lets add an **about** page:

`myblog/about.md`
```markdown
# About me

I am a person that loves simplicity and writing content in 
Markdown format.

Email me at <me@example.com>
Follow me on <https://social.example.com/@me>
```

And a **hello world** post:

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

Emojis :smile:

[1]: https://github.com/rochacbruno/marmite
```

The main difference from those 2 files is the fact that the first one **about.md**
doesn't have a `date:` metadata and since marmite can't detect its date, it considers
it a **page**. 

The second, **hello-world.md** has a `date: ...` on its **FrontMatter** so
marmite considers it a **chronological** content, a **post** and will list it 
on your blog front-page.

Learn more about [Markdown Formatting]

### Generating the site

>>>
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
Config loaded from: defaults
Generated /tmp/site/index.html
Generated /tmp/site/pages.html
Generated /tmp/site/hello-world.html
Generated /tmp/site/about.html
Generated /tmp/site/404.html
Generated /tmp/site/tag-hello.html
Generated /tmp/site/tag-world.html
Generated /tmp/site/tags.html
Generated /tmp/site/static/*{css,js,fonts}
Site generated at: /tmp/site/
```

And that's all! you have full working blog on the `site` folder, as **marmite**
generates a **flat html** website, you can open directly in your browser.

To see your site you can open the `index.html` in your browser, type `Ctrl + O`
on your keyboard and your browser will let you navigate to the folder and then
select the `index.html` file.


<figure>
  <figcaption>Ctrl + O</figcaption>
  <img src="./media/screenshots/browser_open.png" width="300">
</figure>


Then your site will look like:

<figure>
  <figcaption>Home Page</figcaption>
  <img src="./media/screenshots/index.png" width="600">
</figure>

<figure>
  <figcaption>About Page</figcaption>
  <img src="./media/screenshots/about.png" width="600">
</figure>

<figure>
  <figcaption>Post Light Mode</figcaption>
  <img src="./media/screenshots/hello-world.png" width="600">
</figure>

<figure>
  <figcaption>Post Dark Mode</figcaption>
  <img src="./media/screenshots/hello-world-dark.png" width="600">
</figure>

<figure>
  <figcaption>Tags</figcaption>
  <img src="./media/screenshots/tags.png" width="600">
</figure>

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

## Media

You gonna want some nice images in your blog posts, it is easy and simple 
to add it with marmite.

Create your `myblog/media` folder and include any image or video you want inside it
then in your markdown content just reference using the relative path.

`myblog/hello-world.md`
```markdown
---
date: 2024-10-25
tags: hello,world
---
# Hello World

This is my photo:
![me](./media/myphoto.png)
```

<figure>
  <figcaption>Media</figcaption>
  <img src="./media/screenshots/media.png" width="600">
</figure>


**IT IS IMPORTANT** that you always link internally starting with `./`, 
following this rule you can link to any `./media/` file or any other page like 
`./about.html`, remember, the generated site is a **flat html**, there is no
subfolders.

> [!INFO]  
> You **can create** subfolders in the media directory, that is useful to organize
> files by topic or post.

## Organizing content

For now all your markdown files are located on `myblog/` root folder,
that is enough if you are going to stick with the defaults, but as soon you
start customizing templates and configuration you gonna need to organize your 
contents.

**marmite** generated a **flat html** site, that means you can't have subpaths,
your content will always be served from the top level.

But for better organization of your project **YOU CAN OPTIONALLY** move all your **markdown**
files to a subfolder called `content/`

```bash
myblog/
   |_ content/
         |_ about.md
         |_ hello-world.md
         |_ media/
             |_ myphoto.png
```

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

<figure>
  <figcaption>Config</figcaption>
  <img src="./media/screenshots/config.png" width="600">
</figure>


You can see all available options on the Marmite [example config] or reading the [docs]

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

## Enabling Search

Marmite can generate static search index as a `JSON` file 
and then using Javascript library [Fuse] provide full text search for posts and pages.

To enable this feature add to your `marmite.yaml`

```yaml
enable_search: true
```

## Special pages and Fragments

There are some contents that are considered **special** as those are 
not regular posts or pages, right now there are 2:

Hero

  : A banner that shows as the first content in your home page  
    Marmite will look for a file named `_hero.md` within your content folder.

404

  : The page that will show for Not Found error  
    Marmite will look for a file named `_404.md` within your content folder.  
    if not found, marmite will generate a default.


<figure>
  <figcaption>Hero Section Enabled</figcaption>
  <img src="./media/screenshots/hero.png" width="600">
</figure>

## Layout Customization

If you want to keep using the default theme but wants to customize little parts
of its `CSS` and add additional `JS` then you can do it with the **easy mode**

### Easy mode

#### Custom CSS

Just create a new file `myblog/custom.css` and put any CSS you want inside it,
if you are customizing the default template take a look at [PicoCSS] docs.

Example:
`myblog/custom.css`
```css
body {
  background-color: red;
}
```

<figure>
  <figcaption>Custom CSS</figcaption>
  <img src="./media/screenshots/custom.png" width="600">
</figure>


#### Custom JS

The same as above, just create a `custom.js` and add any Javascript,
notice that the `custom.js` file is loaded at the bottom of the html files.

`myblog/custom.js`
```javascript
console.log("Hello from marmite");
```

### Advanced mode

Choose this option if your intention is to **fully** customize the default theme
or **start** a new theme from **scratch**.

#### Custom templates

To fully customize the templates you need a `templates` folder and add all the 
required templates as described on [Customizing Templates]

If you don't want to create the templates manually, you can dump the 
embedded templates in your project folder.

```bash
marmite myblog site --init-templates
```

The above command will create a `myblog/templates` folder with all the required
templates and you can freely customize.

The templates are written in [Tera].

#### Custom theme

Theme is generally the templates + Style, so in addition to custom templates
you need all the static files.

If you wrote your templates from **scratch** then you are free to refer
to any static file you want, create a folder named `static` and put any 
`css`, `js`, `fonts` etc inside it.

> [!IMPORTANT]
> marmite generates **flat html** website, all static file url
> will be relative like `./static/style.css` `./static/fonts/font.woof` etc.

If you don't want to create all static files by hand, but want to reuse the 
embedded theme, you can run.

```bash
marmite myblog site --start-theme
```

The above command will output the `myblog/static` folder and then you can
freely customize.

## Project structure

With contents and a full customized theme your repository will look like:

```bash
myblog/
   |_ content/                     # Markdown content
   |     |_ _404.md
   |     |_ _hero.md
   |     |_ about.md
   |     |_ hello-world.md
   |     |_ media/                 # Images, Videos, PDFs
   |         |_ myphoto.png
   |_ templates/                   # Tera HTML templates
   |     |_ *.html
   |_ static/                      # Theme static files
   |     |_ *.{css,js,ttf,ico}
   |_ marmite.yaml                 # Config
   |_ custom.css                   # Optional css
   |_ custom.js                    # Optional JS
```

## Looking for help

You can ask **Marmite** related questions or suggest features on [Discussions] page

> [!NOTE]  
> **We** would love to know if you publish a site  made with marmite or 
> created a custom theme, please share on the [Discussions] page.

---

𐚮 Marmite.

[docs]: <./tag-docs.html> "Marmite Docs"
[Enabling Comments]: <./enabling-comments.html> "Enabling Comments"
[PicoCSS]: <https://picocss.com> "Pico CSS"
[Customizing Templates]: <./customizing-templates.html> "Customizing Templates"
[Tera]: <https://keats.github.io/tera/docs/#templates> "Tera Docs"
[Discussions]: <https://github.com/rochacbruno/marmite/discussions> "Github Discussions"
[Markdown Formatting]: <./markdown-format.html> "Markdown Formatting"
[Fuse]: <https://fusejs.io> "Fuse JS"
[example config]: <https://github.com/rochacbruno/marmite/blob/main/example/marmite.yaml> "Config Example"