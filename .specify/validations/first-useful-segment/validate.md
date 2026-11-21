# Validation (R4): 026 Primeiro segmento útil para reduzir TTFA

- **Date**: 2026-11-21
- **Branch**: `feature/026-first-useful-segment`
- **Spec**: `specs/026-first-useful-segment/spec.md` (FR-026-001..006, SC-026-001..006)
- **Status**: **PASS nos checks documentais e de host realizados**; dispositivo, `verify.sh` completo, builds, CI e PR/merge estão `[NV]`. Este artefato é documental — nenhum gate, build, teste ou formatador foi executado por ele; `cargo fmt`/`cargo clippy` abaixo são resultados reportados pelo orquestrador do ciclo.

---

## 1. Evidência local obtida (host)

| Check | Evidência LOCAL | Resultado |
|---|---|---|
| Diff restrito [FR-026-005] | `git diff --stat` — 1 arquivo (`src-tauri/src/lib.rs`), 115 inserções, 8 remoções | **PASS** |
| Primário e janela [FR-026-001] | `PRIMER_BYTES = 160` (:5560); `split_first_segment_primer` (:5570-5603) recua `window_end` até `is_char_boundary` (:5581-5584) e usa `rfind` de whitespace ASCII (:5585); sem whitespace, devolve a lista inalterada | **PASS (inspeção estrutural)** |
| Herança e índices [FR-026-004] | `..first.clone()` em primário/restante (:5588-5596), `insert(1, rest)` e reindexação `enumerate().skip(2)` (:5598-5601); `source_range`/`resume_point` afirmados no teste :10746-10747 | **PASS (inspeção estrutural + teste no target)** |
| Primeira unidade do handover [FR-026-002] | `segments[0]` é o primário antes de `install_session_structure`; `handover_pending_for_generation` (:5943), `produce_bounded` (:5773) e `stream_synthesis` (:6134) inalterados | **PASS (leitura)** |
| Retomada e parked [FR-026-003] | `prepare_session_document(..., false)` (:3985) e `prepare_session_document(..., start_offset_secs == 0.0)` (:6830); nenhum outro chamador | **PASS (inspeção estrutural)** |
| Contratos preservados [FR-026-005] | `chunk.rs` (`MAX_CHUNK_BYTES:29`, `DocumentSegment:123-142`, `split_document:163-165`), `synth.rs`, `Cargo.toml`, `expected-tests.txt`, HTML, capabilities e workflow ausentes do diff | **PASS (diff)** |
| Testes no target pinado [FR-026-006] | `ttfa_013_tests` (:10713) com 2 casos novos (:10730 e :10751); nenhum target novo | **PASS (leitura)** |
| Qualidade da spec | Sem `[NEEDS CLARIFICATION]`; 7 clarificações; 6 FR e 6 SC rastreáveis; nenhuma meta absoluta de TTFA | **PASS (revisão documental)** |
| Checklist de qualidade | `specs/026-first-useful-segment/checklists/requirements.md` — CHK001..CHK023 | **23/23 `[x]` — PASS** |
| Gate de avaliação | `.specify/assessments/first-useful-segment/decision.md` — `Verdict: go` | **PASS** |
| Análise cruzada [FR-026-006] | `analyze.md` — 12/12 requisitos cobertos, 0 CRITICAL/HIGH/MEDIUM, 5 divergências LOW resolvidas | **PASS** |
| Formato e lint (orquestrador) | `cargo fmt --all -- --check` e `cargo clippy -p src-tauri --lib -D warnings` | **PASS** (reportados; não executados por este artefato) |
| Limites | `[NV]` para dispositivo e **DEFER** para 183 s presentes; nenhum arquivo de outra spec alterado | **PASS** |

---

## 2. Success Criteria — SC-026-001..006

| SC | Critério (spec) | Evidência LOCAL | Resultado |
|---|---|---|---|
| **SC-026-001** | TTFA no início fresco menor que o baseline pareado (183089 ms em 2201117TY), sem meta absoluta | Primeira unidade reduzida de ~3,9 KB para ≤160 bytes no preparador; handover existente a consome primeiro | **PASS em host (estrutural)**; captura pareada em dispositivo é `[NV]` |
| **SC-026-002** | 100% dos inícios elegíveis com primário ≤160 bytes em fronteira de whitespace; zero cortes no meio de palavra; sem whitespace, lista intacta | `rfind` + `is_char_boundary` (:5581-5587); testes :10730 (190 bytes → 150 + 40) e :10751 (curta, unitária, sem whitespace) | **PASS em host** |
| **SC-026-003** | 100% de retomadas e parked com a segmentação original, zero primários adicionais | `false` (:3985) e `start_offset_secs == 0.0` (:6830); T007/T008/T009 | **PASS em host** |
| **SC-026-004** | 100% dos segmentos com `segment_index` sequencial e metadados herdados | Índices 0/1 e reindexação (:5598-5601); asserções em :10746-10747 | **PASS em host** |
| **SC-026-005** | PCM inalterado, zero arquivos proibidos no diff, testes no target pinado | `git diff --stat` = 1 arquivo; nenhum target novo; contrato mono f32 22050 Hz intacto | **PASS em host** |
| **SC-026-006** | 6/6 FR rastreados, checklist 23/23, `[NV]` device, zero meta absoluta | `analyze.md` (12/12 FR+SC), checklist 23/23, marcadores `[NV]`/**DEFER** na cadeia | **PASS** |

> Todas as SCs são mensuráveis em host por inspeção estrutural ou por testes no target pinado e foram satisfeitas nos checks realizados. Nenhuma afirmação de comportamento em dispositivo é reivindicada.

---

## 3. [NV] — Não verificado neste ciclo

- **Dispositivo Android (`2201117TY`)**: TTFA real após o primário e a comparação pareada com o baseline `play_to_audio_ms=183089` — **[NV]**. A latência de ~183 s é dado histórico registrado como **DEFER**, não medição deste ciclo.
- **Custo fixo de inicialização do modelo**: se a parcela dominante do TTFA é a síntese da primeira unidade ou o startup do motor — **[NV]**; premissa registrada em `spec.md §Assumptions`.
- **`verify.sh` completo (gates 1–9)**: não executado por este artefato; exit code pendente de observação pelo orquestrador — **[NV]**.
- **Builds, formatadores e testes**: não executados nesta cadeia documental; `fmt`/`clippy` PASS e os testes em `ttfa_013_tests` são evidências reportadas/estruturais, não uma suíte completa executada aqui — **[NV]** para a suíte completa.
- **CI remoto / PR / merge**: pipelines, status e merge não executados nem observados — **[NV]**.
- **Gate 9 / aparelho**: build/pacote aarch64 (APK/AAB), instalação e qualquer medição de áudio em device — **[NV]**.
- **T232 / worker de pré-síntese**: permanece **DEFER**, sem tarefa neste ciclo.

---

## 4. Limites do ciclo

- Nenhum arquivo foi editado por esta cadeia documental além dos artefatos do ciclo 026: 5 assessments, `spec.md`/`plan.md`/`tasks.md`/`analyze.md`/`converge.md`, checklist, este `validate.md` (R4) e `release.md` (R7).
- `specs/013`, `specs/025`, bugs, `scripts/verify.sh`, `scripts/expected-tests.txt` e `converge.md` de outras specs não foram alterados.
- Nenhum `git add`/`git commit` foi realizado; as mudanças permanecem no working tree do branch, **sem merge**.
- Nenhuma afirmação de gate, build ou teste é feita além dos checks documentais/host efetivamente observados na seção 1 e dos resultados de `fmt`/`clippy` reportados pelo orquestrador.
