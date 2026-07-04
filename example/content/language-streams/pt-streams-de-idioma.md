---
tags: documentação
description: Escreva conteudo em multiplos idiomas com links de traducao automaticos, paginas de stream por idioma e tags hreflang para SEO usando o recurso de language streams do marmite.
date: 2026-07-02
title: Language Streams - Conteudo Multilingual
---

O marmite suporta sites multilingues atraves de language streams. Cada idioma se torna uma stream com sua propria pagina de listagem e feed RSS, enquanto as traducoes sao automaticamente vinculadas com navegacao "Tambem disponivel em" e tags hreflang para SEO.

## Como Funciona

Os idiomas sao auto-detectados a partir do conteudo. Basta definir `language: xx` no frontmatter ou usar convencoes de nomenclatura por subpasta, e o marmite cuida do resto - nenhuma configuracao e necessaria.

O campo `language` no `marmite.yaml` (padrao `en`) determina o idioma padrao do site. O conteudo no idioma padrao fica no `index.html`. Outros idiomas recebem suas proprias paginas de stream (`pt.html`, `es.html`) e feeds RSS (`pt.rss`, `es.rss`).

### Opcional: Nomes de Exibicao

Por padrao, as language streams sao rotuladas com seu codigo de duas letras (ex., "pt", "es"). Para definir nomes legiveis, adicione uma secao opcional `languages` ao `marmite.yaml`:

```yaml
language: en
languages:
  pt:
    display_name: "Portugues"
  es:
    display_name: "Espanol"
```

Isso segue o mesmo padrao de `streams:` e `series:` - a configuracao e puramente cosmetica. Sites sem conteudo em outros idiomas nao sao afetados.

## Organizacao do Conteudo

Existem quatro formas de organizar conteudo multilingual. Todas produzem saida HTML plana.

### Opcao 1: Agrupamento por Subpasta (Auto-Descoberta)

Agrupe traducoes em uma subpasta com o nome do slug do conteudo base. Arquivos com prefixo de codigo de idioma ISO 639-1 sao automaticamente detectados e vinculados:

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

### Opcao 3: Ponteiro Translates

Cada traducao aponta para o slug do conteudo original usando o campo `translates`. O marmite constroi a rede completa de links bidirecionais automaticamente. Quando `language` e definido com um valor diferente do idioma padrao do site, o marmite automaticamente coloca o conteudo na stream de idioma correspondente:

```yaml
---
title: Ola Mundo
date: 2024-01-01
language: pt
translates: hello
---
```

Isso e mais simples do que manter uma lista `translations` em cada arquivo. Basta definir `translates` em cada traducao e o marmite conecta tudo.

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

> [!IMPORTANT]
> Prefira as opcoes 1 e 2 para auto-descoberta. As opcoes 3 e 4 exigem frontmatter explicito, mas oferecem mais controle sobre a vinculacao.

## Campos de Frontmatter

### `language`

Define explicitamente o codigo de idioma do conteudo:

```yaml
language: pt
```

Normalmente nao e necessario com deteccao por subpasta (Opcoes 1 e 2) - o idioma e inferido automaticamente. Use para as Opcoes 3 e 4 quando o conteudo nao esta em subpastas.

Quando `language` e definido com um valor diferente do idioma padrao do site e nenhuma stream e definida explicitamente, o marmite automaticamente usa o idioma como stream, ou seja, este post sera publicado na stream pt.html.

### `translates`

Aponta uma traducao para o slug do conteudo original:

```yaml
translates: hello
```

O marmite cria links bidirecionais entre o conteudo original e todas as suas traducoes. Nao e necessario com auto-descoberta por subpasta (Opcoes 1 e 2). Veja a Opcao 3 acima para detalhes.

### `translations`

Vincula manualmente a traducoes por slug:

```yaml
translations:
  - en-hello-world
  - es-hola-mundo
```

Nao e necessario ao usar auto-descoberta por subpasta (Opcoes 1 e 2) ou `translates:` (Opcao 3), pois as traducoes sao vinculadas automaticamente.

## Como Funciona Internamente

1. Durante a coleta de conteudo, arquivos em subpastas com prefixo de codigo de idioma ISO 639-1 (ex., `en-`) sao detectados e atribuidos a essa language stream
2. Os idiomas sao auto-populados a partir de todo o conteudo observado - nenhuma configuracao e necessaria
3. Apos toda a coleta de conteudo, uma fase de descoberta de traducoes agrupa conteudo por subpasta, processa ponteiros `translates:` e resolve referencias de frontmatter
4. Todos os membros de um grupo de traducao recebem links cruzados com entradas `TranslationRef` contendo o codigo do idioma, nome de exibicao, slug e titulo
5. Os templates renderizam links de traducao e tags hreflang a partir dessas referencias

## Notas de Compatibilidade

- O conteudo do idioma padrao usa a stream `index` e aparece na pagina principal `index.html`
- A deteccao de idioma por prefixo de nome de arquivo so funciona dentro de subpastas, nunca para arquivos planos na raiz do conteudo (evitando falsos positivos como `essential-guide.md` sendo detectado como idioma `es`)
- Um post pode ter tanto uma `series` quanto uma language stream - funcionam independentemente
- Colisoes de slug entre idiomas sao evitadas pelo prefixo da stream nos slugs (`en-hello` vs `es-hola`)
- Paginas (conteudo sem datas) podem ter `language` e `translations` para exibicao no template, mas nao aparecem em paginas de listagem de stream
