# Validation (R4): 025 Nome de exibição no ACTION_VIEW via MediaStore

- **Date**: 2026-11-21
- **Branch**: `025-action-view-display-name`
- **Spec**: `specs/025-action-view-display-name/spec.md` (FR-025-001..005, SC-025-001..005)
- **Status**: **PASS nos checks documentais/host realizados**; dispositivo, gate completo, builds, formatadores, testes e CI remoto estão `[NV]`. Este artefato é documental — nenhum gate, build, teste ou formatador foi executado por ele.

---

## 1. Evidência local obtida (host)

| Check | Evidência LOCAL | Resultado |
|---|---|---|
| Diff restrito [FR-025-004] | `git diff --stat` — 1 arquivo (`android-shell/src/main/kotlin/app/mdreader/DocIngest.kt`), 36 inserções, 10 remoções | **PASS** |
| Consulta à coleção [FR-025-001] | `MediaStore.Files.getContentUri(volume)` com `"${BaseColumns._ID}=?"` e argumento do id (`DocIngest.kt:659-664`) | **PASS (inspeção estrutural)** |
| Guarda de URI [FR-025-001] | `SCHEME_CONTENT` + `MediaStore.AUTHORITY` + `lastPathSegment?.toLongOrNull()` (`DocIngest.kt:655-657`) | **PASS (inspeção estrutural)** |
| Predicado de nome [FR-025-003] | `isUsableName` (:640-641) aplicado em :650, :669 e :780; fallback `document.md` (:781) | **PASS (inspeção estrutural)** |
| URI sintética [FR-025-002] | `content://display_name/${Uri.encode(name)}` (:782) em todos os caminhos do ramo; nenhum `uri.toString()` entre :768 e :784 | **PASS (inspeção estrutural)** |
| Consumidor Rust [FR-025-002] | `picked_document_name` (`src-tauri/src/shell_jni.rs:1103`) e `percent_decode(last)` (:1108) — lido, não alterado | **PASS (leitura)** |
| Caminhos preservados [FR-025-004] | SAF :634-638; `ACTION_SEND` :785+; BLAKE3/snapshot/JNI :734-751 e `shell_jni.rs:849-893` — inalterados | **PASS (diff)** |
| Checklist de qualidade | `specs/025-action-view-display-name/checklists/requirements.md` — CHK001..CHK023 | **23/23 `[x]`** — PASS |
| Qualidade da spec | Sem marcadores `[NEEDS CLARIFICATION]`; 6 clarificações registradas; FR/SC rastreáveis | **PASS (revisão documental)** |
| Gate de avaliação | `.specify/assessments/action-view-display-name/decision.md` — veredito `go` | **PASS** |
| Análise cruzada [FR-025-005] | `analyze.md` — 10/10 requisitos cobertos, 0 CRITICAL/HIGH, 4 divergências LOW resolvidas | **PASS** |
| Limites | `[NV]` para dispositivo e **DEFER** para 183 s presentes; nenhum arquivo de outra spec alterado | **PASS** |

---

## 2. Success Criteria — SC-025-001..005

| SC | Critério (spec) | Evidência LOCAL | Resultado |
|---|---|---|---|
| **SC-025-001** | 100% dos itens MediaStore com `_display_name` exposto produzem esse nome (baseline: consulta direta retornava `null`) | Consulta à coleção com `_id=?` e filtro de nome presente (:659-669); título final depende do provedor em device | **PASS em host (estrutural)**; resolução real em dispositivo é `[NV]` |
| **SC-025-002** | 100% das saídas do `ACTION_VIEW` com URI sintética codificada; zero `uri.toString()` no ramo | Construção única em :782, entregue em :783; nenhum fallback de URI bruta entre :768 e :784 (T006/T007) | **PASS em host** |
| **SC-025-003** | 100% dos nomes filtrados; zero numéricos/vazios; fallback `document.md` | `isUsableName` nos três candidatos (:650, :669, :780) e fallback (:781) (T001/T005/T006) | **PASS em host** |
| **SC-025-004** | Zero mudanças em SAF/`ACTION_SEND`/BLAKE3/JNI; 1 arquivo, 36+/10- | `git diff --stat` e leitura dos blocos preservados (T009/T010/T011/T012) | **PASS em host** |
| **SC-025-005** | 5/5 FRs rastreados; `[NV]` device; 1 DEFER para 183 s | `analyze.md` (10/10 FR+SC), seções `[NV]` e DEFER em todos os artefatos do ciclo (T012) | **PASS** |

> Todas as SCs são mensuráveis em host por inspeção estrutural e foram satisfeitas nos checks realizados. Nenhuma afirmação de comportamento em dispositivo é reivindicada.

---

## 3. [NV] — Não verificado neste ciclo

- **Dispositivo Android (`2201117TY`)**: resolução do `_display_name` na coleção `MediaStore.Files` e título final exibido ("Hamlet trecho.md" no lugar de `document:1000848919.md`) — **[NV]**.
- **`verify.sh` completo (gates 1–8)**: não executado por este artefato; exit code pendente de observação pelo orquestrador — **[NV]**.
- **Builds, formatação e testes**: `cargo fmt`, `cargo clippy`, `cargo test` e build Android não executados nesta cadeia documental — **[NV]**.
- **CI remoto / PR / merge**: pipelines, status e merge não executados nem observados — **[NV]**.
- **Gate 9 / aparelho**: build/pacote aarch64 (APK/AAB) e qualquer medição de áudio em device — **[NV]**.
- **Latência de 183 s até o primeiro áudio** (`preroll_secs=60.000`): registrada como **DEFER**, fora do escopo deste ciclo e sem tarefa associada.

---

## 4. Limites do ciclo

- Nenhum outro arquivo foi editado por esta cadeia documental além dos artefatos listados na Target: 5 assess, 5 specs (com `spec.md` editado), checklist, `validate.md` e `release.md`.
- `specs/019`, `specs/009`, bugs, `scripts/verify.sh` e `converge.md` de outras specs não foram alterados.
- Nenhum `git add`/`git commit` realizado; o branch permanece com as mudanças no working tree e **sem merge** (ver `.specify/releases/action-view-display-name/release.md`).
- Nenhuma afirmação de gate, build ou teste é feita além dos checks documentais/host efetivamente observados nesta seção 1.
