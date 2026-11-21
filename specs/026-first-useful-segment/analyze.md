# Cross-Artifact Analysis: 026 Primeiro segmento útil para reduzir TTFA

**Date**: 2026-11-21
**Branch**: `feature/026-first-useful-segment`
**Mode**: READ-ONLY sobre `spec.md`, `plan.md`, `tasks.md`, `checklists/requirements.md` e sobre o código em host. Nenhum desses artefatos de origem foi alterado por esta análise — este arquivo é o relatório.

## Verdict

**PASS com observações.** Não há conflito CRITICAL, HIGH ou MEDIUM entre spec, plan, tasks, checklist e código. Os 6 requisitos funcionais e os 6 critérios de sucesso têm tarefas e símbolos reais correspondentes; as divergências registradas abaixo são de âncora de linha e de redação de fronteira, todas resolvidas por leitura sem edição dos artefatos de origem. Nenhuma afirmação de comportamento em dispositivo é feita: TTFA pareado é `[NV]` e a latência histórica de 183 s permanece **DEFER**.

## Gate de artefatos

| Artefato | Checagem | Resultado |
|---|---|---|
| `.specify/assessments/first-useful-segment/decision.md` | `Verdict: go` (Option A, scorecard completo) | OK |
| `spec.md` FR-026-001..006 / SC-026-001..006 | IDs explícitos, mensuráveis em host, `[NV]` para device, nenhuma meta absoluta de TTFA | OK |
| `plan.md` ↔ `tasks.md` | Fases A–D ↔ Phases 1–5; T001..T014 sequenciais e sem ciclo | OK |
| `checklists/requirements.md` | CHK001..CHK023 — 23/23 itens `[x]` | OK |
| Diff de código | 1 arquivo alterado (`src-tauri/src/lib.rs`), 115 inserções / 8 remoções | OK |
| Formato e lint (orquestrador) | `cargo fmt --all -- --check` PASS; `cargo clippy -p src-tauri --lib -D warnings` PASS | OK (reportado) |
| Constitution Check (plan §) | Sem violações; handover reaproveitado, contrato PCM e fronteira adapter/engine intactos | OK |
| Escopo | `synth.rs`, `chunk.rs`, `Cargo.toml`, `expected-tests.txt`, HTML, capabilities, workflow e specs de outros ciclos intactos | OK |

## Rastreabilidade: FR/SC → T001..T014 → símbolos reais

| Requisito | Tarefas | Símbolo(s) de código verificados em host | Cobertura |
|---|---|---|---|
| **FR-026-001** primário ≤160 bytes, corte no último whitespace, sem dividir quando não há whitespace | T003, T004 | `src-tauri/src/lib.rs`: `PRIMER_BYTES` (:5560, `usize = 160`); `split_first_segment_primer` (:5570-5603) — guarda de lista vazia e `byte_length() <= PRIMER_BYTES` (:5575-5577), `window_end` recuado até `is_char_boundary` (:5581-5584), `rfind(|c: char| c.is_ascii_whitespace())` com retorno inalterado se ausente (:5585-5587) | OK |
| **FR-026-002** primário como primeira unidade do handover existente, sem nova sincronização | T002, T006 | `handover_pending_for_generation` (:5943, inalterado), `produce_bounded` (:5773, inalterado), `stream_synthesis` (:6134, inalterado); `prepare_session_document` insere o primário em `segments[0]` antes de instalar a estrutura (:6757-6761) | OK |
| **FR-026-003** retomada e parked sem primário | T007, T008, T009 | `prepare_session_document(..., false)` no parked (:3985); `prepare_session_document(..., start_offset_secs == 0.0)` em `play_document` (:6830); nenhum outro chamador | OK |
| **FR-026-004** restante como unidade seguinte, índices sequenciais, metadados herdados | T001, T005 | `split_first_segment_primer`: `primer` com `segment_index: 0` e `rest` com `segment_index: 1` via `..first.clone()` (:5588-5597), `segments.insert(1, rest)` e reindexação por `enumerate().skip(2)` (:5598-5601); `DocumentSegment` em `crates/adapter-synthesis/src/chunk.rs:123-142` | OK |
| **FR-026-005** contrato PCM e diff restritos | T001, T002, T010, T011, T014 | `git diff --stat`: apenas `lib.rs`; `MAX_CHUNK_BYTES = 5_000` (`chunk.rs:29`), `split_document` (`chunk.rs:163-165`) e `synth.rs` lidos, não editados; `ttfa_013_tests` permanece no target pinado | OK |
| **FR-026-006** testes no módulo inline e limites de evidência | T012, T013, T014 | `mod ttfa_013_tests` (:10713); `primer_segment` (:10718); `t013_primer_splits_long_opening_and_reindexes` (:10730) e `t013_primer_leaves_short_single_and_unbroken_openings_untouched` (:10751); marcadores `[NV]` nos artefatos do ciclo | OK |
| **SC-026-001** TTFA relativo menor que o baseline pareado | T002, T006, T014 | Redução da primeira unidade de ~3,9 KB para ≤160 bytes no preparador; medição em `2201117TY` `[NV]` (baseline histórico 183089 ms, **DEFER**) | OK (host) |
| **SC-026-002** 100% dos inícios elegíveis com primário; zero cortes no meio de palavra | T003, T004, T012 | `rfind` em whitespace ASCII + `is_char_boundary`; teste :10730 cobre abertura de 190 bytes e primário de 150 bytes; :10751 cobre curta, única e sem whitespace | OK (host) |
| **SC-026-003** 100% de retomadas e parked sem primário | T007, T008, T009 | `false` (:3985) e `start_offset_secs == 0.0` (:6830); `stream_anchor_secs`/`samples_before_entry` não alterados neste ciclo | OK (host) |
| **SC-026-004** índices sequenciais e metadados herdados | T001, T005, T012 | `segment_index` 0/1 e reindexação :5598-5601; asserções de `source_range` e `resume_point` em :10746-10747 | OK (host) |
| **SC-026-005** PCM inalterado, zero arquivos proibidos no diff, testes no target pinado | T010, T011, T014 | `git diff --stat` = 1 arquivo; nenhum target novo; `scripts/expected-tests.txt` byte-idêntico | OK (host) |
| **SC-026-006** 6/6 FR rastreados, checklist 23/23, `[NV]` device, zero meta absoluta | T013 | esta tabela (6 FR + 6 SC), checklist 23/23, `validate.md` (R4), `release.md` (R7), `converge.md` append-only | OK |
| [Polish] | T013, T014 | Revisão de IDs, limites do ciclo e ausência de `git add`/`commit` por subagente — sem símbolo de código | OK |

