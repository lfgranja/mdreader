# Cross-Artifact Analysis: 027 Exaustão da produção não finaliza o documento

**Date**: 2026-11-21
**Branch**: `feature/027-production-exhaustion`
**Mode**: READ-ONLY sobre `spec.md`, `plan.md`, `tasks.md`, `checklists/requirements.md` e sobre o código em host. Nenhum desses artefatos de origem foi alterado por esta análise — este arquivo é o relatório. A análise é **pré-implementação**: mapeia requisitos e tarefas aos símbolos reais do checkout atual; não afirma que a correção de código já foi aplicada.

## Verdict

**PASS com observações (pré-implementação)**. Não há conflito CRITICAL, HIGH ou MEDIUM entre spec, plan, tasks, checklist e código. Os 6 requisitos funcionais e os 6 critérios de sucesso têm tarefas e símbolos reais correspondentes; as divergências registradas abaixo são de âncora de linha e de redação de fronteira, todas resolvidas por leitura sem edição dos artefatos de origem. Nenhuma afirmação de comportamento em dispositivo é feita: a reprodução limpa do bug `playback-stall-at-end-of-document` (`action=complete` + `session_end` após >220 s em `2201117TY`) é `[NV]`.

## Gate de artefatos

| Artefato | Checagem | Resultado |
|---|---|---|
| `.specify/assessments/production-exhaustion/decision.md` | `Verdict: go`, origem declarada no bug `playback-stall-at-end-of-document`, scorecard completo | OK |
| `spec.md` FR-027-001..006 / SC-027-001..006 | IDs explícitos, mensuráveis em host, `[NV]` para device, nenhum enum novo, nenhum `complete` no produtor | OK |
| `plan.md` ↔ `tasks.md` | Fases A–D ↔ Phases 1–5; T001..T014 sequenciais e sem ciclo | OK |
| `checklists/requirements.md` | CHK001..CHK023 — 23/23 itens `[x]` | OK |
| Código em host (símbolos de destino) | `produce_bounded`, `finish_production_if_current`, `production_running`, `playback_watch_tick`, `ttfa_013_tests` lidos e ancorados | OK |
| Diff de código | A aplicar pelo agente principal; não observado por esta análise | `[NV]` |
| Formato, lint, gates, builds e testes (orquestrador) | Não executados por esta cadeia documental | `[NV]` |
| Constitution Check (plan §) | Sem violações; gate de drenagem, geração, helper existente e contrato PCM preservados | OK |
| Escopo | Nenhum enum novo; specs/bugs de outros ciclos, `converge.md` alheio e scripts intactos nesta cadeia | OK |

## Rastreabilidade: FR/SC → T001..T014 → símbolos reais

| Requisito | Tarefas | Símbolo(s) de código verificados em host (estado atual) | Cobertura |
|---|---|---|---|
| **FR-027-001** exaustão detectada no topo do loop, antes do `break`/`return`; `done` pós-commit preservado | T003, T004, T005 | `src-tauri/src/lib.rs`: `produce_bounded` (:5773-5863); início do `while` (:5782); `queue_room` (:5783), `has_pending` (:5784-5792), `break` de backpressure (:5803-5805); `get(next_unit)` → `None` (:5817-5819); `done` pós-commit (:5847-5860) | OK (mapeado) |
| **FR-027-002** guard solto + `finish_production_if_current`, generation-safe, sem deadlock | T001, T002, T004, T006, T007, T008 | `guard.as_ref().is_some_and(|p| p.generation == generation)` em :6009-6013, `guard.take()` :6010, `install_session_timeline` :6016; condição de exaustão planejada em bloco curto que devolve `bool`; `Production` com `generation`/`next_unit`/`pending`/`started` em :11026-11037 | OK (mapeado) |
| **FR-027-003** nenhum `complete` antecipado; gate de drenagem intacto | T002, T008, T009 | `playback_watch_tick` (:2628-2692), gate `drained && !commands::production_running(state)` (:2669), `signal_document_end` (:3215-3226), `dispatch_document_end` (:5533-5555), `is_drained` em `crates/player/src/backend.rs:778-780` | OK (mapeado) |
| **FR-027-004** sem enum novo; produtor, handover, backpressure, primário e PCM intactos | T001, T010, T011 | `production_running` (:6360-6365), `advance_production` (:6305-6327), `handover_pending_for_generation` (:5943-5971), `queue_room` (:6331-6341), `PRIMER_BYTES`/`split_first_segment_primer` (:5557+), `DocumentSegment` em `crates/adapter-synthesis/src/chunk.rs:123-142` | OK (mapeado) |
| **FR-027-005** teste no target pinado com quatro comportamentos | T012, T013 | `mod ttfa_013_tests`; `t007_produce_bounded_commits_first_unit_without_draining_all_segments` (:11007-11061) com construtor de `Production` (:11009-11037), hook de synth (:11039) e asserções de `next_unit`/`total_samples` (:11043-11059) | OK (mapeado) |
| **FR-027-006** diff restrito, limites e `[NV]` | T011, T013, T014 | `scripts/verify.sh` (gate autoritativo, não editado), `scripts/expected-tests.txt` (lista canônica, não editada); `specs/013`, `specs/026` e bug de origem como contexto | OK (mapeado) |
| **SC-027-001** 100% dos estados exauridos com geração atual → `production == None` e timeline instalada | T003, T004, T012 | Ramo `get() == None` (:5817-5819) hoje faz `return Ok(())` sem finalizar (baseline 0%); helper :6004-6018 instala a timeline medida | OK (mapeado) |
| **SC-027-002** 100% de geração divergente/lock envenenado preserva a produção; zero deadlock | T006, T007, T012 | `production_generation_matches` (:5708-5710), `production_running_for_generation` (:5716-5726), checagem de geração no helper (:6009-6013), ramos de lock envenenado (:5785-5787, :6006-6008) | OK (mapeado) |
| **SC-027-003** zero `complete` no produtor; gate de drenagem único | T002, T008, T009, T013 | :2669, :3215, :5533; nenhuma chamada de conclusão em :5773-5863 no estado atual | OK (mapeado) |
| **SC-027-004** comportamentos existentes preservados, zero enum e zero mudança em `production_running`/`advance_production` | T010, T011 | `t007` (:11007-11061), `advance_production` (:6305-6327), `production_running` (:6360-6365), contrato PCM em `crates/adapter-synthesis/src/synth.rs` | OK (mapeado) |
| **SC-027-005** 100% dos testes no target pinado, zero target novo, `expected-tests.txt` intacto, diff só em `lib.rs` | T011, T012, T014 | `ttfa_013_tests` (:11007+); `scripts/expected-tests.txt` e `Cargo.toml` não editados por esta cadeia | OK (mapeado) |
| **SC-027-006** 12/12 rastreados, checklist 23/23, `go`, sem ambiguidade, `[NV]` device | T013, T014 | `analyze.md` (esta tabela), `checklists/requirements.md` CHK001..CHK023, `decision.md`, `validate.md` (R4), `release.md` (R7), `converge.md` append-only | OK |
| [Polish] | T013, T014 | Revisão de IDs, limites do ciclo e ausência de `git add`/`commit` por subagente — sem símbolo de código | OK |

