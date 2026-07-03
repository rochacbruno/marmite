---
tags: docs
language: es
stream: es
translations:
  - pagination
---

## Paginacion

Si, tenemos paginacion!

Configura `pagination: 10` en tu `marmite.yaml`

La plantilla de paginacion es muy simple

```html
<div class="pagination">
    <nav>
        <ul>
        {% if previous_page %}
            <li><a href="{{previous_page}}"><strong>&larr;</strong></a></li>
        {% endif %}
        </ul>
        <ul>
            <li><small>{{current_page_number}}/{{total_pages}}</small></li>
        </ul>
        <ul>
        {% if next_page %}
            <li><a href="{{next_page}}"><strong>&rarr;</strong></a></li>
        {% endif %}
        </ul>
    </nav>
</div>
```
