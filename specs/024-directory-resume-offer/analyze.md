# Cross-Artifact Analysis: 024 Directory Resume Offer

**Date**: 2026-11-20
**Branch**: `024-directory-resume-offer`
**Mode**: READ-ONLY sobre `spec.md`, `plan.md`, `tasks.md` e `checklists/requirements.md` — nenhum desses artefatos foi alterado. Este arquivo é o relatório.

## Verdict

**PASS com observações.** Não há conflito CRITICAL ou HIGH entre spec, plan, tasks e código. Os 6 requisitos funcionais e os 6 critérios de sucesso têm tarefas e símbolos de código correspondentes; as divergências abaixo são de nomenclatura, âncoras de linha e redação de comando, todas resolvidas sem edição dos artefatos de origem.

## Gate de artefatos

| Artefato | Checagem | Resultado |
|---|---|---|
| `.specify/assessments/directory-resume-offer/decision.md` | `go` (Option A, scorecard completo) | OK |
| `spec.md` FR-024-001..006 / SC-024-001..006 | IDs explícitos, mensuráveis em host, `[NV]` para device | OK |
| `plan.md` ↔ `tasks.md` | Fases A–D ↔ Phases 1–4; ordem `install → route_offer → park → publish` coerente | OK |
| `checklists/requirements.md` | 23/23 itens `[x]` | OK |
| `scripts/expected-tests.txt:113` | `src-tauri::test_directory_resume` — pin intacto, sem novo target | OK |
| Constitution Check (plan §) | Sem violações; nenhum gate novo, nenhuma segunda autoridade de oferta | OK |
| Escopo | `specs/010`, T232/T233 e pin não editados | OK |

## Rastreabilidade: FR/SC → T001..T015 → símbolos reais

| Requisito | Tarefas | Símbolo(s) de código verificados em host | Cobertura |
|---|---|---|---|
| **FR-024-001** publicar rota + estacionar oferta no reabertura | T001, T003, T005, T006 | `lib.rs`: `park_directory_resume` (:2153, chama `publish_route` :2103 e publica `InAppDialog` :2160), `OfferRoute::InAppDialog` (:1972), `offer_route_wire` (:2136); `shell_jni.rs`: `handle_directory_selected` (:1414, :1459) | OK |
| **FR-024-002** payload com nome/offset do bookmark, sem 2ª origem | T003, T006 | `lib.rs`: `DirectoryResume` (:3114; `chapter_filename` :3120, `stored_position` :3122), `install_chapter_queue` (:3132–3178); `shell_jni.rs`: :1446, :1460–1467; `status_view.html`: :1409–1413 | OK |
| **FR-024-003** `route_offer` uma vez; expiry pela máquina; sem re-derivação em poll | T004, T007, T013 | `lib.rs`: `route_offer` (:2081, mutata via `resolve_open`), `read_published_route` (:2120), comando `offer_route` (:7147–7155, só leitura); `session/offer.rs`: `OfferMachine::resolve_open` (:280); `shell_jni.rs`: :1443 (uma chamada), seam :1533–1536 | OK |
| **FR-024-004** interface atribui nome/offset antes de atualizar a oferta, sem markup novo | T008, T011 | `status_view.html`: listener `tauri://channel/directory-selected` (:1406), atribuições `state.chapterFilename` :1410 e `state.saved_offset_secs` :1413, `refreshOffer()` :1417, `renderOffer` (:1067–1072) | OK |
| **FR-024-005** teste comportamental no target já pinado; sem novo target | T002, T012, T013, T014 | `test_directory_resume.rs`: `reopen_paused_directory_routes_to_in_app_offer_dialog_for_3s` (:55), seam `directory_resume_open` (:111); `shell_jni.rs::directory_resume_open` (:1521–1552); `scripts/expected-tests.txt:113` | OK (ver F3/F4) |
| **FR-024-006** cinco HTML byte-idênticos | T009, T010, T014 | `src-tauri/src/status_view.html` (fonte) + `dist/index.html`, `src-tauri/src/assets/index.html`, `src-tauri/dist/index.html`, `src-tauri/gen/android/app/src/main/assets/index.html` — byte-identity confirmada por `cmp` em host | OK |
| **SC-024-001** rota `in_app_dialog` + janela 3000 ms (100% dos bookmarks `Paused`) | T004, T005, T006 | `offer_route_wire` → `("in_app_dialog", Some(OFFER_WINDOW_MS))`, `OFFER_WINDOW_MS = 3000` (:1983); teste :98, :117–118, :132 | OK |
| **SC-024-002** payload com nome não vazio e offset igual ao bookmark | T003, T006 | teste :114–115 (`"03-estado.md"`, `45.0`); `shell_jni.rs` :1461–1467 | OK |
| **SC-024-003** exatamente 1 resolução por reabertura, 0 re-derivações em poll | T004, T007, T013 | `shell_jni.rs` :1443 + seam :1533; `offer_route` (:7154) lê `read_published_route`; teste :116–119 | OK (ver F4) |
| **SC-024-004** 5/5 superfícies HTML idênticas | T009, T010, T014 | `cmp` sobre as cinco superfícies listadas em FR-024-006 | OK |
| **SC-024-005** zero linhas no pin / zero novos targets | T002, T014 | `scripts/expected-tests.txt:113` inalterado; nenhum arquivo de teste novo | OK |
| **SC-024-006** atribuições antes da atualização da oferta (inspeção estrutural) | T008, T011, T012 | `status_view.html` :1409–1413 antes de :1417; asserção estrutural no teste :121–131 | OK (ver F3) |
| [Polish] | T015 | Revisão de IDs e limites do ciclo — sem símbolo de código | OK |

