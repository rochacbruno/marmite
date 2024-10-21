---
date: 2024-01-01 12:00:01
slug: blog-post
title: Markdown Powered Blog Post (with code blocks)
tags: markdown, python, rust, Common Mark
---

# This is the post content

The content here accepts any valid `CommonMark` or **Github** _Flavoured_ markdown
and some `micromark` extensions.

Simple paragraph and usual formatting like **bold**,__underline__,*italic*
and also all sorts of formatting elements.

### lists

- lists
  - sub item
- images
  * other
- tables
- Formatting

Numbered

1. First item
1. Second item
    - Indented unordered item
    - Indented unordered item
1. Third item
    1. Indented ordered item
    1. Indented ordered item
1. Fourth item 

Starting lists with numbers requires a `number\`

- 1983\. A great year!
- I think 1984 was second best. 

### Images


Photo  
![Photo](./media/marmite.jpg)

Same but containing a tooltip if you hover the mouse on  
![Photo](./media/marmite.jpg "A jar of Marmite")


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

### Strike

The folling is now ~~not valid~~ anymore.

### Table


| String  | g        | 颪                         | 🦀                                   |
|---------|----------|----------------------------|-------------------------------------|
| Unicode | 103      | 39082                      | 129408                              |
| Binary  | 01100111 | 11101001 10100010 10101010 | 11110000 10011111 10100110 10000000 |

Lists Within Table Cells  

You can add a list within a table cell by using HTML tags.  

| Syntax      | Description |
| ----------- | ----------- |
| Header      | Title |
| List        | Here's a list! <ul><li>Item one.</li><li>Item two.</li></ul> |


### Autolink

https://github.com/rochacbruno/marmite  
[A link with a tooltip](https://pudim.com.br "A picture of a pudim")  
<https://www.markdownguide.org>  
<fake@example.com>

### Task

- [x] Task 1
- [ ] Task 2

## Superscript

80^2^

## Footnotes

Here is a simple footnote[^1]. With some additional text after it.  

A reference[1] can also be hidden from footnotes.

## Description lists

First term

: Details for the **first term**

Second term

: Details for the **second term**

    More details in second paragraph.

## Block quote

>No Quote
> quote
> > > Nested quote

Multiline quote

>>>
"Marmite is the easiest SSG" created by
Bruno Rocha with the contribution of various people.
>>>

Multi paragraph quote 

> Dorothy followed her through many of the beautiful rooms in her castle.
> 
> The Witch bade her clean the pots and kettles and sweep the floor and keep the fire fed with 

Rich quotes

> #### The quarterly results look great!
> 
> - Revenue was off the chart.
> - Profits were higher than ever.
> 
>  *Everything* is going according to **plan**.


## Emojis

:smile: - :crab: - :snake:
```
:smile: - :crab: - :snake:
```


## Math

Inline math $1 + 2$ and display math $$x + y$$

$$
x^2
$$

Inline math $`1 + 2`$

```math
x^2
```

## Underline

__dunder__

## Spoiler 

This is ||secret||

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

---

This post specifies `FrontMatter` on its header, so `title`, `slug`, `date` and tags are taken from there.

Bye!


[^1]: My reference.
[1]: <https://en.wikipedia.org/wiki/Hobbit#Lifestyle> "Hobbit lifestyles"