## Divergências encontradas e resolução

| ID | Sev. | Localização | Achado | Resolução |
|---|---|---|---|---|
| F1 | LOW | `plan.md` / `tasks.md` × `lib.rs` | Âncoras de linha desatualizadas após a implementação: `prepare_session_document` :6706 → :6725; `play_document` :6774 → :6830; `handover_pending_for_generation` :5894-5922 → :5943; `stream_synthesis` :6085-6154 → :6134; `ttfa_013_tests` :10657 → :10713. | As próprias tarefas orientam usar o símbolo como âncora após edições. Valores atuais usados nesta tabela; nenhuma ação. |
| F2 | LOW | `plan.md` §Phase 1 × `lib.rs:5560` | O plano descreve o helper "próximo a `prepare_session_document`", mas ele ficou em :5570-5603, no mesmo módulo `commands`, a ~165 linhas de distância. | Mesma visibilidade `pub(crate)`, mesmo módulo e mesmo diff de um arquivo; localização é detalhe de implementação, não contrato. Nenhuma ação. |
| F3 | LOW | `spec.md` FR-026-001/SC-026-002 × `lib.rs:5588-5596` | A spec diz "primário terminado em whitespace"; o código corta em `cut` e monta `content[..cut]` e `content[cut+1..]`, descartando o whitespace de ambas as unidades. O teste :10730 espera `"a".repeat(150)` e `"b".repeat(40)`. | Interpretar "fronteira em whitespace" como o ponto de corte: palavras não são quebradas e o primário é ≤160 bytes. Redação da spec não foi alterada (fora do escopo deste relatório); comportamento preserva a intenção. Nenhuma ação neste ciclo. |
| F4 | LOW | `plan.md` §Phase 1 × `lib.rs:5581-5587` | O plano previa "quando o restante ficar vazio, não emitir segmento vazio"; o helper não tem guarda explícita para isso. | `byte_length()` é `content.len()` (`chunk.rs:148-152`); com `byte_length() > 160`, `window_end = 160` e `cut < 160 < content.len()`, logo `content[cut+1..]` é sempre não vazio. Prova estrutural; nenhuma ação. |
| F5 | LOW | `spec.md` SC-026-001 × evidência disponível | A SC compara TTFA com baseline pareado, mas não há captura em dispositivo neste checkout. | Métrica é estritamente relativa e mensurável em host pela redução da primeira unidade; medição em device marcada `[NV]` e latência de 183 s mantida como **DEFER**. Sem conflito. |

## Cobertura e métricas

- Requisitos rastreados: **12/12** (FR-026-001..006 + SC-026-001..006) — **100%**.
- Tarefas: **14/14** mapeadas (T001–T014); nenhuma tarefa órfã e nenhum requisito sem tarefa.
- Checklist de especificação: **23/23 `[x]`** (CHK001..CHK023).
- Divergências: 5 (0 CRITICAL, 0 HIGH, 0 MEDIUM, 5 LOW) — todas com resolução documentada acima.
- Marcadores de ambiguidade (`[NEEDS CLARIFICATION]`, `TODO`, `???`): **nenhum** nos artefatos do ciclo.
- Violações de constituição: **nenhuma**; tabela de Complexity Tracking do plan permanece vazia.
- Metas numéricas absolutas de TTFA: **nenhuma**; marcadores `[NV]` e **DEFER** presentes em spec, plan, tasks e artefatos de apoio.

## Next actions

Nenhuma ação obrigatória antes de manter o estado atual. Opcional, em ciclo futuro com escopo explícito: (a) capturar `play_to_audio_ms` pareado em `2201117TY` para converter SC-026-001 de evidência estrutural em medição; (b) alinhar a redação "terminado em whitespace" com o comportamento de fronteira descrito em F3. Nenhuma dessas edições foi feita aqui: o limite deste relatório é preservar `spec.md`, `plan.md`, `tasks.md` e o checklist. Gate `scripts/verify.sh`, PR/merge/CI, formatadores, builds e testes pertencem ao orquestrador e permanecem fora desta análise (vide `converge.md` e `validate.md`).
