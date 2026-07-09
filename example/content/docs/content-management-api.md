---
date: 2026-07-09 12:00:00
tags: docs, features, api, server
description: Marmite exposes a local REST API during --serve mode for creating, editing, moving, cloning, and deleting content, plus managing site configuration.
---

# Content Management API

When you run marmite with `--serve`, the built-in server exposes a REST API under `/__marmite__/` for managing content and configuration programmatically. This API is used by the [[marmite-toolbar]] and can also be called directly with `curl` or any HTTP client.

> [!IMPORTANT]
> The API is only available on the local dev server (`--serve`). It is not part of the generated static site and is not exposed in production deployments.

## Endpoints

### Content

#### Create content

```
POST /__marmite__/content
```

Creates a new markdown file in the content directory.

**Request body (JSON):**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | yes | Content title |
| `tags` | string or array | no | Comma-separated string or array of tags |
| `page` | boolean | no | If true, creates a page (no date). Default: false |
| `lang` | string | no | ISO 639-1 language code |
| `translates` | string | no | Slug of the content this translates |
| `directory` | string | no | Subdirectory within the content folder |

**Example:**

```console
$ curl -X POST http://localhost:8000/__marmite__/content \
  -H 'Content-Type: application/json' \
  -d '{"title": "Getting Started with Rust", "tags": "rust, tutorial"}'
```

**Response (201):**

```json
{
  "file": "content/posts/getting-started-with-rust.md",
  "title": "Getting Started with Rust",
  "slug": "getting-started-with-rust",
  "is_page": false,
  "date": "2026-07-09 12:00:00",
  "tags": "rust, tutorial"
}
```

#### Update frontmatter

```
PATCH /__marmite__/content/{slug}
```

Updates frontmatter fields of an existing content file. The markdown body is preserved. Set a field to `null` to remove it.

**Example:**

```console
$ curl -X PATCH http://localhost:8000/__marmite__/content/my-post \
  -H 'Content-Type: application/json' \
  -d '{"tags": "rust, web", "stream": "tutorial", "pinned": true}'
```

**Response (200):**

```json
{
  "slug": "my-post",
  "file": "content/posts/my-post.md",
  "frontmatter": {
    "title": "My Post",
    "tags": "rust, web",
    "stream": "tutorial",
    "pinned": true
  }
}
```

#### Clone content

```
POST /__marmite__/content/{slug}/clone
```

Copies the entire file (frontmatter and markdown body) to a new file with a different title and slug. The `aliases` and `translates` fields are removed from the clone.

**Request body (JSON):**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | yes | Title for the cloned content |
| `slug` | string | no | Custom slug. If omitted, derived from the title |

**Example:**

```console
$ curl -X POST http://localhost:8000/__marmite__/content/my-post/clone \
  -H 'Content-Type: application/json' \
  -d '{"title": "My Post (Copy)"}'
```

**Response (201):**

```json
{
  "slug": "my-post-copy",
  "file": "content/posts/my-post-copy.md",
  "source": "my-post"
}
```

#### Move / Rename content

```
POST /__marmite__/content/{slug}/move
```

Moves or renames a content file. The filename must end with `.md`. Paths with `/` are relative to the content directory and intermediate directories are created automatically.

**Request body (JSON):**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `filename` | string | yes | New filename or path (e.g. `new-name.md` or `posts/new-name.md`) |

**Example:**

```console
$ curl -X POST http://localhost:8000/__marmite__/content/my-post/move \
  -H 'Content-Type: application/json' \
  -d '{"filename": "tutorials/my-post.md"}'
```

**Response (200):**

```json
{
  "slug": "my-post",
  "old_file": "content/posts/my-post.md",
  "new_file": "content/tutorials/my-post.md"
}
```

> The response `slug` reflects the actual slug the content will render as. If the file has an explicit `slug:` in frontmatter, that value is returned regardless of the new filename.

#### Delete content

```
DELETE /__marmite__/content/{slug}
```

Deletes the markdown file associated with the given slug.

**Example:**

```console
$ curl -X DELETE http://localhost:8000/__marmite__/content/my-post
```

**Response (200):**

```json
{
  "slug": "my-post",
  "file": "content/posts/my-post.md",
  "deleted": true
}
```

### Configuration

#### Create config

```
POST /__marmite__/config
```

Creates a new `marmite.yaml` with default values. Returns 409 if the file already exists.

#### Update config

```
PATCH /__marmite__/config
```

Merges fields into the existing `marmite.yaml`. Set a field to `null` to remove it from the config.

**Example:**

```console
$ curl -X PATCH http://localhost:8000/__marmite__/config \
  -H 'Content-Type: application/json' \
  -d '{"pagination": 20, "enable_search": true}'
```

**Response (200):**

```json
{
  "file": "marmite.yaml",
  "config": { "...merged config..." }
}
```

### Site Data

#### Get aggregated data

```
GET /__marmite__/data
```

Returns aggregated site data useful for building tools and editors. Includes lists of all tags, streams, series, authors, languages, content slugs, the full site config, and build stats.

**Response (200):**

```json
{
  "tags": ["docs", "rust", "tutorial"],
  "streams": ["guide", "news"],
  "series": ["python-tutorial"],
  "authors": ["alice"],
  "languages": ["en", "pt"],
  "iso_languages": ["aa", "ab", "...all ISO 639-1 codes..."],
  "slugs": ["about", "hello-world", "my-post"],
  "post_count": 25,
  "page_count": 3,
  "elapsed_time": 0.6,
  "marmite_version": "0.4.1",
  "config": { "...full marmite.yaml config..." }
}
```

### Dev Server Assets

These endpoints serve the [[marmite-toolbar]] assets and are injected automatically into HTML pages during `--serve` mode.

| Endpoint | Description |
|----------|-------------|
| `GET /__marmite__/toolbar.js` | Toolbar JavaScript |
| `GET /__marmite__/toolbar.css` | Toolbar stylesheet |
| `GET /__marmite__/livereload.js` | Live reload script (with `--watch`) |
| `WS /__marmite__/livereload` | Live reload WebSocket (with `--watch`) |

### Content Metadata

When running with `--serve`, marmite generates a metadata JSON file for each content page:

```
GET /{slug}.metadata.json
```

**Response:**

```json
{
  "frontmatter": {
    "title": "My Post",
    "slug": "my-post",
    "date": "2026-07-09 12:00:00",
    "tags": ["rust", "tutorial"],
    "...all frontmatter fields..."
  },
  "source_path": "posts/my-post.md",
  "last_updated": "2026-07-09T12:00:00+00:00"
}
```

## Error responses

All endpoints return JSON error responses:

```json
{"error": "description of the error"}
```

| Status | Meaning |
|--------|---------|
| 400 | Bad request (missing fields, invalid JSON) |
| 404 | Content or config not found |
| 405 | HTTP method not allowed |
| 409 | Conflict (config already exists) |
| 500 | Server error (file write failed) |
