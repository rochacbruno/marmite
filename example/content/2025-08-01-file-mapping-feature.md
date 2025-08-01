---
title: File Mapping Feature
tags: docs, configuration, features
description: Learn how to use file mapping to copy arbitrary files during site generation
authors: rochacbruno
---

# File Mapping Feature

> [!IMPORTANT]
> This is a **Beta** feature currently available only on the main branch. It has not been released in a stable version yet.

Marmite now supports file mapping, which allows you to copy arbitrary files from source locations to destination paths during site generation. This feature is useful for copying files that aren't handled by the standard static or media directory copying.

## Configuration

Add the `file_mapping` section to your `marmite.yaml`:

```yaml
file_mapping:
  - source: path/to/source.txt
    dest: destination/path.txt
```

## How It Works

The file mapping feature supports three types of sources:

### 1. Single Files

Copy a single file to a destination:

```yaml
file_mapping:
  - source: robots.txt
    dest: robots.txt
  - source: ai/llms.txt
    dest: llms.txt
```

### 2. Directories

Copy entire directories:

```yaml
file_mapping:
  - source: extra_assets
    dest: assets
```

### 3. Glob Patterns

Use glob patterns to copy multiple files matching a pattern:

```yaml
file_mapping:
  - source: assets/imgs/*.jpg
    dest: media/photos
  - source: docs/**/*.pdf
    dest: downloads/pdfs
```

## Source Path Resolution

- **Relative paths**: Resolved relative to the input directory
- **Absolute paths**: Used as-is
- **Glob patterns**: Matched relative to the input directory

## Destination Path Behavior

- If the destination has a file extension, it's treated as a file
- If the destination has no extension, it's treated as a directory
- Parent directories are automatically created if they don't exist

## Examples

### Example 1: Copy llms.txt for LLM compatibility

Following the [llmstxt.org](https://llmstxt.org/) standard:

```yaml
file_mapping:
  - source: extrastatic/llms.txt
    dest: llms.txt
```

### Example 2: Copy all images from a specific folder

```yaml
file_mapping:
  - source: photography/*.jpg
    dest: gallery/photos
```

### Example 3: Copy configuration files

```yaml
file_mapping:
  - source: configs/robots.txt
    dest: robots.txt
  - source: configs/.well-known
    dest: .well-known
```

## Use Cases

- Publishing `llms.txt` files for LLM compatibility
- Copying `.well-known` directory for domain verification
- Moving downloaded or generated files to specific locations
- Organizing assets from multiple source directories
- Publishing API documentation or specification files

## Order of Execution

File mappings are processed after:
1. Static directory copying
2. Extra static folders copying
3. Media directory copying

This ensures that file mappings can override files from these standard directories if needed.

---

The file mapping feature provides flexibility in organizing your site's file structure without being constrained by Marmite's standard directory conventions.