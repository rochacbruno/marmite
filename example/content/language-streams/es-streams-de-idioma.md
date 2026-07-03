---
tags: docs, features, i18n, multilingual
description: Escribe contenido en multiples idiomas con enlaces de traduccion automaticos, paginas de stream por idioma y etiquetas hreflang para SEO usando la funcion de language streams de marmite.
date: 2026-07-02
title: Language Streams - Contenido Multilingue
---

Marmite soporta sitios multilingues a traves de language streams. Cada idioma se convierte en una stream con su propia pagina de listado y feed RSS, mientras que las traducciones se vinculan automaticamente con navegacion "Tambien disponible en" y etiquetas hreflang para SEO.

## Configuracion

Declara los idiomas disponibles en `marmite.yaml`:

```yaml
language: en
languages:
  pt:
    name: "Portugues"
  en:
    name: "English"
  es:
    name: "Espanol"
```

El campo `language` (que ya existe y tiene el valor predeterminado `en`) determina el idioma predeterminado. El contenido en el idioma predeterminado permanece en `index.html`. Los demas idiomas obtienen sus propias paginas de stream (`pt.html`, `es.html`) y feeds RSS (`pt.rss`, `es.rss`).

Cuando `languages` no esta configurado, todas las funciones de i18n se deshabilitan y los sitios existentes funcionan exactamente como antes.

## Organizacion del Contenido

Hay cuatro formas de organizar contenido multilingue. Todas producen salida HTML plana.

### Opcion 1: Agrupacion por Subcarpeta (Auto-Descubrimiento)

Agrupa las traducciones en una subcarpeta con el nombre del slug del contenido base. Los archivos con prefijo de codigo de idioma configurado se detectan y vinculan automaticamente:

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

### Opcion 3: Marcadores de Stream

Usa el patron de marcador `-S-` existente para organizacion plana:

```
content/
  hello.md              # Idioma predeterminado
  pt-S-ola.md           # Portugues, stream: pt
```

O estableciendo la language stream directamente en el frontmatter:

```yaml
---
title: ola mundo
date: 2024-01-01
stream: pt
translations:
  - en-hello
---
```

Con este patron, necesitas vincular traducciones manualmente usando el campo `translations` en el frontmatter (ver abajo).

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

El campo `translations` acepta una lista de slugs. Marmite resuelve cada slug al contenido real, completa el codigo de idioma y el nombre de visualizacion desde la configuracion `languages`, y crea enlaces bidireccionales. Si el post A lista al post B como traduccion, el post B automaticamente recibe un enlace de vuelta al post A.

## Campos de Frontmatter

### `language`

Define explicitamente el codigo de idioma del contenido:

```yaml
language: es
```

Normalmente no es necesario - el idioma se infiere del nombre de la stream o de la deteccion por subcarpeta.

Cuando `language` esta definido, pero no se define ninguna stream, marmite asume que la stream es la misma que el idioma, es decir, este post se publicara en la stream es.html.

### `translations`

Vincula manualmente a traducciones por slug:

```yaml
translations:
  - en-hello-world
  - pt-ola-mundo
```

No es necesario al usar auto-descubrimiento por subcarpeta (Opciones 1 y 2), ya que las traducciones se vinculan automaticamente.

## Notas de Compatibilidad

- El contenido del idioma predeterminado usa la stream `index` y aparece en la pagina principal `index.html`
- La deteccion de idioma por prefijo de nombre de archivo solo funciona dentro de subcarpetas, nunca para archivos planos en la raiz del contenido (previniendo falsos positivos como `essential-guide.md` siendo detectado como idioma `es`)
- Un post puede tener tanto una `series` como una language stream - funcionan independientemente
- Las colisiones de slug entre idiomas se evitan con el prefijo de stream en los slugs (`en-hello` vs `es-hola`)
- Las paginas (contenido sin fechas) pueden tener `language` y `translations` para visualizacion en plantillas, pero no aparecen en paginas de listado de stream
