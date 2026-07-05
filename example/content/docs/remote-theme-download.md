+++
date = 2025-07-23
title = "Remote Theme Download"
+++

# Remote Theme Download

Marmite now supports downloading and installing themes directly from remote repositories! This feature makes it easy to share and reuse themes across projects.

## Using Remote Themes

Learn more about themes on [[Introducing themes in marmite]]

### Download from Repository

You can download a theme directly from GitHub, GitLab, or Codeberg:

```bash
marmite your_site --set-theme https://github.com/username/themename
marmite your_site --set-theme https://gitlab.com/username/themename
marmite your_site --set-theme https://codeberg.org/username/themename
```

### Download from Direct URL

If you have a direct link to a theme zip file:

```bash
marmite your_site --set-theme https://example.com/themes/mytheme.zip
```

### Set Local Theme

You can also set a theme that already exists in your project folder:

```bash
marmite your_site --set-theme mytheme
```

## How It Works

When you use `--set-theme` with a remote URL:

1. **Downloads the theme** - The theme is downloaded as a zip file from the repository
2. **Extracts the files** - The zip is extracted to a temporary location
3. **Validates theme.json** - Checks that the theme includes a valid `theme.json` file
4. **Installs the theme** - Moves the theme to your project folder
5. **Updates configuration** - Automatically updates or creates `marmite.yaml` with the new theme

## Theme Structure

A valid theme must include:

- `theme.json` - Theme metadata file
- `templates/` - Template files for your site
- `static/` - CSS, JS, and other static assets

### Example theme.json

```json
{
  "name": "My Theme",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "A beautiful theme for Marmite",
  "features": [
    "Responsive design",
    "Dark mode support",
    "SEO optimized"
  ],
  "tags": ["minimal", "clean", "responsive"]
}
```

## Creating a Theme Repository

To share your theme:

1. Create a new repository on GitHub, GitLab, or Codeberg
2. Add your theme files including `theme.json`
3. Push to the main branch
4. Share the repository URL

Users can then install your theme with:

```bash
marmite their_site --set-theme https://github.com/yourusername/yourtheme
```

## Example Theme

Try the example theme to see how it works:

```bash
marmite my_blog --set-theme https://github.com/rochacbruno/mytheme
```

This will download and install the example theme, showing you the theme information and next steps.

## Error Handling

The command will fail if:

- The URL is invalid or unreachable
- The theme doesn't include a `theme.json` file
- A theme with the same name already exists locally
- The repository host is not supported

In case of errors, any partially downloaded files are automatically cleaned up.