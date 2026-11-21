---

description: "Task list for feature implementation — Ciclo 026, primeiro segmento útil para reduzir TTFA"
---

# Tasks: Primeiro segmento útil para reduzir TTFA

**Input**: Design documents from `/specs/026-first-useful-segment/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), checklists/requirements.md

**Tests**: nenhum novo target de teste. Os testes ficam no módulo inline `ttfa_013_tests` do target já pinado (`src-tauri/src/lib.rs:10657`); gates, builds e testes são executados pelo agente principal.

**Organization**: tarefas agrupadas por user story para permitir verificação independente.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: pode rodar em paralelo (seções diferentes, sem dependência de tarefa incompleta)
- **[Story]**: qual user story a tarefa atende (US1, US2, US3, US4)
- IDs são sequenciais T001..T014 e pertencem exclusivamente a este ciclo

## Path Conventions

- Caminhos relativos à raiz do repositório `mdreader`
- Único arquivo de código do ciclo: `src-tauri/src/lib.rs`
- Contratos lidos, NÃO editados: `crates/adapter-synthesis/src/chunk.rs`, `crates/adapter-synthesis/src/synth.rs`, `scripts/verify.sh`, `scripts/expected-tests.txt`

---

## Phase 1: Setup & Foundational (Shared Infrastructure)

**Purpose**: fixar os símbolos de apoio e confirmar que o handover de 013 atende ao primário sem mudança.

- [ ] T001 [P] Confirmar por leitura que `DocumentSegment` (`crates/adapter-synthesis/src/chunk.rs:123-142`) expõe `content`, `source_range`, `boundary_kind`, `segment_index` e `resume_point`, que `byte_length()` conta bytes UTF-8 e que `split_document` (:163-165) aplica o teto de 5000 bytes. Nenhuma edição. [FR-026-004, FR-026-005]
- [ ] T002 [P] Confirmar por leitura que `handover_pending_for_generation` (`src-tauri/src/lib.rs:5894-5922`) enfileira a primeira peça e inicia o backend, e que `stream_synthesis` (:6085-6154) produz até o start — o primário será a primeira unidade, sem nova sincronização. Nenhuma edição. [FR-026-002, FR-026-005]

**Checkpoint**: fundação confirmada; nenhuma user story depende de arquivo novo.

---

## Phase 2: User Story 1 — Primeiro áudio chega antes da síntese integral (Priority: P1) 🎯 MVP

**Goal**: em início fresco, a primeira unidade de síntese é um primário curto, delimitado por whitespace.

**Independent Test**: em host, o helper recebe segmentos com abertura longa e devolve primário ≤160 bytes, restante logo em seguida e lista reindexada.

### Implementation for User Story 1

- [ ] T003 [US1] Definir `PRIMER_BYTES = 160` e o helper `split_first_segment_primer(Vec<DocumentSegment>) -> Vec<DocumentSegment>` em `src-tauri/src/lib.rs` (próximo a `prepare_session_document`), sem novo arquivo e sem tipo público. [FR-026-001]
- [ ] T004 [US1] No helper, devolver a lista inalterada quando ela estiver vazia ou quando `first.byte_length() <= PRIMER_BYTES`; calcular o fim da janela em fronteira de caractere UTF-8 e localizar o último whitespace ASCII dentro da janela, devolvendo a lista inalterada se não houver whitespace — nunca cortar palavra ou caractere. [FR-026-001]
- [ ] T005 [US1] Construir primário e restante com `content` dividido na fronteira de whitespace, herdando `source_range`, `boundary_kind` e `resume_point` do segmento original via atualização de estrutura; substituir o primeiro, inserir o restante na posição 1 e reindexar todos os segmentos subsequentes a partir de 2. [FR-026-004]
- [ ] T006 [US1] Acrescentar o parâmetro `primer: bool` a `prepare_session_document` (`src-tauri/src/lib.rs`), aplicando o helper imediatamente após `split_document` somente quando `primer` for `true`, antes de calcular título, total de unidades, seções e de instalar a estrutura da sessão. [FR-026-001, FR-026-002, FR-026-004]

**Checkpoint**: US1 verificável em host de forma independente pelo helper e pela lista resultante.

---

## Phase 3: User Story 2 — Retomada preserva o comportamento atual (Priority: P1)

**Goal**: primário apenas em `start_offset_secs == 0.0`; parked e retomada com offset continuam com a segmentação original.

**Independent Test**: em host, inspeção dos dois chamadores mostra `false` no parked e a comparação com `0.0` no start; nenhum outro chamador do preparador existe.

### Implementation for User Story 2

- [ ] T007 [US2] No chamador parked (`src-tauri/src/lib.rs:3985`), passar `false` para `prepare_session_document`, preservando o payload de espera e garantindo que nenhuma síntese ou primário seja criado ao estacionar. [FR-026-003]
- [ ] T008 [US2] No chamador de `play_document` (`src-tauri/src/lib.rs:6774`), passar `start_offset_secs == 0.0` como valor de `primer`, preservando `stream_anchor_secs`, `samples_before_entry` e o timeline de retomada para offsets maiores que zero. [FR-026-003]
- [ ] T009 [US2] Confirmar que não existem outros chamadores de `prepare_session_document` e que a assinatura atualizada compila em todos eles; nenhum fallback de primário para retomada. [FR-026-003]

**Checkpoint**: a distinção fresco/retomada está explícita nos dois caminhos; nenhuma mudança em offsets.

---

## Phase 4: User Story 3 — Integridade de segmentos e contrato de áudio (Priority: P2)

**Goal**: contrato PCM inalterado, handover reaproveitado e diff confinado a um arquivo de código.

**Independent Test**: em host, `git diff --stat` mostra apenas `src-tauri/src/lib.rs` entre os arquivos de código; `synth.rs`, `chunk.rs`, `Cargo.toml`, `expected-tests.txt`, HTML, capabilities e workflow ausentes.

### Implementation for User Story 3

- [ ] T010 [P] [US3] Não alterar `handover_pending_for_generation` (:5894-5922), `produce_bounded`, `stream_synthesis` (:6085-6154), `crates/adapter-synthesis/src/chunk.rs`, `crates/adapter-synthesis/src/synth.rs` nem o contrato mono f32 22050 Hz; o primário é apenas a primeira unidade do produtor já existente. [FR-026-002, FR-026-005]
- [ ] T011 [P] [US3] Não criar target de teste nem editar `scripts/expected-tests.txt`, `Cargo.toml`, HTML, capabilities, workflow, `specs/013`, `specs/025`, bugs ou `converge.md` de outras specs. [FR-026-005]

**Checkpoint**: superfície de mudança confinada ao preparador e ao módulo de testes inline.

---

## Phase 5: User Story 4 — Testes no target pinado e limites do ciclo (Priority: P2)

**Goal**: cobrir os quatro comportamentos de FR-026-006 sem novo target e registrar os limites de evidência.

**Independent Test**: em host, os testes aparecem em `ttfa_013_tests`, a tabela de análise cobre 100% dos FRs/SCs e nenhuma meta absoluta de TTFA aparece na cadeia.

### Verification for User Story 4

- [ ] T012 [US4] Acrescentar casos em `ttfa_013_tests` (`src-tauri/src/lib.rs:10657+`) usando um construtor de segmento de teste: (a) abertura longa com whitespace produz primário, restante, índices 0..3 e metadados herdados; (b) abertura curta, lista de segmento único e abertura sem whitespace permanecem intactas. [FR-026-006]
- [ ] T013 [US4] Verificar em host a rastreabilidade de FR-026-001..006 e SC-026-001..006 em `analyze.md`, o checklist 23/23 em `checklists/requirements.md`, a ausência de meta numérica absoluta de TTFA e a presença de `[NV]` para dispositivo. [FR-026-006, Polish]
- [ ] T014 [US4] Revisar o diff do ciclo: apenas `src-tauri/src/lib.rs` entre arquivos de código; nenhum `git add`/`commit`, gate, build, teste ou formatador executado por subagente; `validate.md` (R4) e `release.md` (R7) registram o estado real da evidência e o rollback. [FR-026-005, FR-026-006, Polish]

**Checkpoint**: cadeia documental completa e coerente; limites declarados.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup/Foundational)**: sem dependências — leitura apenas.
- **Phase 2 (US1)**: depende de T001/T002; T003 → T004 → T005 → T006 são sequenciais.
- **Phase 3 (US2)**: depende de T006 (assinatura do preparador); T007 e T008 são independentes entre si.
- **Phase 4 (US3)**: pode rodar em paralelo com a verificação de US2; restringe o diff.
- **Phase 5 (US4)**: depende de todas as fases anteriores.

### User Story Dependencies

- **US1 (P1)**: MVP — helper e preparador.
- **US2 (P1)**: consome a assinatura de US1, mas é testável isoladamente pela leitura dos chamadores.
- **US3 (P2)**: verifica preservação de contratos; não introduz comportamento de produto.
- **US4 (P2)**: verifica as três stories e os limites do ciclo.

### Parallel Opportunities

- T001 e T002 podem rodar em paralelo.
- T007 e T008 podem rodar em paralelo entre si.
- T010 e T011 podem rodar em paralelo entre si.

---

## Implementation Strategy

### MVP First (User Story 1)

1. Concluir Phase 1 (T001, T002).
2. Concluir Phase 2 (T003–T006).
3. **PARAR E VERIFICAR**: primário ≤160 bytes em whitespace, restante preservado, índices sequenciais e metadados herdados.
4. Só então seguir para US2.

### Incremental Delivery

1. Setup + Foundational → símbolos e handover confirmados.
2. US1 → primário em início fresco (MVP).
3. US2 → retomada e parked intactos.
4. US3 → contratos e diff restritos.
5. US4 → testes no target pinado, rastreabilidade e limites.

### Never Do

- Criar novo target de teste ou editar `scripts/expected-tests.txt`, `Cargo.toml`, HTML, capabilities ou workflow.
- Editar `synth.rs`, `chunk.rs`, `handover_pending_for_generation` ou `stream_synthesis`.
- Aplicar primário em retomada (`start_offset_secs > 0.0`) ou no estacionamento a frio.
- Cortar palavra ou caractere UTF-8 para atingir 160 bytes.
- Definir meta numérica absoluta de TTFA ou reivindicar medição em dispositivo (`[NV]`).
- Executar `git add`/`git commit` ou gates/builds/testes como subagente.

---

## Notes

- **[P]** = seções diferentes, sem dependências entre si.
- **[Story]** rastreia a tarefa até a user story correspondente em `spec.md`.
- Cada tarefa é específica o bastante para ser executada sem contexto adicional; os números de linha são os do checkout no momento da escrita — use o símbolo como âncora após edições.
- Nenhuma tarefa deste ciclo endereça T232 ou a captura pareada de TTFA em dispositivo; ambos são `[NV]`/DEFER fora de escopo.
- Gates, testes e formatadores NÃO são executados por subagentes; a verificação final cabe ao agente principal.
