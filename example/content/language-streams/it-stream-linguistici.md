---
tags: docs, features, i18n, multilingual
description: Scrivi contenuti in piu lingue con collegamento automatico delle traduzioni, pagine di stream per lingua e tag hreflang per la SEO utilizzando la funzione language streams di marmite.
date: 2026-07-02
title: Language Streams - Contenuti Multilingue
---

Marmite supporta siti multilingue attraverso le language streams. Ogni lingua diventa una stream con la propria pagina di elenco e feed RSS, mentre le traduzioni vengono automaticamente collegate con la navigazione "Disponibile anche in" e tag hreflang per la SEO.

## Configurazione

Dichiara le lingue disponibili in `marmite.yaml`:

```yaml
language: pt
languages:
  pt:
    name: "Portugues"
  en:
    name: "English"
  es:
    name: "Espanol"
  it:
    name: "Italiano"
```

Il campo `language` (che esiste gia e ha il valore predefinito `en`) determina la lingua predefinita. I contenuti nella lingua predefinita rimangono su `index.html`. Le altre lingue ottengono le proprie pagine di stream (`en.html`, `es.html`, `it.html`) e feed RSS (`en.rss`, `es.rss`, `it.rss`).

Quando `languages` non e configurato, tutte le funzionalita i18n sono disabilitate e i siti esistenti funzionano esattamente come prima.

## Organizzazione dei Contenuti

Ci sono quattro modi per organizzare contenuti multilingue. Tutti producono output HTML piatto.

### Opzione 1: Raggruppamento per Sottocartella (Auto-Scoperta)

Raggruppa le traduzioni in una sottocartella con il nome dello slug del contenuto base. I file con prefisso del codice lingua configurato vengono automaticamente rilevati e collegati:

```
content/hello/
  hello.md              # Lingua predefinita (pt)
  en-hello-world.md     # Traduzione in inglese
  es-hola-mundo.md      # Traduzione in spagnolo
```

Questo genera:
- `hello.html` - Post in portoghese, elencato in `index.html`
- `en-hello-world.html` - Post in inglese, elencato in `en.html`
- `es-hola-mundo.html` - Post in spagnolo, elencato in `es.html`

Tutte e tre le pagine mostrano automaticamente i link "Disponibile anche in" verso le altre.

### Opzione 2: Misto File Piatto + Sottocartella

Se hai un sito piatto esistente e vuoi aggiungere traduzioni senza spostare i file originali, crea una sottocartella che corrisponda allo slug del contenuto esistente:

```
content/
  hello.md              # File piatto esistente, slug: hello
  hello/
    pt-ola.md           # Traduzione in portoghese, collegata automaticamente
```

Marmite rileva che il nome della sottocartella `hello` corrisponde allo slug del file piatto e li collega come traduzioni.

### Opzione 3: File Piatti con Marcatori di Stream

Usa il pattern del marcatore `-S-` esistente per l'organizzazione piatta:

```
content/
  hello.md              # Lingua predefinita
  pt-S-ola.md           # Portoghese, stream: pt
```

Con questo pattern, devi collegare le traduzioni manualmente usando il campo `translations` nel frontmatter.

### Opzione 4: Solo Frontmatter

Imposta la stream e le traduzioni esplicitamente nel frontmatter:

```yaml
---
title: Hello World
date: 2024-01-01
translations:
  - pt-ola
  - es-hola
---
```

Il campo `translations` accetta una lista di slug. Marmite risolve ogni slug al contenuto reale, compila il codice lingua e il nome di visualizzazione dalla configurazione `languages`, e crea collegamenti bidirezionali.

## Campi del Frontmatter

### `language`

Imposta esplicitamente il codice lingua del contenuto:

```yaml
language: it
```

Normalmente non e necessario - la lingua viene dedotta dal nome della stream o dal rilevamento della sottocartella.

### `translations`

Collega manualmente alle traduzioni per slug:

```yaml
translations:
  - en-hello-world
  - pt-ola-mundo
```

Non e necessario quando si usa l'auto-scoperta per sottocartella (Opzioni 1 e 2), poiche le traduzioni vengono collegate automaticamente.

## Note di Compatibilita

- I contenuti nella lingua predefinita usano la stream `index` e appaiono nella pagina principale `index.html`
- Il rilevamento della lingua dal prefisso del nome file funziona solo all'interno delle sottocartelle, mai per file piatti nella radice dei contenuti
- Un post puo avere sia una `series` che una language stream - funzionano indipendentemente
- I siti senza `languages` configurato non vengono influenzati in alcun modo
