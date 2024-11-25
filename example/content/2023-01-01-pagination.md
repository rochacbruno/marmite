{"tags": "docs"}

<!-- This page uses JSON as front-matter, this format is support but not recommended, use YAML `---` or TOML `+++` -->

# Pagination

Yes! we have pagination!

Set `pagination: 10` on your `marmite.yaml`



The pagination template is very simple


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
