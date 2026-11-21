# Cross-Artifact Analysis: 025 Action View Display Name

**Date**: 2026-11-21
**Branch**: `025-action-view-display-name`
**Mode**: READ-ONLY sobre `spec.md`, `plan.md`, `tasks.md` e `checklists/requirements.md` — nenhum desses artefatos foi alterado por esta análise. Este arquivo é o relatório.

## Verdict

**PASS com observações.** Não há conflito CRITICAL ou HIGH entre spec, plan, tasks e código. Os 5 requisitos funcionais e os 5 critérios de sucesso têm tarefas e símbolos de código correspondentes; as divergências abaixo são de escopo de contagem e de âncora de linha, todas resolvidas sem edição dos artefatos de origem. Nenhuma afirmação de comportamento em dispositivo é feita.

## Gate de artefatos

| Artefato | Checagem | Resultado |
|---|---|---|
| `.specify/assessments/action-view-display-name/decision.md` | `go` (Option A, scorecard completo) | OK |
| `spec.md` FR-025-001..005 / SC-025-001..005 | IDs explícitos, mensuráveis em host, `[NV]` para device, DEFER para 183 s | OK |
| `plan.md` ↔ `tasks.md` | Fases A–D ↔ Phases 1–4/5; sequência `nome → extensão → bytes → URI → JNI` coerente | OK |
| `checklists/requirements.md` | CHK001..CHK023 — 23/23 itens `[x]` | OK |
| Diff de código | 1 arquivo alterado (`DocIngest.kt`), 36 inserções / 10 remoções, quatro seções | OK |
| Constitution Check (plan §) | Sem violações; nenhuma segunda origem de nome, nenhuma mudança JNI | OK |
| Escopo | Rust, SAF, `ACTION_SEND`, BLAKE3, specs 019/009 e bugs intactos | OK |

## Rastreabilidade: FR/SC → T001..T012 → símbolos reais

| Requisito | Tarefas | Símbolo(s) de código verificados em host | Cobertura |
|---|---|---|---|
| **FR-025-001** resolver nome: direto e coleção MediaStore por `_id=?` | T003, T004, T005 | `DocIngest.kt`: `queryName(resolver, uri)` (:643-670) — consulta direta a `OpenableColumns.DISPLAY_NAME` (:644-650), guarda de scheme/authority (:655), `lastPathSegment?.toLongOrNull()` (:656), `volume` (:657), `MediaStore.Files.getContentUri(volume)` com `"${BaseColumns._ID}=?"` (:659-665), filtro final (:669); imports `BaseColumns` (:9) e `MediaStore` (:11) | OK |
| **FR-025-002** `effectiveUri` `content://display_name/<codificado>` e consumo Rust sem mudança JNI | T002, T006, T007, T008 | `DocIngest.kt:782` (`Uri.encode(name)`); `Triple(rawBytes, effectiveUri, ext)` (:783); `src-tauri/src/shell_jni.rs`: `picked_document_name` (:1103-1134), `percent_decode(last)` (:1108) | OK |
| **FR-025-003** predicado único, rejeição de numérico, fallback `document.md` | T001, T005, T006 | `DocIngest.kt`: `isUsableName` (:640-641: `!isNullOrBlank() && !all { isDigit() }`), aplicado em :650, :669 e :780; cascata `displayName ?: lastPathSegment?.takeIf… ?: "document.md"` (:779-781) | OK |
| **FR-025-004** preservar SAF, `ACTION_SEND`, BLAKE3/snapshot e assinatura JNI; diff localizado | T009, T010, T011 | `DocIngest.kt`: `queryName(resolver, treeUri, docId)` (:634-638, SAF — inalterado); ramo `Intent.ACTION_SEND` (:785+, inalterado); `blake3_256`/`writeSnapshot`/`onDocumentIngested` (:734-751, inalterados); `shell_jni.rs::Java_app_mdreader_DocIngest_onDocumentIngested` (:849-893, inalterado); `git diff --stat` = 1 arquivo | OK |
| **FR-025-005** verificação em host, `[NV]` para device e DEFER para 183 s | T012 | `analyze.md` (esta tabela), `validate.md` (R4), `release.md` (R7), `converge.md` (append-only); marcadores `[NV]` e **DEFER** presentes em spec/plan/tasks e nos artefatos de apoio | OK |
| **SC-025-001** nome da coleção em 100% dos itens com `_display_name` | T003, T004, T005 | consulta à coleção com `_id=?` (:662-664) e filtro (:669); baseline: consulta direta retornava `null` | OK (host) |
| **SC-025-002** 100% das saídas do `ACTION_VIEW` com URI sintética, zero fallback de URI bruta no ramo | T006, T007 | construção única em :782, entregue em :783; nenhum `uri.toString()` entre :768 e :784 | OK (host) |
| **SC-025-003** 100% dos nomes filtrados; zero numéricos; fallback `document.md` | T001, T005, T006 | `isUsableName` (:640-641) nos três candidatos (:650, :669, :780) e fallback (:781) | OK (host) |
| **SC-025-004** zero mudanças nos caminhos preservados; 1 arquivo, 36+/10- | T009, T010, T011, T012 | `git diff --stat` sobre `DocIngest.kt`; blocos alterados: imports, predicado/`queryName`, ramo `ACTION_VIEW` | OK (host) |
| **SC-025-005** 5/5 FRs rastreados; `[NV]` device; 1 DEFER | T012 | esta tabela (5 FR + 5 SC), seções `[NV]` e DEFER nos artefatos | OK |
| [Polish] | T012 | Revisão de IDs e limites do ciclo — sem símbolo de código | OK |

