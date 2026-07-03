---
tags: documentação
description: Escreva conteudo em multiplos idiomas com links de traducao automaticos, paginas de stream por idioma e tags hreflang para SEO usando o recurso de language streams do marmite.
date: 2026-07-02
title: Language Streams - Conteudo Multilingual
---

O marmite suporta sites multilingues atraves de language streams. Cada idioma se torna uma stream com sua propria pagina de listagem e feed RSS, enquanto as traducoes sao automaticamente vinculadas com navegacao "Tambem disponivel em" e tags hreflang para SEO.

## Configuracao

Declare os idiomas disponiveis no `marmite.yaml`:

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

O campo `language` (que ja existe e tem o padrao `en`) determina o idioma padrao. O conteudo no idioma padrao fica no `index.html`. Outros idiomas recebem suas proprias paginas de stream (`pt.html`, `es.html`) e feeds RSS (`pt.rss`, `es.rss`).

Quando `languages` nao esta configurado, todos os recursos de i18n ficam desabilitados e sites existentes funcionam exatamente como antes.

## Organizacao do Conteudo

Existem quatro maneiras de organizar conteudo multilingual. Todas produzem saida HTML plana.

### Opcao 1: Agrupamento por Subpasta (Auto-Descoberta)

Agrupe traducoes em uma subpasta com o nome do slug do conteudo base. Arquivos com prefixo de codigo de idioma configurado sao automaticamente detectados e vinculados:

```
content/hello/
  hello.md              # Idioma padrao (en)
  pt-ola-mundo.md       # Traducao em portugues
  es-hola-mundo.md      # Traducao em espanhol
```

Isso gera:
- `hello.html` - Post em ingles, listado no `index.html`
- `pt-ola-mundo.html` - Post em portugues, listado no `pt.html`
- `es-hola-mundo.html` - Post em espanhol, listado no `es.html`

Todas as tres paginas mostram automaticamente links "Tambem disponivel em" para as outras.

> [!TIP]
> A subpasta tambem pode ter a data nela, por exemplo `content/2026-07-02-hello/`, assim voce nao precisa especificar a data no frontmatter de cada traducao.

### Opcao 2: Misto Arquivo Plano + Subpasta

Se voce tem um site plano existente e quer adicionar traducoes sem mover os arquivos originais, crie uma subpasta com o mesmo nome do slug do conteudo existente:

```
content/
  hello.md              # Arquivo plano existente, slug: hello
  hello/
    pt-ola.md           # Traducao em portugues, vinculada automaticamente
```

O marmite detecta que o nome da subpasta `hello` corresponde ao slug do arquivo plano e os vincula como traducoes.

> [!IMPORTANT]
> Os nomes das subpastas devem corresponder ao slug do post original (nao o nome do arquivo, mas o slug resolvido, as vezes retirado do titulo) para serem automaticamente vinculados como traducoes.

### Opcao 3: Marcadores de Stream

Use o padrao de marcador `-S-` existente para organizacao plana:

```
content/
  hello.md              # Idioma padrao
  pt-S-ola.md           # Portugues, stream: pt
```

Ou definindo a language stream diretamente no frontmatter:

```yaml
---
title: ola mundo
date: 2024-01-01
stream: pt
translations:
  - en-hello
---
```

Com este padrao, voce precisa vincular traducoes manualmente usando o campo `translations` no frontmatter (veja abaixo).

### Opcao 4: Link de Traducao via Frontmatter

Defina o idioma e as traducoes explicitamente no frontmatter:

```yaml
---
title: Hello World
date: 2024-01-01
language: en # pode omitir porque e o idioma padrao
translations:
  - pt-ola  # entao voce escreve um post com slug `ola` e language: pt
  - es-hola
---
```

O campo `translations` aceita uma lista de slugs. O marmite resolve cada slug para o conteudo real, preenche o codigo do idioma e o nome de exibicao a partir da configuracao `languages`, e cria links bidirecionais. Se o post A lista o post B como traducao, o post B automaticamente recebe um link de volta para o post A.

## Campos de Frontmatter

### `language`

Define explicitamente o codigo de idioma do conteudo:

```yaml
language: pt
```

Normalmente nao e necessario - o idioma e inferido do nome da stream ou da deteccao por subpasta.

Quando `language` e definido, mas nenhuma stream e definida, o marmite assume que a stream e a mesma que o idioma, ou seja, este post sera publicado na stream pt.html.

### `translations`

Vincula manualmente a traducoes por slug:

```yaml
translations:
  - en-hello-world
  - es-hola-mundo
```

Nao e necessario ao usar auto-descoberta por subpasta (Opcoes 1 e 2), pois as traducoes sao vinculadas automaticamente.

## Notas de Compatibilidade

- O conteudo do idioma padrao usa a stream `index` e aparece na pagina principal `index.html`
- A deteccao de idioma por prefixo de nome de arquivo so funciona dentro de subpastas, nunca para arquivos planos na raiz do conteudo (evitando falsos positivos como `essential-guide.md` sendo detectado como idioma `es`)
- Um post pode ter tanto uma `series` quanto uma language stream - funcionam independentemente
- Colisoes de slug entre idiomas sao evitadas pelo prefixo da stream nos slugs (`en-hello` vs `es-hola`)
- Paginas (conteudo sem datas) podem ter `language` e `translations` para exibicao no template, mas nao aparecem em paginas de listagem de stream
