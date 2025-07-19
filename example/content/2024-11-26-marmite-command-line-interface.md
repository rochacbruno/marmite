---
tags: docs
---
# Marmite Command Line Interface

Marmite CLI is designed to be always executed pointing to an `input_folder` where the markdown content is located,
and besides generating the site it comes with other useful features.

The basic usage is very simple:

```console
$ marmite input_folder
Site generated at input_folder/site
```

Or you can specify the output folder:

```console
$ marmite input_folder output_folder
Site generated at output_folder
```

> Check `marmite --help` for a complete list.

Besides this basic usage Marmite comes with other useful arguments.

## Initialize a Project

Marmite can gerate a site from any folder containing markdown files,
however if you need to initialize a project from the scratch, marmite
can generate the recommended structure.


```console
$ marmite --init-site myblog
Config file generated: "myblog/marmite.yaml"
```

The site will be created with the default configuration, it is possible to [[#Override Configuration]] passing 
each parameter e,g: `--name MySite`.

The generated structure looks like:

```console
$ tree myblog
myblog
├── content
│   ├── media/
│   ├── 2024-11-26-welcome.md
│   ├── about.md
│   ├── _404.md
│   ├── _announce.md
│   ├── _comments.md
│   ├── _hero.md
│   ├── _markdown_footer.md
│   ├── _markdown_header.md
│   ├── _references.md
│   └── _sidebar.example.md
├── custom.css
├── custom.js
└── marmite.yaml
```

Read more on [[Using Markdown to Customize Layout]] and [[Customizing Templates]]

## Add new content 

To add new content you just need to create new markdown files on the `content` folder,
you can use your preferred tools and editors for that task.

Marmite provides the utility to automate creating the file via CLI, which is useful
to have the **date** attribute set automatically.

```console
$ marmite myblog --new "My first post" -t "post,content"   
myblog/content/2024-11-26-12-34-27-my-first-post.md
```

Pass `-p` to create a page instead of a post.  
Pass `-e` to immediately open the file on the `$EDITOR`


## Generate the site

```console
$ /marmite myblog -v
Site generated at: myblog/site/
```

Or specify a custom `output_folder`:
```console
$ /marmite myblog /var/www/myblog -v
Site generated at: /var/www/myblog
```

> It is possible to [[#Override Configuration]] during site generation, for example, passing `--pagination 5` to change how many posts to list per page.


### Rebuild when content changes

Marmite can watch for changes on your project and rebuild the site automatically,
it is required to inform the output folder for this functionality.

Use `-w` or `--watch`

```console
$ /marmite myblog /var/www/myblog -w
Site generated at: /var/www/myblog
Watching for changed on: myblog/
```

### Serving the site 

Marmite generates a flat site, which means you can open it directly on your browser (with some limitations) 
or use any web server to serve it.

Marmite comes with a built-in server to use only locally.

Use `--serve` to start the server.

```console
$ /marmite myblog /var/www/myblog -w
Site generated at: /var/www/myblog
Watching for changed on: myblog/
Starting built-in HTTP server...
Server started at http://0.0.0.0:8000/ - Type ^C to stop.
```

If you want to change the address use `--bind ip:port`

> [!IMPORTANT]  
> The built-in server is not suitable for production, when deploying use a webserver such as [Nginx] or read the [[hosting]] guide to learn how to deploy to Github pages, Gitlab pages, Codeberg Pages, Netlify and more.


## Start a new theme

**Marmite** comes with a default theme that is optimized for readabily on blogs,
you can read more on [[Why to use marmite]] article.

The built-n theme comes with various colorschemes and you can set the desired colorscheme
on the `marmite.yaml` file or passing `--colorscheme` on the CLI.

The built-in theme can be customized using the `custom.css` placed on your root folder,
and individual templates can be overriden on a `templates/` folder.

Read more on [[Customizing Templates]]

However, if you want to start a new theme to fully customize you can use the `--start-theme` argument.

```console
$ marmite myblog /tmp/mysite --start-theme 
Generated myblog/templates/*
Generated myblog/static/*
```

Now you can freely edit the static files and templates.

## Generate configuration

### Default

To generate default config file

```console
$ marmite myblog --generate-config
Config file generated: "myblog/marmite.yaml"
```

### Override Configuration

To override specific keys

```console
$ marmite myblog --generate-config \
  --name MySite \
  --tagline "My articles" \
  --pagination 20 \
  --colorscheme gruvbox \
  --url "https://myblog.com" \
  --image-provider picsum

Config file generated: "myblog/marmite.yaml"
```

## Automatic Image Download

Marmite can automatically download banner images for your posts using image providers.
Currently supported provider is `picsum` which generates beautiful placeholder images.

### Enable automatic image download

Use `--image-provider` to enable automatic image downloads:

```console
$ marmite myblog output/ --image-provider picsum
```

This will automatically download banner images for posts (not pages) when:
- No `banner_image` is specified in the post's frontmatter
- The banner image file doesn't already exist

Images are saved as `{slug}.banner.jpg` in the media folder and use the site name,
post slug, and tags to generate deterministic, unique images.

### Configuration file

You can also set this option in your `marmite.yaml`:

```yaml
image_provider: picsum
```

> [!NOTE]
> Images are only downloaded for posts (content with dates), not for static pages.
> If you don't like a generated image, you can delete it and rebuild, change the post's tags,
> or manually specify a `banner_image` in the frontmatter.

Read more in the [[Automatic Image Download]] guide.

## Markdown Source Publishing

Marmite can publish the original markdown source files alongside your HTML content,
allowing readers to access the raw markdown files directly.

### Enable markdown publishing

Use `--publish-md` to copy markdown files to the output directory:

```console
$ marmite myblog output/ --publish-md true
```

This will copy all `.md` files to the output directory and add "📄 View source" links
to each post that point to the local markdown files.

### Link to source repository

Use `--source-repository` to link to an external repository instead of local files:

```console
$ marmite myblog output/ --publish-md true --source-repository "https://github.com/user/repo/tree/main/content"
```

With this configuration, the "📄 View source" links will point to the files in your
repository instead of local copies.

### Configuration file

You can also set these options in your `marmite.yaml`:

```yaml
publish_md: true
source_repository: https://github.com/user/repo/tree/main/content
```

> [!NOTE]
> Source links only appear on posts (content with dates), not on static pages.
> When both options are enabled, repository links take precedence over local files.

## CLI Help


```console
$ marmite --help
Marmite is the easiest static site generator.

Usage: marmite [OPTIONS] <INPUT_FOLDER> [OUTPUT_FOLDER]

Arguments:
  <INPUT_FOLDER>   Input folder containing markdown files
  [OUTPUT_FOLDER]  Output folder to generate the site [default: `input_folder/site`]

Options:
  -v, --verbose...
          Verbosity level (0-4) [default: 0 warn] options: -v: info,-vv: debug,-vvv: trace,-vvvv: trace
          all
  -w, --watch
          Detect changes and rebuild the site automatically
      --serve
          Serve the site with a built-in HTTP server
      --bind <BIND>
          Address to bind the server [default: 0.0.0.0:8000]
  -c, --config <CONFIG>
          Path to custom configuration file [default: marmite.yaml]
      --init-templates
          Initialize templates in the project
      --start-theme
          Initialize a theme with templates and static assets
      --generate-config
          Generate the configuration file
      --init-site
          Init a new site with sample content and default configuration this will overwrite existing files
          usually you don't need to run this because Marmite can generate a site from any folder with
          markdown files
      --new <NEW>
          Create a new post with the given title and open in the default editor
  -e
          Edit the file in the default editor
  -p
          Set the new content as a page
  -t <TAGS>
          Set the tags for the new content tags are comma separated
      --name <NAME>
          Site name [default: "Home" or value from config file]
      --tagline <TAGLINE>
          Site tagline [default: empty or value from config file]
      --url <URL>
          Site url [default: empty or value from config file]
      --footer <FOOTER>
          Site footer [default: from '_footer.md' or config file]
      --language <LANGUAGE>
          Site main language 2 letter code [default: "en" or value from config file]
      --pagination <PAGINATION>
          Number of posts per page [default: 10 or value from config file]
      --enable-search <ENABLE_SEARCH>
          Enable search [default: false or from config file] [possible values: true, false]
      --enable-related-content <ENABLE_RELATED_CONTENT>
          Enable backlinks and related content for posts [default: true or from config file] [possible values: true, false]
      --show-next-prev-links <SHOW_NEXT_PREV_LINKS>
          Show next and previous links in posts [default: true or from config file] [possible values: true, false]
      --content-path <CONTENT_PATH>
          Path for content subfolder [default: "content" or value from config file] this is the folder
          where markdown files are stored inside `input_folder` no need to change this if your markdown
          files are in `input_folder` directly
      --templates-path <TEMPLATES_PATH>
          Path for templates subfolder [default: "templates" or value from config file]
      --static-path <STATIC_PATH>
          Path for static subfolder [default: "static" or value from config file]
      --media-path <MEDIA_PATH>
          Path for media subfolder [default: "media" or value from config file] this path is relative to
          the folder where your content files are
      --default-date-format <DEFAULT_DATE_FORMAT>
          Default date format [default: "%b %e, %Y" or from config file] see
          <https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html>
      --publish-md <PUBLISH_MD>
          Publish markdown source files alongside HTML [default: false or from config file] [possible values: true, false]
      --source-repository <SOURCE_REPOSITORY>
          Source repository URL to link to markdown files [default: None or from config file]
      --image-provider <IMAGE_PROVIDER>
          Image provider for automatic banner image download [default: None or from config file]
          Available providers: picsum
  -h, --help
          Print help
  -V, --version
          Print version
```
