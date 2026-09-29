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
