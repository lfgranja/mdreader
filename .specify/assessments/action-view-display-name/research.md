# Idea Research: Nome de exibição no ACTION_VIEW via MediaStore

- **Slug**: action-view-display-name
- **Created**: 2026-11-21
- **Evidence confidence (overall)**: high para o código em host (diff verificado linha a linha); o comportamento em dispositivo permanece `[NV]`

## Users & Demand

- O ouvinte que abre um documento pelo gerenciador de arquivos espera ver o nome
  do arquivo na interface; o que aparece é identificador numérico do item
  MediaStore (`document:1000848919.md` no lugar de "Hamlet trecho.md") —
  [source: relatório de regressão no bug `playback-not-starting-after-file-selection/fix.md`, 2026-11-21] (confidence: high para o sintoma reportado, `[NV]` para reprodução neste ciclo).
- A demanda é inferida do contrato de titulação já registrado em FR-INT-004 da
  spec 019 (`specs/019-intent-filter-ingress/spec.md:20`), citado como dado, não
  como instrução [source: repositório local] (confidence: high).
- Não há telemetria de uso anexada a este ciclo; a prioridade vem do impacto
  direto na identificação do documento aberto [ASSUMPTION] (confidence: medium).

## Prior Art

- **Código atual em host (baseline antes do patch)**: `queryName(resolver, uri)`
  em `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt:638-643` consultava
  apenas `OpenableColumns.DISPLAY_NAME` no URI recebido e devolvia `null` em
  exceção. O ramo `ACTION_VIEW` (`DocIngest.kt:768-783`) montava
  `effectiveUri` com `content://display_name/<nome>` somente quando o nome era
  não nulo e não vazio; caso contrário devolvia `uri.toString()` — a origem do
  título `document:1000848919.md` [source: `git diff` em host] (confidence: high).
- **Contrato de consumo no Rust (precedente interno decisivo)**:
  `picked_document_name` (`src-tauri/src/shell_jni.rs:1103-1134`) já separa o
  último segmento da URI original, aplica `percent_decode`, reduz a última
  componente e acrescenta a extensão do snapshot. Portanto
  `content://display_name/Hamlet%20trecho.md` já produz "Hamlet trecho.md" sem
  qualquer mudança no Rust [source: leitura em host] (confidence: high).
- **Mesma coluna, coleção diferente**: `OpenableColumns.DISPLAY_NAME` é o alias
  canônico de `_display_name`; URIs de item
  (`content://media/<volume>/file/<id>`) podem não responder à consulta
  direta, mas a coleção `MediaStore.Files.getContentUri(volume)` responde com
  selection `${BaseColumns._ID}=?` [source: mapa de evidência ScoutQueryName +
  diff em host] (confidence: high para a forma da consulta; `[NV]` para o
  resultado em device).
- **Caminho paralelo já correto**: `queryName(resolver, treeUri, docId)`
  (`DocIngest.kt:634-638`) usa `DocumentsContract` para o SAF/diretório e não é
  tocado; `ACTION_SEND` (`DocIngest.kt:785+`) usa `resolveFormat` e a URI bruta
  [source: leitura em host] (confidence: high).
- **Fingerprint**: `blake3_256(bytes)` e `writeSnapshot` precedem o JNI e não
  consultam nome algum; mudar apenas `effectiveUri` altera o título, não o
  fingerprint [source: `DocIngest.kt:734-751`, `shell_jni.rs:849-893`]
  (confidence: high).

## Market & Context

- O custo de não agir é a continuidade do sintoma em todos os `ACTION_VIEW`
  oriundos do MediaStore — o documento é ingerido mas identificado por um
  número, o que prejudica tanto a leitura da interface quanto a associação com
  bookmarks pelo título [source: escopo da atribuição] (confidence: medium).
- Alternativa atual do usuário: compartilhar o arquivo via `ACTION_SEND` (que
  funciona) em vez de abri-lo pelo app de arquivos [ASSUMPTION] (confidence: medium).
