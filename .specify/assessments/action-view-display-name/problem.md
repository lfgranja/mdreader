# Problem Definition: Título numérico no ACTION_VIEW via MediaStore

- **Slug**: action-view-display-name
- **Created**: 2026-11-21
- **Inputs used**: intake.md, research.md

## Problem Statement

Quando um documento Markdown é aberto a partir do gerenciador de arquivos do
Android, o sistema entrega uma URI de item do MediaStore
(`content://media/external/file/<id>`). O nome de exibição não é resolvido e a
interface identifica o documento por seu identificador numérico —
`document:1000848919.md` em vez de "Hamlet trecho.md". O arquivo é ingerido, o
snapshot é gravado e o fingerprint é calculado, mas a única informação que o
usuário tem para reconhecer o documento está errada. O defeito é específico do
caminho `ACTION_VIEW`/MediaStore: o mesmo documento via `ACTION_SEND` preserva o
nome e a reprodução.

## Affected Users & Stakeholders

- **Users**:
  - **Ouvinte que abre arquivos pelo app de arquivos**: perde a referência
    visual do documento e não consegue distingui-lo de outros itens com títulos
    numéricos.
  - **Ouvinte em trânsito**: depende do título exibido para confirmar que abriu
    o texto correto sem inspecionar a barra de status ou o logcat.
- **Stakeholders**:
  - **Mantenedor do projeto**: precisa fechar a regressão reaberta sem alterar
    o contrato JNI, o fingerprint ou os caminhos SAF/`ACTION_SEND` que já
    funcionam.
  - **Responsável pelo gate**: depende de que a mudança permaneça confinada a um
    arquivo Kotlin e de que `scripts/verify.sh` continue a autoridade única.

## Goals

- Resolver o nome de exibição também para URIs de item do MediaStore, usando a
  mesma coluna `_display_name` consultada na coleção correta.
- Fazer com que o ramo `ACTION_VIEW` sempre produza um URI sintético
  `content://display_name/<nome codificado>`, inclusive quando nenhum nome é
  obtido.
- Rejeitar nomes inutilizáveis (vazios, em branco ou somente dígitos) antes de
  propagá-los como título.
- Preservar integralmente SAF/diretório, `ACTION_SEND`, snapshot/BLAKE3 e a
  assinatura JNI.
- Tornar a correção verificável em host por inspeção estrutural do diff e do
  consumidor Rust, sem reivindicar verificação em dispositivo.

## Non-Goals

- Reduzir a latência de 183 s até o primeiro áudio (pré-roll de 60 s): registro
  **DEFER**, pertence à área de pré-síntese/T232 e não é deste ciclo.
- Alterar `queryName(resolver, treeUri, docId)` (SAF/diretório) ou o ramo
  `ACTION_SEND`.
- Alterar `blake3_256`, `writeSnapshot`, `onDocumentIngested` ou
  `picked_document_name` no Rust.
- Editar `specs/019`, `specs/009`, o bug ou qualquer `converge.md` de outra
  spec.
- Afirmar comportamento em dispositivo: toda evidência de device é `[NV]`.

## Success Metrics

- **Nome legível**: 100% dos URIs `content://media/<volume>/file/<id>` cuja
  coleção `MediaStore.Files` expõe `_display_name` produzem esse valor como
  título (baseline em host: 0% — o nome direto retornava `null`). Mensurável em
  host pela presença da consulta à coleção com selection `_id=?`.
- **URI sintético estável**: 100% dos caminhos `ACTION_VIEW` constroem
  `content://display_name/<Uri.encode(nome)>`, com zero ocorrências do fallback
  `uri.toString()` nesse ramo (baseline: 1 ocorrência).
- **Nomes inutilizáveis filtrados**: 100% dos nomes aceitos passam pelo
  predicado `isUsableName`; zero nomes somente dígitos ou vazios propagados, com
  fallback determinístico `document.md` (baseline: `1000848919` virava título).
- **Preservação de caminhos**: zero alterações em `ACTION_SEND`, SAF/diretório,
  BLAKE3/snapshot e assinatura JNI; o diff toca apenas imports, predicado,
  `queryName(resolver, uri)` e o ramo `ACTION_VIEW` (baseline: 1 arquivo, 36
  inserções/10 remoções).
- **Rastreabilidade em host**: 100% dos 5 FRs mapeados a tarefas T001..T012 e a
  símbolos reais, com `[NV]` explícito para device e DEFER registrado.

## Cost of Inaction

A regressão permanece aberta e cada abertura de arquivo pelo gerenciador
exibe um identificador numérico como título. Como o mesmo fingerprint pode ser
reutilizado entre sessões, o usuário não consegue associar o título ao
bookmark, e qualquer correção futura terá que redescobrir que a coluna já
existe — só muda a coleção consultada.

## Open Questions

- [NEEDS CLARIFICATION: validar `volume` contra volumes disponíveis ou aceitar o primeiro segmento com default `external`?]
- [NEEDS CLARIFICATION: manter `document.md` como fallback fixo ou sanitizar o último segmento quando ele for legível?]
- [NV] Resolução do nome e título final em dispositivo real — fora do escopo de verificação deste ciclo.
