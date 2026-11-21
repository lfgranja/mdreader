---

description: "Task list for feature implementation — Ciclo 025, nome de exibição no ACTION_VIEW via MediaStore"
---

# Tasks: Nome de exibição no ACTION_VIEW via MediaStore

**Input**: Design documents from `/specs/025-action-view-display-name/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), checklists/requirements.md

**Tests**: nenhum novo target de teste. A evidência exigida por FR-025-005 é inspeção estrutural em host do diff e do consumidor Rust; gates, builds e testes são executados pelo agente principal.

**Organization**: tarefas agrupadas por user story para permitir verificação independente.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: pode rodar em paralelo (seções diferentes, sem dependência de tarefa incompleta)
- **[Story]**: qual user story a tarefa atende (US1, US2, US3, US4)
- IDs são sequenciais T001..T012 e pertencem exclusivamente a este ciclo

## Path Conventions

- Caminhos relativos à raiz do repositório `mdreader`
- Arquivo de código do ciclo: `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt`
- Consumidor verificado por leitura (NÃO editado): `src-tauri/src/shell_jni.rs`

---

## Phase 1: Setup & Foundational (Shared Infrastructure)

**Purpose**: fixar os símbolos de apoio e confirmar que o consumidor Rust já aceita a URI sintética.

- [ ] T001 [P] Adicionar os imports `android.provider.BaseColumns` e `android.provider.MediaStore` em `DocIngest.kt` (seção de imports, linhas 9 e 11) e garantir o predicado único `isUsableName` (:640-641), que rejeita nulo, em branco e somente dígitos. [FR-025-003]
- [ ] T002 [P] Confirmar por leitura que `picked_document_name` (`src-tauri/src/shell_jni.rs:1103-1134`) isola o último segmento da URI original, aplica `percent_decode` e acrescenta a extensão do snapshot — logo `content://display_name/<nome codificado>` já produz o título sem mudança em Rust. [FR-025-002]

**Checkpoint**: fundação pronta — nenhuma user story depende de arquivo novo.

---

## Phase 2: User Story 1 — Nome correto ao abrir pelo gerenciador de arquivos (Priority: P1) 🎯 MVP

**Goal**: URIs de item do MediaStore passam a produzir o `_display_name` da coleção como nome de exibição.

**Independent Test**: em host, inspeção estrutural mostra a consulta direta seguida da consulta à coleção `MediaStore.Files` com `_id=?`, restrita a URIs `content://` com authority do MediaStore e id numérico.

### Implementation for User Story 1

- [ ] T003 [US1] Em `queryName(resolver, uri)` (`DocIngest.kt:643-650`), manter a consulta direta a `OpenableColumns.DISPLAY_NAME` com `try/catch` e devolver o valor imediatamente quando `isUsableName` o aceitar. [FR-025-001]
- [ ] T004 [US1] Quando a consulta direta falhar, verificar `uri.scheme == ContentResolver.SCHEME_CONTENT` e `uri.authority == MediaStore.AUTHORITY`, extrair `id = uri.lastPathSegment?.toLongOrNull()` e `volume = uri.pathSegments.firstOrNull() ?: "external"` e consultar `MediaStore.Files.getContentUri(volume)` com projeção `DISPLAY_NAME`, selection `"${BaseColumns._ID}=?"` e argumento `id.toString()` (`DocIngest.kt:652-665`). [FR-025-001]
- [ ] T005 [US1] Aplicar `isUsableName` ao resultado da coleção antes de devolvê-lo (`DocIngest.kt:669`) e garantir que URIs de outro provedor ou sem id numérico retornem `null` sem consulta adicional (:655-656). Nenhum nome somente dígitos deve sobreviver. [FR-025-001, FR-025-003]

**Checkpoint**: US1 verificável em host de forma independente.

---

## Phase 3: User Story 2 — URI sintética estável no ACTION_VIEW (Priority: P1)

**Goal**: o ramo `ACTION_VIEW` sempre entrega `content://display_name/<nome codificado>`, com fallback determinístico e sem retorno à URI bruta.

**Independent Test**: em host, inspeção do ramo mostra uma única construção de URI sintética, aplicada em todos os caminhos de saída, e ausência de `uri.toString()` como fallback.

### Implementation for User Story 2

- [ ] T006 [US2] No ramo `Intent.ACTION_VIEW` (`DocIngest.kt:779-781`), escolher o nome pela cascata `displayName ?: uri.lastPathSegment?.takeIf { isUsableName(it) } ?: "document.md"`, garantindo que o identificador numérico nunca seja usado como nome. [FR-025-002, FR-025-003]
- [ ] T007 [US2] Construir `effectiveUri = "content://display_name/${Uri.encode(name)}"` incondicionalmente (`DocIngest.kt:782`) e entregá-la no `Triple` ao JNI (:783). Remover o antigo fallback que devolvia `uri.toString()` quando o nome era nulo ou vazio. [FR-025-002]
- [ ] T008 [US2] Manter a inferência de extensão a partir de `displayName` (`md`, `markdown`, `txt`) com fallback para `resolveFormat` e `md` (:771-772), e a sequência ler bytes → escolher nome → montar URI → JNI (:773-783), sem alterar a leitura do documento. [FR-025-002, FR-025-004]

**Checkpoint**: US1 e US2 funcionam independentemente; a URI entregue ao Rust contém sempre um nome utilizável.

---

