---
title: Marmite 0.4.1 Release Notes
slug: marmite-0-4-1-release-notes
description: "Marmite 0.4.1 adds smart directory auto-detection for the --new CLI command in structured projects."
tags: [release-notes, marmite, features]
author: rochacbruno
stream: draft
---

## Improvements

### Smart directory auto-detection for `--new` command

The `--new` CLI command now automatically detects `posts/` and `pages/` subdirectories in structured projects and places content in the right location without requiring the `-d` flag.

**Before:** Users had to remember to pass `-d posts` or `-d pages` every time, even when the project already had those directories.

```console
$ marmite myblog --new "My Post" -d posts
$ marmite myblog --new "About" -p -d pages
```

**After:** The CLI detects the project structure and routes content automatically.

```console
$ marmite myblog --new "My Post"
# -> created in content/posts/

$ marmite myblog --new "About" -p
# -> created in content/pages/
```

The `-d` flag still works to override auto-detection or to place content in custom subfolders. Flat projects and content-folder projects without `posts/`/`pages/` subdirectories are completely unaffected.
