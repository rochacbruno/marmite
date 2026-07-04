---
tags: documentazione
description: Scrivi contenuti in piu lingue con collegamento automatico delle traduzioni, pagine di stream per lingua e tag hreflang per la SEO utilizzando la funzione language streams di marmite.
date: 2026-07-02
title: Language Streams - Contenuti Multilingue
---

Marmite supporta siti multilingue attraverso le language streams. Ogni lingua diventa una stream con la propria pagina di elenco e feed RSS, mentre le traduzioni vengono automaticamente collegate con la navigazione "Disponibile anche in" e tag hreflang per la SEO.

## Come Funziona

Le lingue vengono rilevate automaticamente dai contenuti. Basta impostare `language: xx` nel frontmatter oppure usare le convenzioni di denominazione delle sottocartelle, e marmite gestisce il resto - nessuna configurazione necessaria.

Il campo `language` in `marmite.yaml` (valore predefinito `en`) determina la lingua predefinita del sito. I contenuti nella lingua predefinita rimangono su `index.html`. Le altre lingue ottengono le proprie pagine di stream (`pt.html`, `es.html`) e feed RSS (`pt.rss`, `es.rss`).

### Opzionale: Nomi di Visualizzazione

Per impostazione predefinita, le language stream vengono etichettate con il loro codice a due lettere (ad esempio "pt", "es"). Per impostare nomi leggibili, aggiungi una sezione opzionale `languages` in `marmite.yaml`:

```yaml
language: en
languages:
  pt:
    display_name: "Portugues"
  es:
    display_name: "Espanol"
```

Questo segue lo stesso pattern di `streams:` e `series:` - la configurazione e puramente estetica. I siti senza contenuti multilingue non vengono influenzati in alcun modo.

## Organizzazione dei Contenuti

Ci sono quattro modi per organizzare contenuti multilingue. Tutti producono output HTML piatto.

### Opzione 1: Raggruppamento per Sottocartella (Auto-Scoperta)

Raggruppa le traduzioni in una sottocartella con il nome dello slug del contenuto base. I file con prefisso di un codice lingua ISO 639-1 vengono automaticamente rilevati e collegati:

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

### Opzione 3: Puntatore Translates

Ogni traduzione punta allo slug del contenuto originale usando il campo `translates`. Marmite costruisce automaticamente la rete completa di collegamenti bidirezionali:

```yaml
---
title: Ciao Mondo
date: 2024-01-01
language: it
translates: hello
---
```

Questo e piu semplice rispetto al mantenere una lista `translations` in ogni file. Basta impostare `translates` su ogni traduzione e marmite collega tutto automaticamente. Quando `language` e impostato su un valore diverso dalla lingua predefinita del sito e nessuna `stream` e definita esplicitamente, marmite inserisce automaticamente il contenuto nella stream della lingua corrispondente.

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

Il campo `translations` accetta una lista di slug. Marmite risolve ogni slug al contenuto reale, compila il codice lingua e il nome di visualizzazione, e crea collegamenti bidirezionali. Se il post A elenca il post B come traduzione, il post B riceve automaticamente un collegamento di ritorno al post A.

> [!IMPORTANT]
> Preferisci le opzioni 1 e 2 per il rilevamento automatico. Le opzioni 3 e 4 richiedono frontmatter esplicito ma offrono maggiore controllo sui collegamenti.

## Campi del Frontmatter

### `language`

Imposta esplicitamente il codice lingua del contenuto:

```yaml
language: it
```

Di solito non e necessario con il rilevamento per sottocartella (Opzioni 1 e 2), dove la lingua viene dedotta automaticamente. Usalo per le Opzioni 3 e 4, dove la lingua deve essere specificata nel frontmatter.

Quando `language` e impostato su un valore diverso dalla lingua predefinita del sito e nessuna `stream` e definita esplicitamente, marmite usa automaticamente la lingua come stream, cioe il contenuto verra pubblicato nella stream corrispondente (ad esempio `it.html`).

### `translates`

Punta una traduzione allo slug del contenuto originale:

```yaml
translates: hello
```

Marmite crea collegamenti bidirezionali tra il contenuto originale e tutte le sue traduzioni. Non e necessario con l'auto-scoperta per sottocartella (Opzioni 1 e 2). Vedi l'Opzione 3 sopra per i dettagli.

### `translations`

Collega manualmente alle traduzioni per slug:

```yaml
translations:
  - en-hello-world
  - pt-ola-mundo
```

Non e necessario quando si usa l'auto-scoperta per sottocartella (Opzioni 1 e 2) o il campo `translates:` (Opzione 3), poiche le traduzioni vengono collegate automaticamente.

## Note di Compatibilita

- I contenuti nella lingua predefinita usano la stream `index` e appaiono nella pagina principale `index.html`
- Le lingue vengono auto-popolate da tutti i contenuti osservati - nessuna configurazione necessaria
- Il rilevamento della lingua dal prefisso del nome file funziona solo all'interno delle sottocartelle, mai per file piatti nella radice dei contenuti (prevenendo falsi positivi come `essential-guide.md` rilevato come lingua `es`)
- Dopo la raccolta dei contenuti, una fase di scoperta delle traduzioni raggruppa i contenuti per sottocartella, elabora i puntatori `translates:` e risolve i riferimenti del frontmatter
- Un post puo avere sia una `series` che una language stream - funzionano indipendentemente
- Le collisioni di slug tra lingue sono evitate dal prefisso della stream negli slug (`en-hello` vs `es-hola`)
- Le pagine (contenuti senza date) possono avere `language` e `translations` per la visualizzazione nei template, ma non appaiono nelle pagine di elenco della stream