## Phase 4: User Story 3 — Preservação dos caminhos existentes (Priority: P2)

**Goal**: SAF/diretório, `ACTION_SEND`, snapshot, BLAKE3 e assinatura JNI permanecem idênticos; o diff é localizado.

**Independent Test**: em host, a enumeração do diff mostra um único arquivo alterado nas quatro seções permitidas, e os símbolos preservados são lidos sem alteração.

### Implementation for User Story 3

- [ ] T009 [P] [US3] Não alterar `queryName(resolver, treeUri, docId)` (`DocIngest.kt:634-638`), que usa `DocumentsContract.buildDocumentUriUsingTree` e a coluna `COLUMN_DISPLAY_NAME` para o SAF/diretório. [FR-025-004]
- [ ] T010 [P] [US3] Não alterar o ramo `Intent.ACTION_SEND` (`DocIngest.kt:785+`) nem outros chamadores da ponte JNI (por exemplo `DebugIngestHook.kt`), preservando `resolveFormat`, a URI bruta e o comportamento de compartilhamento. [FR-025-004]
- [ ] T011 [US3] Não alterar `blake3_256`, `writeSnapshot`, `trimSnapshots` e a chamada `onDocumentIngested` (`DocIngest.kt:734-751`) nem a assinatura `Java_app_mdreader_DocIngest_onDocumentIngested` (`src-tauri/src/shell_jni.rs:849-893`); o fingerprint continua independente do nome exibido. [FR-025-004]

**Checkpoint**: a superfície de mudança está confinada ao resolvedor de nome da URI e ao ramo `ACTION_VIEW`.

---

## Phase 5: User Story 4 — Evidência em host e limites do ciclo (Priority: P2)

**Goal**: rastreabilidade completa, marcação explícita do que não foi verificado e registro do DEFER.

**Independent Test**: em host, a tabela de análise cobre 100% dos FRs, todos os marcadores `[NV]` e `DEFER` estão presentes e nenhum arquivo fora do escopo aparece no diff.

### Verification for User Story 4

- [ ] T012 [US4] Verificar em host: (a) `git diff --stat` aponta apenas `DocIngest.kt`; (b) a consulta à coleção, o predicado e a URI sintética estão presentes; (c) não há `uri.toString()` no ramo `ACTION_VIEW`; (d) `picked_document_name` decodifica o último segmento; (e) todos os FRs estão rastreados em `analyze.md`; (f) dispositivo está `[NV]` e a latência de 183 s está DEFER. Revisar ainda que os IDs são T001..T012 e que nenhum arquivo fora do escopo foi tocado. [FR-025-005, Polish]

**Checkpoint**: cadeia documental completa e coerente; limites declarados.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup/Foundational)**: sem dependências — pode começar imediatamente.
- **Phase 2 (US1)**: depende de T001 (imports e predicado).
- **Phase 3 (US2)**: depende de T003–T005 (nome resolvido); T006–T008 são sequenciais dentro do ramo.
- **Phase 4 (US3)**: depende de T007 para comprovar que nada além do ramo `ACTION_VIEW` mudou; T009/T010 podem rodar em paralelo.
- **Phase 5 (US4)**: depende de todas as fases anteriores.

### User Story Dependencies

- **US1 (P1)**: depende apenas da Phase 1.
- **US2 (P1)**: consome o nome produzido por US1, mas é testável isoladamente pela inspeção do ramo.
- **US3 (P2)**: verifica a preservação dos demais caminhos; não introduz comportamento de produto.
- **US4 (P2)**: verifica as três stories e os limites do ciclo.

### Parallel Opportunities

- T001 e T002 podem rodar em paralelo.
- T009 e T010 podem rodar em paralelo entre si.

---

## Implementation Strategy

### MVP First (User Story 1)

1. Concluir Phase 1 (T001, T002).
2. Concluir Phase 2 (T003–T005).
3. **PARAR E VERIFICAR**: a consulta à coleção está presente e filtrada; nenhum número é aceito como nome.
4. Só então seguir para US2.

### Incremental Delivery

1. Setup + Foundational → imports, predicado e consumidor confirmado.
2. US1 → nome resolvido a partir da coleção MediaStore (MVP).
3. US2 → URI sintética estável.
4. US3 → caminhos existentes preservados.
5. US4 → evidência em host e limites declarados.

### Never Do

- Criar novo target de teste ou editar `scripts/expected-tests.txt`.
- Editar Rust, `specs/019`, `specs/009`, bugs ou `converge.md` de outra spec.
- Reintroduzir `uri.toString()` como fallback de nome no `ACTION_VIEW`.
- Aceitar identificador numérico como nome, em qualquer dos três candidatos.
- Alterar `blake3_256`, `writeSnapshot` ou a assinatura JNI.
- Reivindicar verificação em dispositivo: toda evidência de device é `[NV]`.
- Executar `git add`/`git commit` ou gates/builds/testes como subagente.

---

## Notes

- **[P]** = seções diferentes, sem dependências entre si.
- **[Story]** rastreia a tarefa até a user story correspondente em `spec.md`.
- Cada tarefa é específica o bastante para ser executada sem contexto adicional; os números de linha são os do diff verificado em host e podem derivar após edições — use o símbolo como âncora.
- O item de latência de 183 s (pré-roll 60 s) está **DEFER** e não possui tarefa neste ciclo.
- Gates, testes e formatadores NÃO são executados por subagentes; a verificação final cabe ao agente principal.
