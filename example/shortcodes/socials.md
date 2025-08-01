{# Display social network links #}
## My social networks

{% if site_data.site.extra.social_networks %}
- LinkedIn: {{ site_data.site.extra.social_networks.linkedin.url | default(value="Not configured") }}
- Github: {{ site_data.site.extra.social_networks.github.url | default(value="Not configured") }}
{% else %}
Social networks not configured in marmite.yaml
{% endif %}