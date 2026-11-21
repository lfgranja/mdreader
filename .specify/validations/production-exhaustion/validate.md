# Validation (R4): 027 Exaustão da produção não finaliza o documento

- **Date**: 2026-11-21
- **Branch**: `feature/027-production-exhaustion`
- **Spec**: `specs/027-production-exhaustion/spec.md` (FR-027-001..006, SC-027-001..006)
- **Status**: **PASS nos checks documentais e de leitura de host realizados**; implementação de código, `verify.sh` completo, builds, testes, CI, PR/merge e dispositivo estão `[NV]`. Este artefato é documental — nenhum gate, build, teste ou formatador foi executado por ele.

---

## 1. Evidência local obtida (host)

| Check | Evidência LOCAL | Resultado |
|---|---|---|
| Gate de avaliação | `.specify/assessments/production-exhaustion/decision.md` — `Verdict: go`, origem no bug `playback-stall-at-end-of-document` | **PASS** |
| Causa localizada [FR-027-001] | `produce_bounded` (:5773-5863); `get(next_unit) == None` com `return Ok(())` (:5817-5819); `break` de backpressure (:5803-5805) — baseline de finalização: 0% | **PASS (inspeção estrutural)** |
| Helper generation-safe [FR-027-002] | `finish_production_if_current` (:6004-6018) checa `p.generation == generation` (:6009-6013) antes do `take()` (:6010) e instala a timeline medida (:6016); `Production` com campos necessários (:11026-11037) | **PASS (leitura)** |
| Sem `complete` antecipado [FR-027-003] | Gate `drained && !production_running` (:2669), `signal_document_end` (:3215-3226), `dispatch_document_end` (:5533-5555), `is_drained` (`crates/player/src/backend.rs:778-780`); nenhuma chamada de conclusão no corpo atual de `produce_bounded` | **PASS (leitura)** |
| Sem enum / fronteiras [FR-027-004] | `production_running` (:6360-6365), `advance_production` (:6305-6327), `handover_pending_for_generation` (:5943-5971), `queue_room` (:6331-6341), primário de 026 e contrato PCM lidos, não alterados | **PASS (leitura)** |
| Teste no target pinado [FR-027-005] | `t007_produce_bounded_commits_first_unit_without_draining_all_segments` (:11007-11061) fornece o construtor de `Production` e o padrão de hook para o caso de regressão; nenhum target novo especificado | **PASS (mapeamento)** |
| Qualidade da spec | Sem `[NEEDS CLARIFICATION]`; 7 clarificações; 6 FR e 6 SC rastreáveis; nenhum enum novo; nenhuma meta absoluta | **PASS (revisão documental)** |
| Checklist de qualidade | `specs/027-production-exhaustion/checklists/requirements.md` — CHK001..CHK023 | **23/23 `[x]` — PASS** |
| Análise cruzada [FR-027-005] | `analyze.md` — 12/12 requisitos cobertos, 0 CRITICAL/HIGH/MEDIUM, 5 divergências LOW resolvidas | **PASS** |
| Limites | `[NV]` para dispositivo presente; nenhum arquivo de outra spec alterado por esta cadeia | **PASS** |
| Diff, fmt/clippy, gates, builds, testes | Não executados/observados por este artefato | **[NV]** |

---

## 2. Success Criteria — SC-027-001..006

| SC | Critério (spec) | Evidência LOCAL | Resultado |
|---|---|---|---|
| **SC-027-001** | 100% dos estados exauridos com geração atual → `production == None` e timeline instalada (baseline 0%) | Causa confirmada em :5817-5819; helper existente :6004-6018; verificação final depende do teste T012 | **PASS no mapeamento em host**; aplicação e teste `[NV]` |
| **SC-027-002** | 100% de geração divergente/lock envenenado preserva a produção; zero deadlock | `production_generation_matches` (:5708-5710), checagem do helper (:6009-6013), ramos de lock envenenado (:5785-5787, :6006-6008), soltura do guard prevista no plan | **PASS no mapeamento em host** |
| **SC-027-003** | zero `complete` no produtor; gate de drenagem único | :2669, :3215, :5533; nenhuma chamada de conclusão em :5773-5863 | **PASS em host**; `action=complete`/`session_end` em device `[NV]` |
| **SC-027-004** | comportamentos existentes preservados; zero enum; zero mudança em `production_running`/`advance_production` | `t007` (:11007-11061), :6305-6327, :6360-6365, contrato PCM intacto no estado lido | **PASS (mapeamento)** |
| **SC-027-005** | 100% dos testes no target pinado; zero target novo; `expected-tests.txt` intacto; diff só em `lib.rs` | Módulo `ttfa_013_tests` (:11007+); nenhum arquivo de teste novo previsto; diff ainda a aplicar | **PASS no planejamento**; diff final `[NV]` |
| **SC-027-006** | 12/12 rastreados, checklist 23/23, `go`, sem ambiguidade, `[NV]` device | `analyze.md` (12/12), checklist 23/23, `decision.md`, marcadores `[NV]` na cadeia | **PASS** |

> As SCs são mensuráveis em host por inspeção estrutural ou por testes no target pinado. Os itens que dependem da aplicação do código estão marcados como mapeamento validado, não como verificação de execução. Nenhuma afirmação de comportamento em dispositivo é reivindicada.

---

## 3. [NV] — Não verificado neste ciclo

- **Implementação de código**: checagem de exaustão, chamada ao helper, `break` e teste de regressão não aplicados por esta cadeia documental — **[NV]**.
- **Dispositivo Android (`2201117TY`)**: `action=complete` + `session_end` após >220 s com diag zerado, e A/B com/sem primário — **[NV]**. Bug de origem permanece `pending`.
- **`verify.sh` completo (gates 1–9)**: não executado por este artefato; exit code pendente de observação pelo orquestrador — **[NV]**.
- **Builds, formatadores e testes**: não executados nesta cadeia documental — **[NV]** para a suíte completa.
- **CI remoto / PR / merge**: pipelines, status e merge não executados nem observados — **[NV]**.
- **Gate 9 / aparelho**: build/pacote aarch64 (APK/AAB), instalação e medição de áudio em device — **[NV]**.

---

## 4. Limites do ciclo

- Nenhum arquivo foi editado por esta cadeia documental além dos artefatos do ciclo 027: 5 assessments, `spec.md`/`plan.md`/`tasks.md`/`analyze.md`/`converge.md`, checklist, este `validate.md` (R4) e `release.md` (R7).
- `specs/013`, `specs/026`, `.specify/bugs/playback-stall-at-end-of-document`, `scripts/verify.sh`, `scripts/expected-tests.txt` e `converge.md` de outras specs não foram alterados.
- Nenhum `git add`/`git commit` foi realizado; as mudanças permanecem no working tree do branch, **sem merge**.
- Nenhuma afirmação de gate, build ou teste é feita além dos checks documentais/host efetivamente observados na seção 1.
