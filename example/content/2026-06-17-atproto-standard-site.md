---
tags: docs, atproto, standard.site
description: Complete guide to publishing your Marmite blog posts to the decentralized AT Protocol using the standard.site specification.
---

# AT Protocol standard.site

Marmite natively supports publishing your blog posts using the decentralized [AT Protocol](https://atproto.com) and the [standard.site](https://standard.site) lexicons. This enables your posts to be discovered, read, and interacted with (e.g., inside the [Bluesky](https://bsky.app)) across the decentralized web while keeping your Marmite site as the canonical source.

---

## Setup & Configuration

To enable the AT Protocol features, add the `atproto` configuration block to your `marmite.yaml`:

```yaml
name: "My Blog"
url: "https://myblog.com"

atproto:
  handle: "myhandle.bsky.social"
  publication_uri: "at://did:plc:.../site.standard.publication/..."
  publish_content: true
```


| Name | Required | Description |
|---|---|---|
| `handle` | Yes | Your AT Protocol handle (e.g. `yourname.bsky.social` or a custom domain handle) |
| `publication_uri` | Yes | The AT-URI of your publication. You should register this publication externally (e.g., using a client like [standard.horse](https://standard.horse), [std-pub](https://cuducos.tngl.io/std-pub), or the [`goat` CLI](https://github.com/bluesky-social/goat)) |
| `publish_content` | No | If `true`, Marmite will strip HTML tags from your compiled markdown and publish the post body text up to 10,000 characters as `textContent` to the AT Protocol record (defaults to `true`) |

---

## Authentication

Before publishing, you must authenticate Marmite with your <a href="https://atproto.com/guides/going-to-production#pds"><abbr title="Personal Data Server">PDS</abbr></a>.

First, ensure `atproto.handle` is configured in your `marmite.yaml`.

Next, create an _App Password_ for your account (e.g., in Bluesky go to _Settings, App Passwords_).

Export the password in your environment:

```bash
export ATPROTO_APP_PASSWORD="xxxx-xxxx-xxxx-xxxx"
```

Finally, run the authentication subcommand pointing to your site's directory:

```bash
marmite <site_folder> atproto auth
```

Marmite will:
*   Perform a decentralized DNS/HTTP lookup to resolve your handle to its Decentralized Identifier (DID).
*   Query `plc.directory` to resolve your DID to your actual <abbr title="Personal Data Server">PDS</abbr> endpoint.
*   Acquire an authentication session from your <abbr title="Personal Data Server">PDS</abbr> and save the credentials locally at `~/.config/marmite/credentials.json`.

---

## Local Site Build & Verification

When you compile your site using `marmite build` or run the dev server with `marmite serve`, Marmite automatically takes care of domain-level and page-level standard.site verification:

### Automatic `.well-known` Generation
Marmite generates a verification file at:
`/.well-known/site.standard.publication` in your output directory.
This file contains your `publication_uri`. Indexers and clients check this file on your domain to verify that you indeed own the AT Protocol publication record.

### Automatic Header Injection

For the **homepage and pages**, the default templates automatically inject the publication discovery link:

```html
<link rel="site.standard.publication" href="at://did:plc:.../site.standard.publication/...">
```

For **posts**, Marmite reads the state mapping from your published records and automatically injects the document link into the header of each post:

```html
<link rel="site.standard.document" href="at://did:plc:.../site.standard.document/...">
```

---

## Publishing Posts (`publish`)

To publish your posts to your <abbr title="Personal Data Server">PDS</abbr>, run:

```bash
marmite <site_folder> atproto publish
```

Also, check `--help` for more options.

That command:

1. Gathers all valid markdown posts (excluding drafts).
2. Computes a content hash of each post to detect modifications.
3. Authenticates with your resolved <abbr title="Personal Data Server">PDS</abbr>.
4. Performs `createRecord` (for new posts) or `putRecord` (for modified posts) under the `site.standard.document` collection in your repository.
5. Saves the mapping of post slugs to AT-URIs inside `.marmite-atproto-state.json`.

Subsequent site `marmite build` read this local state file and automatically inject the document AT-URIs into the compiled HTML heads.

---

## Customizing Templates (Advanced)

If you are using a custom theme and want to manually inject standard.site tags, add the following to your HTML heads:

### Base layout (`base.html` or similar):
```html
{% if site.atproto and site.atproto.publication_uri %}
<link rel="site.standard.publication" href="{{ site.atproto.publication_uri }}">
{% endif %}
```

### Content layout (`content.html` or similar):
```html
{% if content.at_uri %}
<link rel="site.standard.document" href="{{ content.at_uri }}">
{% endif %}
```
