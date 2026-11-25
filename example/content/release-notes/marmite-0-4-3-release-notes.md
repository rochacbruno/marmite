---
date: 2026-09-29
title: Marmite 0.4.3 Release Notes
slug: marmite-0-4-3-release-notes
stream: draft
tags: [release-notes, marmite]
---

## Bug Fixes

### Development server binding

`--serve` now defaults to `127.0.0.1:8000`. If the requested port is unavailable,
the fallback uses an OS-assigned port while preserving the requested interface,
instead of binding to all interfaces. Use `--bind 0.0.0.0:8000` explicitly for
access from other machines on your network.

The CLI help and server startup message now explicitly identify the built-in
server as development-only, not for production deployments.

## Changes

### Markdown engine upgraded to comrak 1.0

Marmite now uses comrak 1.0 for Markdown parsing. Rendered output is unchanged
for all existing content (the full test suite and example site build identically).
The `parser_options.extension.tagfilter` option is now deprecated and has no
effect: the tagfilter extension was removed upstream in comrak 1.0. Setting it
to `true` logs a warning; remove it from your `marmite.yaml`.
