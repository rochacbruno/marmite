>>>
I'm a big user of other SSGs but it is frequently frustrating that it takes so much setup to get started.  
Just having a directory of markdown files and running a single command sounds really useful.  
&mdash; Michael, marmite user.
>>>

<div style="padding-bottom:0;">
Try a different colorscheme:   <select name="colorscheme" class="colorscheme-toggle"><option value="default">default</option></select><span class="theme-toggle secondary" title="dark mode">&#9789;</span><br>

Or try an alternative [theme](https://marmite.blog/theme_template/)
</div>

## Quick Start

### Install

```bash
curl -sS https://marmite.blog/install.sh | sh
```

<small> or check [[Installation]] for more install options </small>

### Start blogging

```bash
# Create a new blog (or simply use a folder with markdown files)
marmite myblog --init-site \
    --name Mysite \
    --tagline "My Articles and Notes" \
    --colorscheme nord \
    --toc true \
    --enable-search true 

# Create a new post
marmite myblog --new "My First Blog Post" -t "new,post"

# Build and serve the blog
marmite myblog --serve
```