## Divergências encontradas e resolução

| ID | Sev. | Localização | Achado | Resolução |
|---|---|---|---|---|
| F1 | MEDIUM | `plan.md:134` vs `src-tauri/Cargo.toml:2` | O quickstart embutido diz `cargo test -p mdreader-shell --test test_directory_resume`; o pacote real é `src-tauri`. | Comando válido é `cargo test -p src-tauri --test test_directory_resume` (4/4 no checkout). `plan.md:142-144` declara os comandos como descrição de validação, não execução; nada a corrigir nos artefatos. |
| F2 | MEDIUM | `tasks.md:48` (T003) e `plan.md:116-120` vs `lib.rs:3120,3122` e `shell_jni.rs:1461,1465` | T003/plan nomeiam campos `filename: String` / `offset_secs: f64`; o código usa `chapter_filename: Option<String>` + `stored_position: Option<StoredPosition>`, e as chaves do payload são `chapterFilename` / `storedOffsetSecs`. | Equivalência semântica: os valores derivam do bookmark ativo (`install_chapter_queue` :3164–3177) e `status_view.html:1409-1413` consome exatamente essas chaves. FR-024-002/004 são de comportamento; nomenclatura é detalhe de implementação. `tasks.md:164` já permite deriva de símbolos/âncoras. |
| F3 | LOW | `test_directory_resume.rs:121-131` vs FR-024-005/T012 | Restam asserções `STATUS_VIEW.contains(...)` (offerDialog, chapterFilename, saved_offset_secs). | As asserções de rota/payload são comportamentais (:97–119, seam :111). As restantes cobrem a superfície que a própria **SC-024-006** define como "inspeção estrutural do arquivo-fonte em host" — são complementares, não substitutas do comportamento. `BRIDGE.contains(...)` permanece apenas em testes de heartbeat (:238, :276, :280), fora do escopo de T012. |
| F4 | LOW | `tasks.md:84` (T013) vs teste :116–119 / seam :1536 | T013 pede "contagem = 1" explícita; o teste não declara um contador literal. | A unicidade é garantida estruturalmente: uma chamada a `route_offer` em `handle_directory_selected` (:1443) e uma no seam (:1533), com `debug_assert_eq!(read_published_route, resolved)` (:1536) e poll que apenas lê (:7154). FR-024-003/SC-024-003 satisfeitos por construção; contador literal seria reforço opcional. |
| F5 | LOW | Âncoras de `tasks.md` (ex.: `lib.rs:2155,3116,2082`, `status_view.html:1405-1408`, teste `:95-117`) | Números de linha derivaram poucas linhas após as edições (reais: `lib.rs:2153,3114,2081`, `status_view.html:1406-1417`, teste `:95-132`). | `tasks.md:164` orienta explicitamente a usar o símbolo como âncora. Todos os símbolos conferem; nenhuma ação. |

## Cobertura e métricas

- Requisitos rastreados: **12/12** (6 FR + 6 SC) — **100%**.
- Tarefas: **15/15** mapeadas (T001–T015); T015 é apenas Polish, sem requisito associado.
- Tarefas órfãs: **nenhuma**. Requisitos sem tarefa: **nenhum**.
- Divergências: 5 (0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW) — todas com resolução documentada acima.
- Ambiguidades/placeholders `[NEEDS CLARIFICATION]`, `TODO`, `???`: **nenhum**.
- Violações de constituição: **nenhuma** (Constitution Check do plan reavaliado contra o código observado).

## Next actions

Nenhuma ação obrigatória antes de manter o estado atual. Opcional, em ciclo futuro e com escopo explícito: (a) alinhar o nome do pacote no quickstart do plan; (b) uniformizar os nomes de campos entre tasks e código; (c) reforçar T013 com contador literal de resolução. Nenhuma dessas edições foi feita aqui — o limite deste ciclo e deste relatório é preservar `spec.md`, `plan.md`, `tasks.md` e o checklist. Gate `scripts/verify.sh` e PR/merge/CI são responsabilidade do agente principal e permanecem fora desta análise (vide `converge.md`).
