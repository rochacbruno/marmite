# Clean Theme Documentation

A minimal, clean theme for Marmite static site generator.

## Features

- **Clean Design**: Minimal, distraction-free layout
- **Responsive**: Works great on desktop, tablet, and mobile
- **Dark Mode**: Automatic dark mode based on system preference
- **Fast**: Lightweight CSS and JavaScript
- **Accessible**: Semantic HTML and keyboard navigation
- **SEO Ready**: Proper meta tags and structured data
- **Search**: Built-in search functionality (when enabled)

## Customization

### Colors

You can easily customize the theme colors by editing the CSS custom properties in `static/style.css`:

```css
:root {
  --primary-color: #2c5aa0;        /* Main brand color */
  --secondary-color: #f8f9fa;      /* Background accents */
  --text-color: #333;              /* Main text color */
  --link-color: #2c5aa0;           /* Link color */
  --background-color: #fff;        /* Page background */
}
```

### Layout

The theme uses a flexible layout system. You can customize:

- **Header**: Edit `templates/base.html` or create `content/_header.md`
- **Footer**: Edit `templates/base.html` or create `content/_footer.md`
- **Sidebar**: Create `content/_sidebar.md`
- **Hero Section**: Create `content/_hero.md`

### Typography

The theme includes the accessible Atkinson Hyperlegible font for better readability:

```css
@font-face {
  font-family: "Atkinson Hyperlegible";
  src: url("./Atkinson-Hyperlegible-Regular-102.woff");
}
```

It also uses system fonts as fallback for fast loading:

```css
font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
```

You can change the font settings in `static/style.css` or add additional web fonts.

### JavaScript

The theme includes minimal JavaScript for:

- Search overlay functionality
- Smooth scrolling for anchor links
- External link indicators
- Mobile menu support

## Configuration

### Required Marmite Settings

```yaml
# marmite.yaml
name: "Your Site Name"
tagline: "Your site description"
enable_search: true  # For search functionality
```

### Optional Settings

```yaml
# Enhanced footer
footer: |
  <p>&copy; 2025 Your Name. All rights reserved.</p>
  <p><a href="/about.html">About</a> | <a href="/contact.html">Contact</a></p>

# Custom menu
menu:
  - ["Home", "index.html"]
  - ["About", "about.html"]
  - ["Tags", "tags.html"]
  - ["Archive", "archive.html"]

# Fediverse verification (adds rel="me" link)
extra:
  fediverse_verification: "https://mastodon.social/@yourusername"
```

## Markdown Fragments

The theme supports these optional Markdown fragments:

- `_announce.md` - Site-wide announcement bar
- `_header.md` - Custom header content
- `_hero.md` - Hero section on homepage
- `_sidebar.md` - Sidebar content
- `_footer.md` - Custom footer content
- `_comments.md` - Comments system integration
- `_htmlhead.md` - Custom HTML in `<head>`
- `_htmltail.md` - Custom HTML before `</body>`

## Content Structure

### Posts

Posts should include front matter for best results:

```markdown
---
title: "Your Post Title"
date: 2025-07-21
authors: ["Your Name"]
tags: ["tag1", "tag2"]
description: "Brief description for SEO"
banner_image: "media/your-image.jpg"
---

Your post content here...
```

### Pages

Pages are similar but typically don't have dates:

```markdown
---
title: "About"
description: "About this website"
---

About page content...
```

## Assets Included

The theme comes with these pre-built assets:

- **Atkinson-Hyperlegible-Regular-102.woff**: Accessible web font for better readability
- **avatar-placeholder.png**: Default avatar image for authors
- **favicon.ico**: Site favicon
- **logo.png**: Default site logo placeholder
- **robots.txt**: SEO robots file
- **script.js**: Theme JavaScript functionality
- **search.js**: Search functionality (provided by Marmite)
- **style.css**: Complete theme stylesheet

## Development

To customize the theme further:

1. **CSS**: Edit `static/style.css`
2. **JavaScript**: Edit `static/script.js`  
3. **Templates**: Modify files in `templates/`
4. **Assets**: Replace or add images, fonts, etc. in `static/`
5. **Fonts**: The theme includes Atkinson Hyperlegible font, or you can add your own

## Browser Support

The theme supports all modern browsers:

- Chrome 60+
- Firefox 60+
- Safari 12+
- Edge 79+

## Performance

The theme is optimized for performance:

- **CSS**: Single file with CSS custom properties for easy theming
- **JavaScript**: Minimal, focused functionality
- **Web font**: Single Atkinson Hyperlegible font file (~40KB) for accessibility
- **System fonts**: Used as fallback for fast loading
- **No external dependencies** (except for search.js from Marmite)
- **Embedded assets**: All theme files are embedded in Marmite binary

## Accessibility

The theme follows accessibility best practices:

- Semantic HTML structure
- Keyboard navigation support
- High contrast colors
- Screen reader friendly
- Focus indicators

## Theme Configuration (theme.json)

The theme includes a `theme.json` file that describes the theme and its capabilities:

```json
{
  "name": "THEME_NAME",
  "version": "0.1.0",
  "author": "AUTHOR_NAME",
  "description": "THEME_DESCRIPTION",
  "marmite_version": ">=0.2.6",
  "extra_config_allowed": {
    "fediverse_verification": {
      "enabled": true,
      "required": false,
      "description": "Enable Fediverse Username Verification",
      "type": "string",
      "example": "https://mastodon.social/@username"
    }
  }
}
```

This configuration allows the theme to:
- Define supported extra configuration options
- Specify minimum Marmite version requirements
- Document theme metadata and features

## Embedded Theme Template

As of Marmite 0.2.6+, this theme template is **embedded directly in the Marmite binary**. This means:

- **Always available**: No need to download or locate template files
- **Version consistency**: Template matches your Marmite version exactly
- **Fast theme creation**: `--start-theme` extracts embedded files instantly
- **No external dependencies**: Everything needed is included

## License

This theme is released under the MIT License, same as Marmite.