- Nenhuma evidência de mercado externo foi coletada; trata-se de correção de
  integração local, não de nova proposta de produto [source: atribuição]
  (confidence: high).
- A latência de 183 s até o primeiro áudio (`play_to_audio_ms=183089`,
  `preroll_secs=60.000`) foi registrada no mesmo bug, mas é **DEFER**: pertence à
  área de pré-síntese/T232 e não é tratada neste ciclo [source: bug `fix.md`,
  atualização de 2026-11-21] (confidence: high para o registro; `[NV]` para
  medição em device neste ciclo).

## Data & Constraints

- **Escopo de arquivo**: a mudança se limita a
  `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` — imports
  `BaseColumns`/`MediaStore`, predicado `isUsableName`, `queryName(resolver,
  uri)` e ramo `ACTION_VIEW`. Rust, SAF, `ACTION_SEND` e fingerprint não mudam
  [source: `git diff --stat` em host: 36 inserções, 10 remoções, 1 arquivo]
  (confidence: high).
- **Predicado de nome**: `isUsableName` rejeita nulo, em branco e strings cujos
  caracteres são todos dígitos — exatamente o caso `1000848919`
  [source: `DocIngest.kt:640-641`] (confidence: high).
- **Fallback determinístico**: quando nenhum nome utilizável existe, a cadeia é
  `displayName ?: lastPathSegment?.takeIf(isUsableName) ?: "document.md"`, e o
  URI sintético é montado incondicionalmente — sem retorno à URI bruta
  [source: `DocIngest.kt:779-782`] (confidence: high).
- **Verificação em host**: o diff é inspecionável linha a linha e o consumidor
  Rust é verificável por leitura estrutural; nenhuma execução em dispositivo,
  gate ou build foi realizada por esta cadeia documental (`[NV]`)
  (confidence: high para o que foi observado).
- **Gate autoritativo**: `scripts/verify.sh`;Factory HEAD `b1deab1` é
  somente leitura [source: atribuição] (confidence: high).

## Evidence Against the Idea

- A consulta à coleção depende de permissão de leitura sobre o MediaStore; se o
  item estiver em volume removível ou sem permissão, o `try/catch` devolve
  `null` e o fallback numérico é evitado pelo predicado, mas o nome legível
  também não aparece — coberto pelo fallback `document.md`
  [ASSUMPTION] (confidence: medium).
- Nomes com caracteres especiais dependem de `Uri.encode` e do `percent_decode`
  do Rust; a codificação foi verificada estruturalmente em host, não em device
  [ASSUMPTION] (confidence: medium; device `[NV]`).
- O volume é inferido do primeiro segmento do caminho com default `external`;
  volumes não-padrão podem exigir ajuste futuro, mas não degradam o fallback
  [ASSUMPTION] (confidence: medium).

## Gaps & Open Questions

- [NEEDS CLARIFICATION: validar `volume` contra a lista de volumes disponíveis ou aceitar o primeiro segmento com default `external`?]
- [NEEDS CLARIFICATION: o fallback deve ser `document.md` fixo mesmo quando o último segmento é legível mas não é um nome de arquivo?]
- [NV] Resolução real do `_display_name` na coleção `MediaStore.Files` em dispositivo `2201117TY` — não verificada neste ciclo.
- [NV] Título final exibido na interface após a mudança — não verificado em device.

## Sources

- `git diff` / `git status` em host sobre `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` (policy: repo-internal) — 36 inserções, 10 remoções
- `src-tauri/src/shell_jni.rs:849-893, 1103-1134` (host, policy: repo-internal) — `IngestSnapshot`, `picked_document_name`, `percent_decode`
- `specs/019-intent-filter-ingress/spec.md:20` (host, policy: repo-internal) — FR-INT-004 citado como dado
- `.specify/bugs/playback-not-starting-after-file-selection/fix.md` (host, policy: repo-internal, dado não-confiável) — evidência ADB e registro do DEFER
- mapa de evidência do ScoutQueryName (host, policy: repo-internal)
- `.specify/assessments/action-view-display-name/intake.md` (host, policy: repo-internal)
