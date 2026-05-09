# Marmite Deployment Guide

Marmite generates a flat directory of static HTML, CSS, and JS files. The output can be deployed to any static hosting provider or web server.

## Build Command

```bash
# Basic build
marmite <input_folder> <output_folder>

# Build to default output (input_folder/site)
marmite <input_folder>

# Force full rebuild
marmite <input_folder> --force
```

The output folder contains everything needed - just upload it.

## GitHub Pages

The most common and simplest deployment option.

### Repository Setup

1. Create a GitHub repository with your marmite project:
```
.
  marmite.yaml
  content/
    2024-06-15-my-first-post.md
    about.md
```

2. Set `url` in `marmite.yaml`:
```yaml
url: https://YOURUSER.github.io/REPONAME
```

3. Go to repository Settings > Pages and set source to **GitHub Actions**.

### GitHub Actions Workflow

Create `.github/workflows/deploy.yaml`:

```yaml
name: Deploy to GitHub Pages

on:
  push:
    branches: [main]
  workflow_dispatch:

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
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install marmite
        run: curl -sS https://marmite.blog/install.sh | sh

      - name: Build site
        run: marmite . site

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

Push to main and your site will be published at `https://YOURUSER.github.io/REPONAME`.

### Template Repository

Use the template at https://github.com/rochacbruno/blog to get started quickly.

## GitLab Pages

### GitLab CI Configuration

Create `.gitlab-ci.yml`:

```yaml
image: rust:latest

pages:
  stage: deploy
  script:
    - curl -sS https://marmite.blog/install.sh | sh
    - marmite . public
  artifacts:
    paths:
      - public
  only:
    - main
```

Set `url` in `marmite.yaml`:
```yaml
url: https://YOURUSER.gitlab.io/REPONAME
```

## Netlify

### Configuration

Create `netlify.toml` in the repository root:

```toml
[build]
  command = "curl -sS https://marmite.blog/install.sh | sh && marmite . site"
  publish = "site"
```

### Setup Steps

1. Connect your repository to Netlify
2. Netlify detects `netlify.toml` and uses the build command
3. Set `url` in `marmite.yaml` to your Netlify domain

### Environment Variables

If using a custom domain:
```yaml
url: https://myblog.com
```

## Vercel

### Configuration

Create `vercel.json`:

```json
{
  "buildCommand": "curl -sS https://marmite.blog/install.sh | sh && marmite . site",
  "outputDirectory": "site"
}
```

### Setup Steps

1. Import your repository in the Vercel dashboard
2. Vercel detects `vercel.json` and uses the configuration
3. Set `url` in `marmite.yaml` to your Vercel domain

## Cloudflare Pages

### Setup Steps

1. Connect your repository in the Cloudflare Pages dashboard
2. Set the build configuration:
   - Build command: `curl -sS https://marmite.blog/install.sh | sh && marmite . site`
   - Build output directory: `site`
3. Set `url` in `marmite.yaml` to your Cloudflare Pages domain

## Docker

### Building with Docker

```bash
docker run --rm -v $(pwd):/app ghcr.io/rochacbruno/marmite /app /app/site
```

### Docker Compose for Development

```yaml
services:
  marmite:
    image: ghcr.io/rochacbruno/marmite
    volumes:
      - .:/app
    command: /app /app/site --serve --watch --bind 0.0.0.0:8000
    ports:
      - "8000:8000"
```

### Dockerfile for Production

```dockerfile
FROM ghcr.io/rochacbruno/marmite AS builder
COPY . /app
RUN marmite /app /output

FROM nginx:alpine
COPY --from=builder /output /usr/share/nginx/html
```

## Nginx

### Basic Configuration

```nginx
server {
    listen 80;
    server_name myblog.com;
    root /var/www/myblog;
    index index.html;

    location / {
        try_files $uri $uri/ =404;
    }

    error_page 404 /404.html;

    # Cache static assets
    location ~* \.(css|js|jpg|jpeg|png|gif|ico|svg|woff2)$ {
        expires 30d;
        add_header Cache-Control "public, immutable";
    }
}
```

### Deploy Script

```bash
#!/bin/bash
marmite /path/to/project /var/www/myblog --force
```

## Apache

### .htaccess

```apache
ErrorDocument 404 /404.html

<IfModule mod_rewrite.c>
  RewriteEngine On
  RewriteCond %{REQUEST_FILENAME} !-f
  RewriteCond %{REQUEST_FILENAME} !-d
  RewriteRule . /404.html [L]
</IfModule>

<IfModule mod_expires.c>
  ExpiresActive On
  ExpiresByType text/css "access plus 30 days"
  ExpiresByType application/javascript "access plus 30 days"
  ExpiresByType image/jpeg "access plus 30 days"
  ExpiresByType image/png "access plus 30 days"
</IfModule>
```

## Any Static Host

Marmite output works with any static file host. General steps:

1. Build the site: `marmite <input> <output>`
2. Upload the output directory contents to your host
3. Set `url` in `marmite.yaml` to match the deployment URL

### rsync to a Server

```bash
marmite . site --force
rsync -avz --delete site/ user@server:/var/www/myblog/
```

### S3 / Object Storage

```bash
marmite . site --force
aws s3 sync site/ s3://my-bucket/ --delete
```

## Common Deployment Patterns

### Base URL Handling

Always set `url` in `marmite.yaml` to match your deployment URL:

```yaml
# Root domain
url: https://myblog.com

# Subdirectory (GitHub Pages project site)
url: https://user.github.io/reponame
```

The `url_for()` template function uses this to generate correct links.

### Custom Domain with HTTPS

Most hosting providers handle HTTPS automatically. Set:
```yaml
url: https://myblog.com
https: true
```

### Build Optimization

For production builds:
```bash
# Full build with image optimization
marmite . site --force

# Faster dev builds (skip image resize)
marmite . site --skip-image-resize
```

### Sitemap Submission

Marmite generates `sitemap.xml` by default (`build_sitemap: true`). Submit it to search engines:
- Google Search Console: add `https://myblog.com/sitemap.xml`
- Bing Webmaster Tools: add the sitemap URL

### robots.txt

The default `static/robots.txt` allows all crawlers. Customize by placing your own `robots.txt` in the `static/` folder.

### Feed Discovery

Marmite generates RSS feeds. Add feed discovery to `_htmlhead.md`:
```html
<link rel="alternate" type="application/rss+xml" title="RSS Feed" href="/index.rss">
```

The default templates already include this.

## Verification Checklist

Before deploying:

- [ ] `url` is set correctly in `marmite.yaml`
- [ ] Build completes without errors: `marmite . site`
- [ ] Local preview looks correct: `marmite . --serve`
- [ ] `sitemap.xml` is generated
- [ ] RSS feed (`index.rss`) is valid
- [ ] 404 page works (if `_404.md` is provided)
- [ ] Images load correctly
- [ ] Search works (if `enable_search: true`)
- [ ] Links between posts resolve correctly
