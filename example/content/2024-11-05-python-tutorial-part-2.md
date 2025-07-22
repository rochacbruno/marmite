---
title: "Python Tutorial - Part 2: Data Types and Variables"
date: 2024-11-05
series: python-tutorial
stream: tutorial
tags: ["python", "tutorial", "programming", "data-types"]
authors: ["marmite"]
description: "Second part of our Python tutorial series focusing on data types and variables"
---

# Python Tutorial - Part 2: Data Types and Variables

Welcome back to our Python tutorial series! In this second part, we'll explore Python's built-in data types and how to work with variables.

## Python Variables

Variables in Python are used to store data values. Unlike many other programming languages, Python has no command for declaring a variable.

```python
name = "Alice"
age = 25
height = 5.6
```

## Basic Data Types

Python has several built-in data types:

### Numbers
- **int**: Integer numbers (e.g., 42, -17)
- **float**: Decimal numbers (e.g., 3.14, -0.5)

```python
integer_number = 42
floating_number = 3.14159
```

### Strings
Text data is represented as strings in Python.

```python
greeting = "Hello, World!"
multiline_text = """This is a
multiline string"""
```

### Booleans
Boolean values represent True or False.

```python
is_python_fun = True
is_difficult = False
```

## Type Checking

You can check the type of any variable using the `type()` function:

```python
print(type(42))        # <class 'int'>
print(type(3.14))      # <class 'float'>
print(type("Hello"))   # <class 'str'>
```

## What's Next?

In Part 3, we'll dive into Python collections: lists, tuples, and dictionaries. These are fundamental data structures you'll use constantly in Python programming.
