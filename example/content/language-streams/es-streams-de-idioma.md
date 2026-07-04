---
tags: documentacion
description: Escribe contenido en multiples idiomas con enlaces de traduccion automaticos, paginas de stream por idioma y etiquetas hreflang para SEO usando la funcion de language streams de marmite.
date: 2026-07-02
title: Language Streams - Contenido Multilingue
---

Marmite soporta sitios multilingues a traves de language streams. Cada idioma se convierte en una stream con su propia pagina de listado y feed RSS, mientras que las traducciones se vinculan automaticamente con navegacion "Tambien disponible en" y etiquetas hreflang para SEO.

## Como Funciona

Los idiomas se detectan automaticamente a partir del contenido. Solo establece `language: xx` en el frontmatter o usa convenciones de nombres de subcarpetas, y marmite se encarga del resto - no se requiere configuracion.

El campo `language` en `marmite.yaml` (valor predeterminado `en`) determina el idioma predeterminado del sitio. El contenido en el idioma predeterminado permanece en `index.html`. Los demas idiomas obtienen sus propias paginas de stream (`pt.html`, `es.html`) y feeds RSS (`pt.rss`, `es.rss`).

### Opcional: Nombres para Visualizacion

Por defecto, las streams de idioma se etiquetan con su codigo de dos letras (por ejemplo, "pt", "es"). Para establecer nombres legibles, agrega una seccion opcional `languages` en `marmite.yaml`:

```yaml
language: en
languages:
  pt:
    display_name: "Portugues"
  es:
    display_name: "Espanol"
```

Esto sigue el mismo patron que `streams:` y `series:` - la configuracion es puramente estetica. Los sitios sin contenido en multiples idiomas no se ven afectados.

## Organizacion del Contenido

Hay cuatro formas de organizar contenido multilingue. Todas producen salida HTML plana.

### Opcion 1: Agrupacion por Subcarpeta (Auto-Descubrimiento)

Agrupa las traducciones en una subcarpeta con el nombre del slug del contenido base. Los archivos con prefijo de codigo de idioma ISO 639-1 se detectan y vinculan automaticamente:

```
content/hello/
  hello.md              # Idioma predeterminado (en)
  pt-ola-mundo.md       # Traduccion al portugues
  es-hola-mundo.md      # Traduccion al espanol
```

Esto genera:
- `hello.html` - Post en ingles, listado en `index.html`
- `pt-ola-mundo.html` - Post en portugues, listado en `pt.html`
- `es-hola-mundo.html` - Post en espanol, listado en `es.html`

Las tres paginas muestran automaticamente enlaces "Tambien disponible en" hacia las otras.

> [!TIP]
> La subcarpeta tambien puede tener la fecha, por ejemplo `content/2026-07-02-hello/`, asi no necesitas especificar la fecha en el frontmatter de cada traduccion.

### Opcion 2: Mixto Archivo Plano + Subcarpeta

Si tienes un sitio plano existente y quieres agregar traducciones sin mover los archivos originales, crea una subcarpeta que coincida con el slug del contenido existente:

```
content/
  hello.md              # Archivo plano existente, slug: hello
  hello/
    pt-ola.md           # Traduccion al portugues, vinculada automaticamente
```

Marmite detecta que el nombre de la subcarpeta `hello` coincide con el slug del archivo plano y los vincula como traducciones.

> [!IMPORTANT]
> Los nombres de las subcarpetas deben coincidir con el slug del post original (no el nombre del archivo, sino el slug resuelto, a veces tomado del titulo) para ser vinculados automaticamente como traducciones.

### Opcion 3: Puntero Translates

Cada traduccion apunta al slug del contenido original usando el campo `translates`. Marmite construye la red completa de enlaces bidireccionales automaticamente:

```yaml
---
title: Hola Mundo
date: 2024-01-01
language: es
translates: hello
---
```

Esto es mas simple que mantener una lista `translations` en cada archivo. Solo establece `translates` en cada traduccion y marmite conecta todo. Cuando `language` se define con un valor diferente al idioma predeterminado del sitio, marmite automaticamente coloca el contenido en la stream del idioma correspondiente.

### Opcion 4: Enlace de Traduccion via Frontmatter

Define el idioma y las traducciones explicitamente en el frontmatter:

```yaml
---
title: Hello World
date: 2024-01-01
language: en # se puede omitir porque es el idioma predeterminado
translations:
  - pt-ola  # luego escribes un post con slug `ola` y language: pt
  - es-hola
---
```

El campo `translations` acepta una lista de slugs. Marmite resuelve cada slug al contenido real, completa el codigo de idioma y el nombre de visualizacion, y crea enlaces bidireccionales. Si el post A lista al post B como traduccion, el post B automaticamente recibe un enlace de vuelta al post A.

> [!IMPORTANT]
> Prefiere las opciones 1 y 2 para auto-descubrimiento. Las opciones 3 y 4 requieren frontmatter explicito pero te dan mas control sobre la vinculacion.

## Campos de Frontmatter

### `language`

Define explicitamente el codigo de idioma del contenido:

```yaml
language: es
```

Normalmente no es necesario con la deteccion por subcarpeta (Opciones 1 y 2) - el idioma se infiere automaticamente. Usalo para las Opciones 3 y 4.

Cuando `language` se define con un valor diferente al idioma predeterminado del sitio y no se establece una `stream` explicita, marmite automaticamente usa el idioma como stream, es decir, este post se publicara en la stream es.html.

### `translates`

Apunta una traduccion al slug del contenido original:

```yaml
translates: hello
```

Marmite crea enlaces bidireccionales entre el contenido original y todas sus traducciones. No es necesario con auto-descubrimiento por subcarpeta. Ver Opcion 3 arriba para mas detalles.

### `translations`

Vincula manualmente a traducciones por slug:

```yaml
translations:
  - en-hello-world
  - pt-ola-mundo
```

No es necesario al usar auto-descubrimiento por subcarpeta (Opciones 1 y 2) o `translates:` (Opcion 3), ya que las traducciones se vinculan automaticamente.

## Notas de Compatibilidad

- El contenido del idioma predeterminado usa la stream `index` y aparece en la pagina principal `index.html`
- Los idiomas se auto-detectan a partir de todo el contenido observado - no se requiere configuracion
- La deteccion de idioma por prefijo de nombre de archivo solo funciona dentro de subcarpetas, nunca para archivos planos en la raiz del contenido (previniendo falsos positivos como `essential-guide.md` siendo detectado como idioma `es`)
- Los punteros `translates:` se procesan despues de la recoleccion de contenido para construir enlaces bidireccionales
- Un post puede tener tanto una `series` como una language stream - funcionan independientemente
- Las colisiones de slug entre idiomas se evitan con el prefijo de stream en los slugs (`en-hello` vs `es-hola`)
- Las paginas (contenido sin fechas) pueden tener `language` y `translations` para visualizacion en plantillas, pero no aparecen en paginas de listado de stream
