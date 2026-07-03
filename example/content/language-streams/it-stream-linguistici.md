---
tags: documentazione
description: Scrivi contenuti in piu lingue con collegamento automatico delle traduzioni, pagine di stream per lingua e tag hreflang per la SEO utilizzando la funzione language streams di marmite.
date: 2026-07-02
title: Language Streams - Contenuti Multilingue
---

Marmite supporta siti multilingue attraverso le language streams. Ogni lingua diventa una stream con la propria pagina di elenco e feed RSS, mentre le traduzioni vengono automaticamente collegate con la navigazione "Disponibile anche in" e tag hreflang per la SEO.

## Configurazione

Dichiara le lingue disponibili in `marmite.yaml`:

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

Il campo `language` (che esiste gia e ha il valore predefinito `en`) determina la lingua predefinita. I contenuti nella lingua predefinita rimangono su `index.html`. Le altre lingue ottengono le proprie pagine di stream (`pt.html`, `es.html`) e feed RSS (`pt.rss`, `es.rss`).

Quando `languages` non e configurato, tutte le funzionalita i18n sono disabilitate e i siti esistenti funzionano esattamente come prima.

## Organizzazione dei Contenuti

Ci sono quattro modi per organizzare contenuti multilingue. Tutti producono output HTML piatto.

### Opzione 1: Raggruppamento per Sottocartella (Auto-Scoperta)

Raggruppa le traduzioni in una sottocartella con il nome dello slug del contenuto base. I file con prefisso del codice lingua configurato vengono automaticamente rilevati e collegati:

```
content/hello/
  hello.md              # Lingua predefinita (en)
  pt-ola-mundo.md       # Traduzione in portoghese
  es-hola-mundo.md      # Traduzione in spagnolo
```

Questo genera:
- `hello.html` - Post in inglese, elencato in `index.html`
- `pt-ola-mundo.html` - Post in portoghese, elencato in `pt.html`
- `es-hola-mundo.html` - Post in spagnolo, elencato in `es.html`

Tutte e tre le pagine mostrano automaticamente i link "Disponibile anche in" verso le altre.

> [!TIP]
> La sottocartella puo anche avere la data, ad esempio `content/2026-07-02-hello/`, cosi non devi specificare la data nel frontmatter di ogni traduzione.

### Opzione 2: Misto File Piatto + Sottocartella

Se hai un sito piatto esistente e vuoi aggiungere traduzioni senza spostare i file originali, crea una sottocartella che corrisponda allo slug del contenuto esistente:

```
content/
  hello.md              # File piatto esistente, slug: hello
  hello/
    pt-ola.md           # Traduzione in portoghese, collegata automaticamente
```

Marmite rileva che il nome della sottocartella `hello` corrisponde allo slug del file piatto e li collega come traduzioni.

> [!IMPORTANT]
> I nomi delle sottocartelle devono corrispondere allo slug del post originale (non il nome del file, ma lo slug risolto, a volte preso dal titolo) per essere collegati automaticamente come traduzioni.

### Opzione 3: Marcatori di Stream

Usa il pattern del marcatore `-S-` esistente per l'organizzazione piatta:

```
content/
  hello.md              # Lingua predefinita
  pt-S-ola.md           # Portoghese, stream: pt
```

Oppure impostando la language stream direttamente nel frontmatter:

```yaml
---
title: ola mundo
date: 2024-01-01
stream: pt
translations:
  - en-hello
---
```

Con questo pattern, devi collegare le traduzioni manualmente usando il campo `translations` nel frontmatter (vedi sotto).

### Opzione 4: Collegamento Traduzione via Frontmatter

Imposta la lingua e le traduzioni esplicitamente nel frontmatter:

```yaml
---
title: Hello World
date: 2024-01-01
language: en # puo essere omesso perche e la lingua predefinita
translations:
  - pt-ola  # poi scrivi un post con slug `ola` e language: pt
  - es-hola
---
```

Il campo `translations` accetta una lista di slug. Marmite risolve ogni slug al contenuto reale, compila il codice lingua e il nome di visualizzazione dalla configurazione `languages`, e crea collegamenti bidirezionali. Se il post A elenca il post B come traduzione, il post B riceve automaticamente un collegamento di ritorno al post A.

## Campi del Frontmatter

### `language`

Imposta esplicitamente il codice lingua del contenuto:

```yaml
language: it
```

Normalmente non e necessario - la lingua viene dedotta dal nome della stream o dal rilevamento della sottocartella.

Quando `language` e impostato, ma nessuna stream e definita, marmite assume che la stream sia la stessa della lingua, cioe questo post verra pubblicato nella stream it.html.

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
- Il rilevamento della lingua dal prefisso del nome file funziona solo all'interno delle sottocartelle, mai per file piatti nella radice dei contenuti (prevenendo falsi positivi come `essential-guide.md` rilevato come lingua `es`)
- Un post puo avere sia una `series` che una language stream - funzionano indipendentemente
- Le collisioni di slug tra lingue sono evitate dal prefisso della stream negli slug (`en-hello` vs `es-hola`)
- Le pagine (contenuti senza date) possono avere `language` e `translations` per la visualizzazione nei template, ma non appaiono nelle pagine di elenco della stream
