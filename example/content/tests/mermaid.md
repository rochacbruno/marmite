---
title: Mermaid Diagram Tests
date: 2026-01-01
stream: draft
mermaid_config:
  theme: default
  flowchart:
    nodeSpacing: 60
    rankSpacing: 50
  sequence:
    mirrorActors: true
---

This page tests various mermaid diagram types with native rendering.

## Flowchart

```mermaid
flowchart TD
    A[Start] --> B{Is it raining?}
    B -- Yes --> C[Take umbrella]
    B -- No --> D[Wear sunglasses]
    C --> E[Go outside]
    D --> E
    E --> F[Enjoy the day]
```

Source:

````
```mermaid
flowchart TD
    A[Start] --> B{Is it raining?}
    B -- Yes --> C[Take umbrella]
    B -- No --> D[Wear sunglasses]
    C --> E[Go outside]
    D --> E
    E --> F[Enjoy the day]
```
````

---

## Sequence Diagram

```mermaid
sequenceDiagram
    participant Browser
    participant Server
    participant Database

    Browser->>Server: GET /api/posts
    Server->>Database: SELECT * FROM posts
    Database-->>Server: ResultSet
    Server-->>Browser: 200 OK (JSON)
    Browser->>Server: POST /api/posts
    Server->>Database: INSERT INTO posts
    Database-->>Server: OK
    Server-->>Browser: 201 Created
```

Source:

````
```mermaid
sequenceDiagram
    participant Browser
    participant Server
    participant Database

    Browser->>Server: GET /api/posts
    Server->>Database: SELECT * FROM posts
    Database-->>Server: ResultSet
    Server-->>Browser: 200 OK (JSON)
    Browser->>Server: POST /api/posts
    Server->>Database: INSERT INTO posts
    Database-->>Server: OK
    Server-->>Browser: 201 Created
```
````

---

## Class Diagram

```mermaid
classDiagram
    class Animal {
        +String name
        +int age
        +makeSound()
    }
    class Dog {
        +String breed
        +fetch()
        +makeSound()
    }
    class Cat {
        +bool indoor
        +purr()
        +makeSound()
    }
    class Shelter {
        +String location
        +List~Animal~ animals
        +adopt(Animal)
        +intake(Animal)
    }
    Animal <|-- Dog
    Animal <|-- Cat
    Shelter "1" --> "*" Animal : houses
```

Source:

````
```mermaid
classDiagram
    class Animal {
        +String name
        +int age
        +makeSound()
    }
    class Dog {
        +String breed
        +fetch()
        +makeSound()
    }
    class Cat {
        +bool indoor
        +purr()
        +makeSound()
    }
    class Shelter {
        +String location
        +List~Animal~ animals
        +adopt(Animal)
        +intake(Animal)
    }
    Animal <|-- Dog
    Animal <|-- Cat
    Shelter "1" --> "*" Animal : houses
```
````

---

## State Diagram

```mermaid
stateDiagram-v2
    [*] --> Draft
    Draft --> Review : Submit
    Review --> Draft : Request changes
    Review --> Approved : Approve
    Approved --> Published : Publish
    Published --> Archived : Archive
    Archived --> Draft : Restore
    Published --> [*]
```

Source:

````
```mermaid
stateDiagram-v2
    [*] --> Draft
    Draft --> Review : Submit
    Review --> Draft : Request changes
    Review --> Approved : Approve
    Approved --> Published : Publish
    Published --> Archived : Archive
    Archived --> Draft : Restore
    Published --> [*]
```
````

---

## Entity Relationship Diagram

```mermaid
erDiagram
    USER ||--o{ POST : writes
    USER ||--o{ COMMENT : writes
    POST ||--o{ COMMENT : has
    POST ||--o{ TAG : "tagged with"
    POST {
        int id PK
        string title
        text body
        date published_at
    }
    USER {
        int id PK
        string username
        string email
    }
    COMMENT {
        int id PK
        text body
        date created_at
    }
    TAG {
        int id PK
        string name
    }
```

Source:

````
```mermaid
erDiagram
    USER ||--o{ POST : writes
    USER ||--o{ COMMENT : writes
    POST ||--o{ COMMENT : has
    POST ||--o{ TAG : "tagged with"
    POST {
        int id PK
        string title
        text body
        date published_at
    }
    USER {
        int id PK
        string username
        string email
    }
    COMMENT {
        int id PK
        text body
        date created_at
    }
    TAG {
        int id PK
        string name
    }
```
````

---

## Gantt Chart

