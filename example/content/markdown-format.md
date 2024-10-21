---
date: 2024-01-01 12:00:01
slug: markdown-format
title: Markdown Formatting Options
tags: markdown, python, rust, Common Mark
extra:
  math: true
---

# This is the post content

The content here accepts any valid `CommonMark` or **Github** _Flavoured_ markdown
and some `GFM` extensions.

Simple paragraph and usual formatting like **bold**, __underline__, *italic*
and also all sorts of formatting elements.

### Strikethrough

The following is ~~no more~~.
```markdown
The following is ~~no more~~.
```

### Table

| Syntax      | Description |
| ----------- | ----------- |
| Header      | Title |
| List        | Here's a list! <ul><li>Item one.</li><li>Item two.</li></ul> |

```markdown
| Syntax      | Description |
| ----------- | ----------- |
| Header      | Title |
| List        | Here's a list! <ul><li>Item one.</li><li>Item two.</li></ul> |
```

### Autolink

https://github.com/rochacbruno/marmite  
[A link with a tooltip](https://pudim.com.br "A picture of a pudim")  
<https://www.markdownguide.org>  
<fake@example.com>

```markdown
https://github.com/rochacbruno/marmite  
[A link with a tooltip](https://pudim.com.br "A picture of a pudim")  
<https://www.markdownguide.org>  
<fake@example.com>
```

### Task

- [x] Task 1
- [ ] Task 2

```markdown
- [x] Task 1
- [ ] Task 2
```

## Superscript

> All **raw** html is allowed

80<sup>2</sup>

```html
80<sup>2</sup>
```

## Footnotes

Here is a simple footnote[^1]. With some additional text after it.  

A reference[1] can also be hidden from footnotes.

```markdown
Here is a simple footnote[^1]. With some additional text after it.  

A reference[1] can also be hidden from footnotes.
```
And on the end of the file:
```markdown
[^1]: My reference.
[1]: <https://en.wikipedia.org/wiki/Hobbit#Lifestyle> "Hobbit lifestyles"
```


## Description lists

First term

: Details for the **first term**

Second term

: Details for the **second term**

    More details in second paragraph.

```markdown
First term

: Details for the **first term**

Second term

: Details for the **second term**

    More details in second paragraph.
```

## Block quote

>Not a quote
> quote
> > > Nested quote

```markdown
>Not a quote
> quote
> > > Nested quote
```

Multiline quote

>>>
"Marmite is the easiest SSG" created by
Bruno Rocha with the contribution of various people.
>>>

```markdown
>>>
"Marmite is the easiest SSG" created by
Bruno Rocha with the contribution of various people.
>>>
```

Multi paragraph quote 

> Dorothy followed her through many of the beautiful rooms in her castle.
> 
> The Witch bade her clean the pots and kettles and sweep the floor and keep the fire fed with 

```markdown
> Dorothy followed her through many of the beautiful rooms in her castle.
> 
> The Witch bade her clean the pots and kettles and sweep the floor and keep the fire fed with 
```

Rich quotes

> #### The quarterly results look great!
> 
> - Revenue was off the chart.
> - Profits were higher than ever.
> 
>  *Everything* is going according to **plan**.

```markdown
> #### The quarterly results look great!
> 
> - Revenue was off the chart.
> - Profits were higher than ever.
> 
>  *Everything* is going according to **plan**.
```

## Emojis

:smile: - :crab: - :snake:

```markdown
:smile: - :crab: - :snake:
```


## Math

> Depends on `extra: {"math": true}` defined on frontmatter, then **MathJax** is loaded.

When $a \ne 0$, there are two solutions to \\(ax^2 + bx + c = 0\\) and they are
$$x = {-b \pm \sqrt{b^2-4ac} \over 2a}.$$

Inline math $1 + 2$ and display math $$x + y$$

$$
x^2
$$

```html
When $a \ne 0$, there are two solutions to \\(ax^2 + bx + c = 0\\) and they are
$$x = {-b \pm \sqrt{b^2-4ac} \over 2a}.$$

Inline math $1 + 2$ and display math $$x + y$$

$$
x^2
$$
```

## Underline

__dunder__

```markdown
__dunder__
```

## Spoiler 

This is ||secret||
```markdown
This is ||secret||
```

## Code

```python
import antigravity

def main():
    print("Python is a great language")
```

```rust
fn main() {
    println!("Marmite is made with Rust!");
}
```
````markdown
```python
import antigravity

def main():
    print("Python is a great language")
```

```rust
fn main() {
    println!("Marmite is made with Rust!");
}
```
````

### lists

- lists
  - sub item
- images
  * other
- tables
- Formatting
```markdown
- lists
  - sub item
- images
  * other
- tables
- Formatting
```

Numbered

1. First item
1. Second item
    - Indented unordered item
    - Indented unordered item
1. Third item
    1. Indented ordered item
    1. Indented ordered item
1. Fourth item 
```markdown
1. First item
1. Second item
    - Indented unordered item
    - Indented unordered item
1. Third item
    1. Indented ordered item
    1. Indented ordered item
1. Fourth item 
```

Starting lists with numbers requires a `number\`

- 1983\. A great year!
- I think 1984 was second best. 
```markdown
- 1983\. A great year!
- I think 1984 was second best. 
```

### Images

Photo  
![Photo](./media/marmite.jpg)

Same but containing a tooltip if you hover the mouse on  
![Photo](./media/marmite.jpg "A jar of Marmite")
```markdown
Photo  
![Photo](./media/marmite.jpg)

Same but containing a tooltip if you hover the mouse on  
![Photo](./media/marmite.jpg "A jar of Marmite")
```

### Symbols


    Copyright (©) — &copy;
    Registered trademark (®) — &reg;
    Trademark (™) — &trade;
    Euro (€) — &euro;
    Left arrow (←) — &larr;
    Up arrow (↑) — &uarr;
    Right arrow (→) — &rarr;
    Down arrow (↓) — &darr;
    Degree (°) — &#176;
    Pi (π) — &#960;

This is a &copy;left material.
```markdown
This is a &copy;left material.
```

---

This post specifies `FrontMatter` on its header
```yaml
---
date: 2024-01-01 12:00:01
slug: markdown-format
title: Markdown Formatting Options
tags: markdown, python, rust, Common Mark
extra:
  math: true
---
```

Bye!


[^1]: My reference.
[1]: <https://en.wikipedia.org/wiki/Hobbit#Lifestyle> "Hobbit lifestyles"