## Divergências encontradas e resolução

| ID | Sev. | Localização | Achado | Resolução |
|---|---|---|---|---|
| F1 | LOW | `plan.md` / `tasks.md` × `lib.rs` | Âncoras de linha citam o checkout atual; a implementação pode deslocar `produce_bounded`, `finish_production_if_current` e `ttfa_013_tests`. | As próprias tarefas orientam usar o símbolo como âncora após edições. Nenhuma ação. |
| F2 | LOW | `spec.md` FR-027-001 × `lib.rs:5817-5819` | A spec descreve a exaustão no topo do loop, mas o código atual só a reconhece implicitamente no `else` de `get()`, e mesmo assim retorna sem finalizar. | É exatamente o defeito a corrigir, não uma divergência de requisito: o mapeamento registra o estado atual (baseline 0%). Nenhuma ação nesta análise. |
| F3 | LOW | `spec.md` FR-027-001 × `lib.rs:5803-5805` | O `break` de backpressure (`production_started && room == 0`) pode ocorrer com a produção exaurida antes do `get()`. | Justificativa da checagem no topo do loop (Option A em `concept.md`); tratar apenas o `else` do `get()` seria incompleto. Nenhuma ação. |
| F4 | LOW | `spec.md` FR-027-002 × `finish_production_if_current` | O helper chama `install_session_timeline` (:6016), que toma o lock de sessão; chamá-lo com o guard de `state.production` retido seria reentrância. | A checagem deve devolver `bool` e soltar o guard antes da chamada; requisito e plano já explicítos. Nenhuma ação. |
| F5 | LOW | `spec.md` SC-027-003 × evidência disponível | A SC exige `action=complete` + `session_end` como resultado final, mas não há reprodução limpa com diag zerado neste checkout. | A medição em dispositivo é `[NV]`; a SC é verificável em host pela ausência de `complete` no produtor e pela manutenção do gate de drenagem. Sem conflito. |

## Cobertura e métricas

- Requisitos rastreados: **12/12** (FR-027-001..006 + SC-027-001..006) — **100%**.
- Tarefas: **14/14** mapeadas (T001–T014); nenhuma tarefa órfã e nenhum requisito sem tarefa.
- Checklist de especificação: **23/23 `[x]`** (CHK001..CHK023).
- Divergências: 5 (0 CRITICAL, 0 HIGH, 0 MEDIUM, 5 LOW) — todas com resolução documentada acima.
- Marcadores de ambiguidade (`[NEEDS CLARIFICATION]`, `TODO`, `???`): **nenhum** nos artefatos do ciclo.
- Violações de constituição: **nenhuma**; tabela de Complexity Tracking do plan permanece vazia.
- Enum novo: **nenhum**; `action=complete`/`session_end` em dispositivo: **`[NV]`**.
- Implementação de código, gates, builds, testes e formatadores: **`[NV]`** nesta análise (pré-implementação; competência do orquestrador).

## Next actions

Nenhuma ação obrigatória antes de manter o estado atual. Opcional, em ciclo futuro com escopo explícito: (a) reprodução limpa em `2201117TY` (diag zerado, ACTION_VIEW com `FLAG_GRANT_READ_URI_PERMISSION`, observação >220 s) para confirmar `action=complete` + `session_end` e fechar o bug de origem; (b) A/B com/sem primário para descartar contribuição do ciclo 026. Nenhuma dessas ações foi executada aqui: o limite deste relatório é preservar `spec.md`, `plan.md`, `tasks.md` e o checklist. Gate `scripts/verify.sh`, PR/merge/CI, formatadores, builds e testes pertencem ao orquestrador e permanecem fora desta análise (vide `converge.md` e `validate.md`).