## Divergências encontradas e resolução

| ID | Sev. | Localização | Achado | Resolução |
|---|---|---|---|---|
| F1 | LOW | `tasks.md` T002 / `shell_jni.rs:1103` | A tarefa cita o intervalo 1103-1134 para `picked_document_name`; a função e seu helper de extensão ocupam esse bloco, mas o `percent_decode` citado está em :1108. | Intervalo correto; nenhuma ação. `tasks.md` orienta usar o símbolo como âncora, não o número. |
| F2 | LOW | `spec.md` SC-025-002 vs `DocIngest.kt:260` | A SC exige "zero ocorrências do retorno à URI original como fallback", mas `uri.toString()` continua existindo em :260 — fora do ramo `ACTION_VIEW`. | A SC é explícita: "dos caminhos de saída do ramo `ACTION_VIEW`". A ocorrência em :260 pertence ao caminho SAF/diretório preservado por FR-025-004. Sem conflito. |
| F3 | LOW | `tasks.md` T004 vs `DocIngest.kt:657` | O volume é inferido do primeiro segmento com default `external`; não há validação contra a lista de volumes disponíveis. | Premissa registrada em `spec.md §Assumptions`: volumes não-padrão degradam para `document.md`, nunca para a URI bruta. Nenhuma ação neste ciclo. |
| F4 | LOW | `spec.md` FR-025-002 vs `DocIngest.kt:771-772` | Quando `displayName` é `null`, a extensão deixa de vir do nome e passa a depender de `resolveFormat`/`md`. | Comportamento já existente e preservado por T008; o requisito fala do nome codificado na URI, não da origem da extensão. Sem conflito. |

## Cobertura e métricas

- Requisitos rastreados: **10/10** (5 FR + 5 SC) — **100%**.
- Tarefas: **12/12** mapeadas (T001–T012); T012 cobre simultaneamente FR-025-005 e Polish.
- Tarefas órfãs: **nenhuma**. Requisitos sem tarefa: **nenhum**.
- Divergências: 4 (0 CRITICAL, 0 HIGH, 0 MEDIUM, 4 LOW) — todas com resolução documentada acima.
- Ambiguidades/placeholders `[NEEDS CLARIFICATION]`, `TODO`, `???`: **nenhum** nos artefatos do ciclo.
- Violações de constituição: **nenhuma** (Constitution Check do plan reavaliado contra o diff em host).
- Marcadores de limite: `[NV]` para dispositivo e **DEFER** para a latência de 183 s — presentes e sem tarefa associada ao DEFER.

## Next actions

Nenhuma ação obrigatória antes de manter o estado atual. Opcional, em ciclo futuro e com escopo explícito: (a) validar o `volume` contra a lista de volumes disponíveis; (b) substituir `document.md` por um nome derivado de metadados do próprio item quando a coleção não responder. Nenhuma dessas edições foi feita aqui — o limite deste relatório é preservar `spec.md`, `plan.md`, `tasks.md` e o checklist. Gate `scripts/verify.sh`, PR/merge/CI e qualquer verificação em dispositivo são responsabilidade do agente principal e permanecem fora desta análise (vide `converge.md` e `validate.md`).
