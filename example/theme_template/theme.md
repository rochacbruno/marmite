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

The theme uses system fonts for fast loading:

```css
font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
```

You can change this in `static/style.css` to use custom fonts.

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

## Development

To customize the theme further:

1. **CSS**: Edit `static/style.css`
2. **JavaScript**: Edit `static/script.js`  
3. **Templates**: Modify files in `templates/`
4. **Assets**: Add images, fonts, etc. to `static/`

## Browser Support

The theme supports all modern browsers:

- Chrome 60+
- Firefox 60+
- Safari 12+
- Edge 79+

## Performance

The theme is optimized for performance:

- **CSS**: Single file, ~15KB minified
- **JavaScript**: Minimal, ~5KB
- **No external dependencies** (except for search.js from Marmite)
- **System fonts** for fast loading

## Accessibility

The theme follows accessibility best practices:

- Semantic HTML structure
- Keyboard navigation support
- High contrast colors
- Screen reader friendly
- Focus indicators

## License

This theme is released under the MIT License, same as Marmite.