---
title: "Python Tutorial - Part 3: Collections and Data Structures"
date: 2024-11-12
series: python-tutorial
stream: tutorial
tags: ["python", "tutorial", "programming", "collections", "data-structures"]
authors: ["marmite"]
description: "Third part of our Python tutorial series covering lists, tuples, and dictionaries"
---

# Python Tutorial - Part 3: Collections and Data Structures

In this third installment of our Python tutorial series, we'll explore Python's built-in collection types that allow you to store and organize multiple pieces of data.

## Lists

Lists are ordered collections of items that can be changed (mutable).

```python
fruits = ["apple", "banana", "orange"]
numbers = [1, 2, 3, 4, 5]
mixed_list = ["hello", 42, 3.14, True]

# Accessing elements
print(fruits[0])  # "apple"
print(fruits[-1]) # "orange" (last element)

# Adding elements
fruits.append("grape")
```

## Tuples

Tuples are ordered collections that cannot be changed (immutable).

```python
coordinates = (10, 20)
colors = ("red", "green", "blue")

# Accessing elements (same as lists)
print(coordinates[0])  # 10

# Tuples are immutable - this would cause an error:
# coordinates[0] = 15  # TypeError!
```

## Dictionaries

Dictionaries store data in key-value pairs.

```python
person = {
    "name": "Alice",
    "age": 25,
    "city": "New York"
}

# Accessing values
print(person["name"])     # "Alice"
print(person.get("age"))  # 25

# Adding new key-value pairs
person["job"] = "Developer"
```

## Working with Collections

### Iterating through collections

```python
# Lists and tuples
for fruit in fruits:
    print(fruit)

# Dictionaries
for key, value in person.items():
    print(f"{key}: {value}")
```

### Length and membership

```python
print(len(fruits))           # Number of items
print("apple" in fruits)     # Check if item exists
print("name" in person)      # Check if key exists
```

## What's Next?

In Part 4, we'll learn about control flow: if statements, loops, and functions. These are the building blocks that make your Python programs dynamic and interactive!
