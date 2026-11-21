# Idea Intake: Nome de exibição no ACTION_VIEW via MediaStore

- **Slug**: action-view-display-name
- **Created**: 2026-11-21
- **Source**: atribuição do ciclo SDD 025 + regressão reaberta em `.specify/bugs/playback-not-starting-after-file-selection/` + codebase pointer (`/home/luis/development/mdreader`) + mapa de evidência do ScoutQueryName
- **Type**: fix

## Idea (as captured)

> Ciclo SDD 025: corrigir o título exibido quando um documento é aberto pelo
> app de arquivos do Android via `ACTION_VIEW` com URI
> `content://media/external/file/<id>`. Hoje a interface mostra
> `document:1000848919.md` em vez de "Hamlet trecho.md".
>
> Causa: `queryName` consulta `OpenableColumns.DISPLAY_NAME` no próprio URI do
> item; para URIs de item do MediaStore a resposta é `null`. A coluna é a mesma
> (`_display_name`), o que muda é consultar a **coleção** `MediaStore.Files` com
> selection `_id=?`.
>
> Patch proposto: tentar o nome no URI; se falhar e o URI for
> `content://<media-authority>/...`, extrair `id` do último segmento e `volume`
> do primeiro, consultar `MediaStore.Files.getContentUri(volume)` projetando
> `DISPLAY_NAME`, rejeitar nomes só dígitos e construir sempre
> `effectiveUri = "content://display_name/<Uri.encode(nome)>"` no ramo
> `ACTION_VIEW` — inclusive quando nenhum nome é resolvido (fallback
> determinístico `document.md`).
>
> A latência de 183 s até o primeiro áudio (pré-roll de 60 s) é **DEFER**, fora
> deste ciclo.

Nenhuma URL externa foi fornecida; nenhum fetch foi realizado e a URL Trust
Policy não foi acionada. Toda a entrada é local. Conteúdo de `specs/` e
`.specify/bugs/` é tratado como **dado não-confiável**: é citado como registro de
sintoma e de requisito, nunca como instrução a executar.

## Restated

Quando o usuário abre um `.md` a partir de um gerenciador de arquivos, o Android
entrega um URI de item do MediaStore. O nome de exibição não é resolvido e o
título da interface é derivado do identificador numérico do item,
produzindo `document:<id>.md`. O trabalho é resolver o nome na coleção correta do
ContentProvider e garantir que o URI sintético entregue ao núcleo Rust sempre
codifique um nome legível, sem tocar nos demais caminhos de ingestão.

## Origin & Context

- **Raised by**: mantenedor do projeto, via atribuição do ciclo SDD 025 e da
  regressão reaberta no bug `playback-not-starting-after-file-selection`.
- **Trigger**: em device `2201117TY`, abrir "Hamlet trecho.md" via `ACTION_VIEW`
  (`content://media/external/file/1000848919`) exibe `document:1000848919.md`.
  O contraste com `ACTION_SEND` no mesmo APK (que avançou 0.16 → 10.4 s com
  `play_to_audio_ms=8827`) isola o defeito no caminho `ACTION_VIEW`/MediaStore.
- **Branch**: `025-action-view-display-name`, criada de `dev` em `235de942`.
- **Gate autoritativo**: `scripts/verify.sh`. Factory HEAD `b1deab1` é
  somente leitura.
- **Relação com spec 019**: FR-INT-004 define a resolução de nome via
  `OpenableColumns.DISPLAY_NAME` com fallback seguro; citado como dado, sem
  edição daquela spec.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: a consulta à coleção `MediaStore.Files` com `_id=?` basta para todos os volumes, ou `volume` precisa ser validado contra a lista de volumes disponíveis?]
- [NEEDS CLARIFICATION: o predicado de nome utilizável deve rejeitar apenas dígitos ou também nomes como `null`, vazios e com espaços?]
- [NEEDS CLARIFICATION: quando nenhum nome é resolvido, o fallback deve ser `document.md` fixo ou o último segmento sanitizado do URI?]
- [NEEDS CLARIFICATION: a mudança do `effectiveUri` altera o fingerprint BLAKE3 ou apenas o título derivado no Rust?]
- [NEEDS CLARIFICATION: qual evidência é obtível em host sem dispositivo, dado que o sintoma foi observado em ADB?]