```mermaid
gantt
    title Project Timeline
    dateFormat YYYY-MM-DD
    section Planning
        Requirements     :a1, 2025-01-01, 14d
        Design           :a2, after a1, 10d
    section Development
        Backend          :b1, after a2, 30d
        Frontend         :b2, after a2, 25d
        Integration      :b3, after b1, 10d
    section Testing
        QA               :c1, after b3, 14d
        UAT              :c2, after c1, 7d
    section Launch
        Deployment       :d1, after c2, 3d
```

Source:

````
```mermaid
gantt
    title Project Timeline
    dateFormat YYYY-MM-DD
    section Planning
        Requirements     :a1, 2025-01-01, 14d
        Design           :a2, after a1, 10d
    section Development
        Backend          :b1, after a2, 30d
        Frontend         :b2, after a2, 25d
        Integration      :b3, after b1, 10d
    section Testing
        QA               :c1, after b3, 14d
        UAT              :c2, after c1, 7d
    section Launch
        Deployment       :d1, after c2, 3d
```
````

---

## Pie Chart

```mermaid
pie title Languages in the Project
    "Rust" : 65
    "HTML/Tera" : 15
    "CSS" : 10
    "JavaScript" : 5
    "YAML" : 5
```

Source:

````
```mermaid
pie title Languages in the Project
    "Rust" : 65
    "HTML/Tera" : 15
    "CSS" : 10
    "JavaScript" : 5
    "YAML" : 5
```
````

---

## Gitgraph

```mermaid
gitGraph
    commit id: "init"
    commit id: "add-readme"
    branch feature/auth
    commit id: "login-form"
    commit id: "jwt-tokens"
    checkout main
    branch fix/typo
    commit id: "fix-typo"
    checkout main
    merge fix/typo
    checkout feature/auth
    commit id: "tests"
    checkout main
    merge feature/auth
    commit id: "release-v1"
```

Source:

````
```mermaid
gitGraph
    commit id: "init"
    commit id: "add-readme"
    branch feature/auth
    commit id: "login-form"
    commit id: "jwt-tokens"
    checkout main
    branch fix/typo
    commit id: "fix-typo"
    checkout main
    merge fix/typo
    checkout feature/auth
    commit id: "tests"
    checkout main
    merge feature/auth
    commit id: "release-v1"
```
````

---

## Flowchart with Subgraphs

```mermaid
flowchart LR
    subgraph Input
        A[Markdown Files]
        B[marmite.yaml]
        C[Templates]
    end
    subgraph Processing
        D[Parse Frontmatter]
        E[Convert to HTML]
        F[Render Templates]
    end
    subgraph Output
        G[HTML Pages]
        H[RSS Feeds]
        I[Sitemap]
    end
    A --> D
    B --> F
    C --> F
    D --> E
    E --> F
    F --> G
    F --> H
    F --> I
```

Source:

````
```mermaid
flowchart LR
    subgraph Input
        A[Markdown Files]
        B[marmite.yaml]
        C[Templates]
    end
    subgraph Processing
        D[Parse Frontmatter]
        E[Convert to HTML]
        F[Render Templates]
    end
    subgraph Output
        G[HTML Pages]
        H[RSS Feeds]
        I[Sitemap]
    end
    A --> D
    B --> F
    C --> F
    D --> E
    E --> F
    F --> G
    F --> H
    F --> I
```
````

---

## Mindmap

```mermaid
mindmap
    root((Static Site Generator))
        Content
            Markdown
            Frontmatter
            Media
        Templates
            Tera
            Shortcodes
            Themes
        Output
            HTML
            RSS
            Sitemap
            JSON Feed
        Features
            Search
            Syntax Highlighting
            Image Resize
            Live Reload
```

Source:

````
```mermaid
mindmap
    root((Static Site Generator))
        Content
            Markdown
            Frontmatter
            Media
        Templates
            Tera
            Shortcodes
            Themes
        Output
            HTML
            RSS
            Sitemap
            JSON Feed
        Features
            Search
            Syntax Highlighting
            Image Resize
            Live Reload
```
````

---

## Timeline

```mermaid
timeline
    title Marmite Evolution
    2023 : Initial release
         : Basic markdown to HTML
    2024 : Shortcodes support
         : Theme system
         : Image optimization
         : Search integration
    2025 : Native mermaid rendering
         : Workspace multi-site
         : Gallery system
         : ATProto integration
```

Source:

````
```mermaid
timeline
    title Marmite Evolution
    2023 : Initial release
         : Basic markdown to HTML
    2024 : Shortcodes support
         : Theme system
         : Image optimization
         : Search integration
    2025 : Native mermaid rendering
         : Workspace multi-site
         : Gallery system
         : ATProto integration
```
````
