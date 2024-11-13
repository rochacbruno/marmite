---
title: Markdown Formatting Options
date: 2024-10-17 12:00:01
slug: markdown-format
description: The content here accepts any valid CommonMark or Github Flavoured markdown and some GFM extensions.
tags: docs, markdown, Common Mark, GFM
extra:
  math: true
  mermaid: true
  mermaid_theme: neutral
---

# Writing content in Markdown

The content here accepts any valid `CommonMark` or **Github** _Flavoured_ markdown
and some `GFM` extensions.

Simple paragraph and usual formatting like **bold**, __underline__, *italic*
and also all sorts of formatting elements.

### Strike-through

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

### Auto-link

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

### Wikilinks 

Wikilinks allows to link using `[[name|url]]` syntax.

[[Read the Tutorial|getting-started]] and [[Read the Tutorial|getting-started.md]] and [[Read the Tutorial|getting-started.html]]  

[[Pudim|https://pudim.com.br]]

```markdown
[[Read the Tutorial|getting-started]] and [[Read the Tutorial|getting-started.md]] and [[Read the Tutorial|getting-started.html]]

[[Pudim|https://pudim.com.br]]
```

### Obsidian Links

Obsidian links are made using `[[page-slug]]` or `[[page-slug.md]]`

Example:

[[about]] and [[about.md]] and [[about.html]] should point both to the about page.

[[https://pudim.com.br]]

```markdown
[[about]] and [[about.md]] and [[about.html]] should point both to the about page.

[[https://pudim.com.br]]
```


### Back-links

Every time you link to another page or post
using the backreference like `{slug}`, `{slug}.md` or `{slug}.html`
**marmite** will track the backlinking and show
a list of pages that links to each other.

Examples:

[[hello]]
[Hello1](hello.html)
[Hello2](hello.md)
[[Hello3|hello]]
[[Hello4|hello.md]]
<a href="hello.html">Hello5</a>

```markdown
[[hello]]
[Hello1](hello.html)
[Hello2](hello.md)
[[Hello3|hello]]
[[Hello4|hello.md]]
<a href="hello.html">Hello5</a>
```

In any case the `hello.html` page will have a this page
on its list of back-links:

<figure>
  <figcaption>Backlinks</figcaption>
  <img src="media/screenshots/backlink.png" width="500">
</figure>



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

## Emoji

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

### Diagrams

> Depends on `extra: {"mermaid": true}` defined on frontmatter, then **MermaidJS** is loaded.
>  
> `mermaid_theme` is also configurable with values `forest,neutral*,dark,forest,base,default`

```mermaid
sequenceDiagram
    Alice ->> Bob: Hello Bob, how are you?
    Bob-->>John: How about you John?
    Bob--x Alice: I am good thanks!
    Bob-x John: I am good thanks!
    Note right of John: Bob thinks a long<br/>long time, so long<br/>that the text does<br/>not fit on a row.

    Bob-->Alice: Checking with John...
    Alice->John: Yes... John, how are you?

```

```mermaid
graph LR
    A[Square Rect] -- Link text --> B((Circle))
    A --> C(Round Rect)
    B --> D{Rhombus}
    C --> D

```

```mermaid
xychart-beta
    title "Sales Revenue"
    x-axis [jan, feb, mar, apr, may, jun, jul, aug, sep, oct, nov, dec]
    y-axis "Revenue (in $)" 4000 --> 11000
    bar [5000, 6000, 7500, 8200, 9500, 10500, 11000, 10200, 9200, 8500, 7000, 6000]
    line [5000, 6000, 7500, 8200, 9500, 10500, 11000, 10200, 9200, 8500, 7000, 6000]

```

```mermaid
pie title Pets adopted by volunteers
    "Dogs" : 386
    "Cats" : 85
    "Rats" : 15

```

```mermaid
timeline
    title History of marmite
    2024-10-13 : Created
    2024-10-14 : First Release
               : First Contribution
    2024-10-20 : Big refactor
    2024-10-22 : Diagram support
```

<details>
<summary> Click to see the raw mermaid </summary>

````markdown
```mermaid
sequenceDiagram
    Alice ->> Bob: Hello Bob, how are you?
    Bob-->>John: How about you John?
    Bob--x Alice: I am good thanks!
    Bob-x John: I am good thanks!
    Note right of John: Bob thinks a long<br/>long time, so long<br/>that the text does<br/>not fit on a row.

    Bob-->Alice: Checking with John...
    Alice->John: Yes... John, how are you?

```

```mermaid
graph LR
    A[Square Rect] -- Link text --> B((Circle))
    A --> C(Round Rect)
    B --> D{Rhombus}
    C --> D

```

```mermaid
xychart-beta
    title "Sales Revenue"
    x-axis [jan, feb, mar, apr, may, jun, jul, aug, sep, oct, nov, dec]
    y-axis "Revenue (in $)" 4000 --> 11000
    bar [5000, 6000, 7500, 8200, 9500, 10500, 11000, 10200, 9200, 8500, 7000, 6000]
    line [5000, 6000, 7500, 8200, 9500, 10500, 11000, 10200, 9200, 8500, 7000, 6000]

```

```mermaid
pie title Pets adopted by volunteers
    "Dogs" : 386
    "Cats" : 85
    "Rats" : 15

```

```mermaid
timeline
    title History of marmite
    2024-10-13 : Created
    2024-10-14 : First Release
               : First Contribution
    2024-10-20 : Big refactor
    2024-10-22 : Diagram support
```
````

</details>

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
![Photo](media/marmite.jpg)

Same but containing a tooltip if you hover the mouse on  
![Photo](media/marmite.jpg "A jar of Marmite")
```markdown
Photo  
![Photo](media/marmite.jpg)

Same but containing a tooltip if you hover the mouse on  
![Photo](media/marmite.jpg "A jar of Marmite")
```

### Embed

Just use raw HTML for now, in future we may have a shorcode.

<iframe width="260" height="160" src="https://www.youtube.com/embed/MjrBTcnnK6c?si=PmQWsGiTh5XguSpb" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

```html
<iframe width="260" height="160" src="https://www.youtube.com/embed/MjrBTcnnK6c?si=PmQWsGiTh5XguSpb" title="YouTube video player"></iframe>
```

### Pico CSS components

<small>The default embedded template uses [picocss](https://picocss.com) so it is possible to write raw HTML like this:</small>

#### FAQ

<details>
<summary>Why is it named Marmite?</summary>

The creator of this project was looking for some cool name
to use for a **mark**down related project.
Then while having bread with Marmite spread for breakfast
it looked like a good idea!

</details>

<hr />

<details>
<summary>Why Rust?</summary>

**Why not?**

</details>

<hr />


```html
<details>
<summary>Why is it named Marmite?</summary>

The ...

</details>

<hr />

<details>
<summary>Why Rust?</summary>

**Why not?**

</details>

<hr />

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
  mermaid: true
---
```

Bye!


[^1]: My reference.
[1]: <https://en.wikipedia.org/wiki/Hobbit#Lifestyle> "Hobbit lifestyles"
