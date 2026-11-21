---

description: "Task list for feature implementation — Ciclo 027, exaustão da produção não finaliza o documento"
---

# Tasks: Exaustão da produção não finaliza o documento

**Input**: Design documents from `/specs/027-production-exhaustion/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), checklists/requirements.md

**Tests**: nenhum novo target de teste. O caso de regressão fica no módulo inline `ttfa_013_tests` do target já pinado (`src-tauri/src/lib.rs`, ao lado de `t007_produce_bounded_commits_first_unit_without_draining_all_segments` :11007-11061); gates, builds e testes são executados pelo agente principal.

**Organization**: tarefas agrupadas por user story para permitir verificação independente.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: pode rodar em paralelo (seções diferentes, sem dependência de tarefa incompleta)
- **[Story]**: qual user story a tarefa atende (US1, US2, US3, US4)
- IDs são sequenciais T001..T014 e pertencem exclusivamente a este ciclo

## Path Conventions

- Caminhos relativos à raiz do repositório `mdreader`
- Único arquivo de código do ciclo: `src-tauri/src/lib.rs`
- Caminhos lidos, NÃO editados: `crates/adapter-synthesis/src/chunk.rs`, `crates/adapter-synthesis/src/synth.rs`, `scripts/verify.sh`, `scripts/expected-tests.txt`, `.specify/bugs/playback-stall-at-end-of-document/`

---

## Phase 1: Setup & Foundational (Shared Infrastructure)

**Purpose**: fixar os símbolos de apoio e confirmar que o caminho de fim já existente atende à correção sem mudança.

- [ ] T001 [P] Confirmar por leitura que `Production` expõe `generation`, `segments`, `next_unit`, `pending`, `total_samples` e `started`, e que `production_running` (`src-tauri/src/lib.rs:6360-6365`) é `guard.is_some()`. Nenhuma edição. [FR-027-002, FR-027-004]
- [ ] T002 [P] Confirmar por leitura que `finish_production_if_current` (:6004-6018) só faz `guard.take()` quando `p.generation == generation`, instala a timeline medida via `install_session_timeline` (:6473-6492) e que `playback_watch_tick` (:2669) é o único gate de fim (`drained && !production_running`). Nenhuma edição. [FR-027-002, FR-027-003]

**Checkpoint**: fundação confirmada; nenhuma user story depende de arquivo novo.

---

## Phase 2: User Story 1 — Fim de documento conclui quando a produção se esgota (Priority: P1) 🎯 MVP

**Goal**: exaustão detectada antes de qualquer `return`/`break`, com finalização pelo helper existente.

**Independent Test**: em host, uma `Production` com `next_unit == len()`, `pending` vazio e geração atual passa a `None` após um único `produce_bounded`.

### Implementation for User Story 1

- [ ] T003 [US1] No topo do `while` de `produce_bounded` (`src-tauri/src/lib.rs:5782`), antes de `queue_room`, do ramo `has_pending` (:5784-5802) e do `break` de backpressure (:5803-5805), introduzir uma checagem de exaustão em bloco curto que leia `state.production` sob lock e calcule `generation == p.generation && p.next_unit >= p.segments.len() && p.pending.is_empty()`. Nenhum enum novo. [FR-027-001, FR-027-004]
- [ ] T004 [US1] Se exaurida, soltar o `MutexGuard` no fim do bloco, chamar `finish_production_if_current(state, generation)` e dar `break`; se o lock falhar, retornar `Ok(())` sem finalizar, como os demais ramos de falha de lock. Não tocar no corpo incremental existente (handover, `get()`, síntese, commit e checagem de `done`). [FR-027-001, FR-027-002]
- [ ] T005 [US1] Preservar integralmente a checagem de `done` pós-commit (:5847-5860) para a última unidade produzida no mesmo tick; confirmar por leitura que a idempotência do `take()` no helper evita duplo efeito quando a nova checagem e a antiga coincidem. [FR-027-001, FR-027-002]

**Checkpoint**: US1 verificável em host pela checagem no topo do loop e pela ausência de retorno silencioso na exaustão.

---

## Phase 3: User Story 2 — Finalização generation-safe e sem deadlock (Priority: P1)

**Goal**: nenhuma geração obsoleta finalizada, nenhum guard retido na chamada ao helper.

**Independent Test**: em host, exaustão com geração divergente preserva a produção; chamada ao helper ocorre após a soltura do guard.

### Implementation for User Story 2

- [ ] T006 [US2] Incluir a condição de geração na checagem de exaustão (`p.generation == generation`) e confirmar que `finish_production_if_current` também a verifica antes do `take()` (:6009-6013); nenhuma finalização para geração divergente ou `generation == 0`. [FR-027-002]
- [ ] T007 [US2] Garantir que o `MutexGuard` de `state.production` não seja mantido através da chamada ao helper: a checagem devolve apenas um `bool`, o helper é chamado fora do bloco que tomou o lock; lock envenenado resulta em retorno sem finalização. [FR-027-002]
- [ ] T008 [US2] Confirmar que a condição NÃO exige `started == true`: exaustão sem start (documento vazio ou falha anterior) também libera o gate, mantendo a conclusão subordinada à drenagem do backend. [FR-027-002, FR-027-003]

**Checkpoint**: finalização restrita à geração corrente, sem deadlock e sem dependência de `started`.

---

## Phase 4: User Story 3 — Sem `complete` antecipado e sem enum novo (Priority: P2)

**Goal**: fronteiras preservadas — o produtor apenas libera o gate, a conclusão permanece no watch.

**Independent Test**: em host, `git diff` mostra apenas `src-tauri/src/lib.rs`, sem enum novo e sem chamada de `complete` em `produce_bounded`.

### Implementation for User Story 3

- [ ] T009 [US3] Não introduzir chamada a `signal_document_end` (:3215), `dispatch_document_end` (:5533) ou `ShellVerb::Complete` em `produce_bounded`; o gate `drained && !production_running` (:2669) continua decidindo `action=complete`/`session_end`. [FR-027-003]
- [ ] T010 [P] [US3] Não alterar `production_running` (:6360-6365), `advance_production` (:6305-6327), `handover_pending_for_generation` (:5943-5971), `commit_produced_unit`, `queue_room` (:6331-6341), `synth.rs`, `chunk.rs` nem o contrato PCM mono `f32` @ 22050 Hz; o primário de 026 e a segmentação permanecem intactos. [FR-027-004]
- [ ] T011 [P] [US3] Não criar enum, variante ou tipo público novo, não criar target de teste e não editar `scripts/expected-tests.txt`, `Cargo.toml`, HTML, capabilities, workflow, specs/bugs de outros ciclos ou `converge.md` alheio. [FR-027-004, FR-027-006]

**Checkpoint**: superfície de mudança confinada à checagem de exaustão no produtor.

---

## Phase 5: User Story 4 — Testes no target pinado e limites do ciclo (Priority: P2)

**Goal**: cobrir os quatro comportamentos de FR-027-005 sem novo target e registrar os limites de evidência.

**Independent Test**: em host, o teste aparece em `ttfa_013_tests`, a tabela de análise cobre 100% dos FRs/SCs e nenhuma afirmação de dispositivo é feita sem `[NV]`.

### Verification for User Story 4

- [ ] T012 [US4] Acrescentar um caso em `ttfa_013_tests` (`src-tauri/src/lib.rs`, próximo a :11007-11061) reaproveitando o construtor de `Production` do teste `t007`: (a) `next_unit == segments.len()`, `pending` vazio, `started = true`, geração atual → `production == None` após `produce_bounded`; (b) geração divergente → produção preservada; (c) `pending` com uma peça → não finaliza. O teste NÃO deve instalar hooks de synth para o caso (a). [FR-027-005]
- [ ] T013 [US4] Verificar em host a rastreabilidade de FR-027-001..006 e SC-027-001..006 em `analyze.md`, o checklist 23/23 em `checklists/requirements.md`, a ausência de enum novo e de chamada de `complete` no produtor, e a presença de `[NV]` para dispositivo. [FR-027-005, FR-027-006, Polish]
- [ ] T014 [US4] Revisar o diff do ciclo: apenas `src-tauri/src/lib.rs` entre arquivos de código; nenhum `git add`/`commit`, gate, build, teste ou formatador executado por subagente; `validate.md` (R4) e `release.md` (R7) registram o estado real da evidência e o rollback. [FR-027-006, Polish]

**Checkpoint**: cadeia documental completa e coerente; limites declarados.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup/Foundational)**: sem dependências — leitura apenas.
- **Phase 2 (US1)**: depende de T001/T002; T003 → T004 → T005 são sequenciais.
- **Phase 3 (US2)**: depende de T003/T004; T006, T007 e T008 são independentes entre si.
- **Phase 4 (US3)**: pode rodar em paralelo com a verificação de US2; restringe o diff.
- **Phase 5 (US4)**: depende de todas as fases anteriores.

### User Story Dependencies

- **US1 (P1)**: MVP — checagem de exaustão e chamada ao helper.
- **US2 (P1)**: consome a checagem de US1, mas é verificável isoladamente pela leitura dos guardas.
- **US3 (P2)**: verifica preservação de fronteiras; não introduz comportamento de produto.
- **US4 (P2)**: verifica as três stories e os limites do ciclo.

### Parallel Opportunities

- T001 e T002 podem rodar em paralelo.
- T006, T007 e T008 podem rodar em paralelo entre si.
- T010 e T011 podem rodar em paralelo entre si.

---

## Implementation Strategy

### MVP First (User Story 1)

1. Concluir Phase 1 (T001, T002).
2. Concluir Phase 2 (T003–T005).
3. **PARAR E VERIFICAR**: exaustão detectada antes do `break`, guard solto, helper chamado, `production == None`, nenhum `complete` no produtor.
4. Só então seguir para US2.

### Incremental Delivery

1. Setup + Foundational → símbolos e gate de fim confirmados.
2. US1 → exaustão finaliza a produção (MVP).
3. US2 → generation-safety e ausência de deadlock.
4. US3 → fronteiras e diff restritos.
5. US4 → teste no target pinado, rastreabilidade e limites.

### Never Do

- Chamar `signal_document_end`, `dispatch_document_end` ou `ShellVerb::Complete` a partir de `produce_bounded`.
- Criar enum novo, mudar `production_running` ou esconder a exaustão no watch.
- Alterar handover, backpressure, primário de 026, `synth.rs`, `chunk.rs`, `Cargo.toml`, HTML, capabilities, workflow ou `expected-tests.txt`.
- Manter o `MutexGuard` de `state.production` vivo ao chamar `finish_production_if_current`.
- Reivindicar `action=complete`/`session_end` em dispositivo sem captura (`[NV]`).
- Executar `git add`/`git commit` ou gates/builds/testes como subagente.

---

## Notes

- **[P]** = seções diferentes, sem dependências entre si.
- **[Story]** rastreia a tarefa até a user story correspondente em `spec.md`.
- Cada tarefa é específica o bastante para ser executada sem contexto adicional; os números de linha são os do checkout no momento da escrita — use o símbolo como âncora após edições.
- Nenhuma tarefa deste ciclo reexecuta o smoke em `2201117TY` nem altera o bug de origem; a reprodução limpa com diag zerado é dívida `[NV]` fora de escopo.
- Gates, testes e formatadores NÃO são executados por subagentes; a verificação final cabe ao agente principal.
