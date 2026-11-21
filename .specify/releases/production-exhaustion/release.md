# Release (R7): 027 Exaustão da produção não finaliza o documento

- **Date**: 2026-11-21
- **Branch**: `feature/027-production-exhaustion` (criada de `dev @ 61c86200`)
- **SemVer proposto**: **v0.1.0-rc6** (pre-release incremental sobre a proposta `v0.1.0-rc5` do ciclo 026; sem bump de aplicativo independente — a trilha `v0.1.0-rc` permanece). **Proposta, não aplicada.** Tags presentes no checkout: `v0.1.0-alpha.1`, `v0.1.0-rc1`, `v0.1.0-rc2`.
- **Validation**: R4 PASS nos checks documentais e de leitura de host realizados (`.specify/validations/production-exhaustion/validate.md`); implementação de código, dispositivo, `verify.sh` completo, builds, testes, CI e merge `[NV]`.
- **Status de merge**: **NÃO houve merge.** Nenhum `git add`/`git commit` foi realizado por este ciclo de artefatos.

---

## 1. Resumo

Correção do bug `playback-stall-at-end-of-document`, origem desta cadeia. Quando
a produção de um documento se esgota — todas as unidades produzidas e
`pending` vazio — `produce_bounded` (`src-tauri/src/lib.rs:5773-5863`) retornava
sem remover `state.production`. Como `production_running` (:6360-6365) reflete
apenas a presença da estrutura, o gate de fim em `playback_watch_tick` (:2669,
`drained && !production_running`) nunca abria: sem `action=complete`, sem
`session_end`, MediaSession preso em `state=3` (PLAYING) com a posição congelada
em 199,203 s no smoke do ciclo 026. A correção detecta a exaustão no topo do
loop, solta o guard e chama `finish_production_if_current` (:6004-6018), que já
é generation-safe e instala a timeline medida. O `complete` NÃO é antecipado:
a conclusão continua dependendo da drenagem real do backend. Nenhum enum novo,
nenhum target novo, fronteiras de áudio e chunk intactas.

---

## 2. Changelog (FR/T)

- **FR-027-001 / T003, T004, T005**: checagem de exaustão no topo do `while`
  (:5782), antes de `queue_room`, do ramo `has_pending` (:5784-5802), do `break`
  de backpressure (:5803-5805) e de qualquer retorno. Condição: geração atual,
  `next_unit >= segments.len()`, `pending.is_empty()`. A checagem de `done`
  pós-commit (:5847-5860) é preservada para o tick da última unidade.
- **FR-027-002 / T001, T002, T004, T006, T007, T008**: `MutexGuard` solto ao
  fim do bloco de checagem; chamada a `finish_production_if_current`
  (:6004-6018), que verifica `p.generation == generation` (:6009-6013) antes do
  `take()` (:6010) e chama `install_session_timeline` (:6016). Lock envenenado
  e geração divergente não finalizam; `started == false` não impede a liberação
  do gate.
- **FR-027-003 / T002, T008, T009**: nenhuma chamada a `signal_document_end`
  (:3215), `dispatch_document_end` (:5533) ou `ShellVerb::Complete` em
  `produce_bounded`. O gate `drained && !production_running` (:2669) permanece a
  única decisão de fim; `action=complete`/`session_end` em dispositivo `[NV]`.
- **FR-027-004 / T001, T010, T011**: nenhum enum, variante ou tipo público
  novo. `production_running` (:6360-6365), `advance_production` (:6305-6327),
  `handover_pending_for_generation` (:5943-5971), `queue_room` (:6331-6341),
  o primário de 160 bytes do ciclo 026 e o contrato PCM mono `f32` @ 22050 Hz
  inalterados.
- **FR-027-005 / T012, T013**: caso de regressão em `ttfa_013_tests`, ao lado de
  `t007_produce_bounded_commits_first_unit_without_draining_all_segments`
  (:11007-11061): exaustão com geração atual → `production == None`; geração
  divergente → produção preservada; `pending` não vazio → não finaliza.
- **FR-027-006 / T011, T013, T014**: diff de código restrito a
  `src-tauri/src/lib.rs`; `scripts/expected-tests.txt`, `Cargo.toml`, HTML,
  capabilities, workflow, specs/bugs de outros ciclos e `converge.md` alheio
  intactos. Rastreabilidade 12/12 em `analyze.md`, checklist 23/23, `[NV]` para
  dispositivo.

Evidência de host local: leitura estrutural de `produce_bounded`, do helper de
finalização, do gate do watch, do construtor de `Production` em `t007` e de
`is_drained` (`crates/player/src/backend.rs:778-780`); `analyze.md` com 5
divergências LOW resolvidas e 23/23 itens do checklist revisados.

---

## 3. Rollback

Reverter a alteração de código em `src-tauri/src/lib.rs` (remover a checagem de
exaustão, a chamada a `finish_production_if_current`, o `break` associado e o
teste de regressão) e remover os artefatos documentais criados neste ciclo,
todos no working tree — como nada foi comitado nem mesclado, o rollback é local
e não afeta nenhuma tag/release publicada:

**Código (1):** `src-tauri/src/lib.rs`.

**Artefatos centrais (5):** `specs/027-production-exhaustion/spec.md`,
`plan.md`, `tasks.md`, `analyze.md`, `converge.md`.

**Documentos de apoio:** `specs/027-production-exhaustion/checklists/requirements.md`,
`.specify/assessments/production-exhaustion/{intake,research,problem,concept,decision}.md`,
`.specify/validations/production-exhaustion/validate.md` e este `release.md`.

Procedimento: restaurar `src-tauri/src/lib.rs` ao estado anterior ao branch e
remover os diretórios de artefatos listados. Nenhuma configuração remota,
migração de schema, surface de consentimento, tag, assinatura ou build assinado
está envolvida. `specs/013`, `specs/026`,
`.specify/bugs/playback-stall-at-end-of-document`, `scripts/verify.sh`,
`scripts/expected-tests.txt` e `converge.md` de outras specs **não** fazem parte
do rollback (nunca foram alterados). Restaurar o produtor devolve o comportamento
anterior: exaustão com retorno silencioso e `production_running == true`, com o
gate de fim bloqueado.

---

## 4. Riscos residuais / [NV]

- **Dispositivo (`2201117TY`)**: `action=complete` + `session_end` após >220 s
  com diag zerado e A/B com/sem primário — **[NV]**; o bug de origem permanece
  `pending` até a reprodução limpa.
- **Implementação de código**: checagem de exaustão e teste ainda não aplicados
  nesta cadeia documental — **[NV]**.
- **`verify.sh` completo (gates 1–9)**: não executado por esta cadeia; resultado
  pendente de observação pelo orquestrador — **[NV]**.
- **Builds, formatadores e testes**: não executados nesta cadeia documental —
  **[NV]** para a suíte completa.
- **CI remoto / merge de PR**: não executado; nenhuma verificação remota é
  reivindicada — **[NV]**.
- **Gate 9 / aparelho**: build/pacote aarch64 (APK/AAB) e medição de áudio —
  **[NV]**.
- **Ramo de backpressure (F3 em `analyze.md`)**: a checagem no topo do loop é
  necessária porque o `break` de `room == 0` ocorre antes do `get()`; validação
  final depende do teste no target pinado — **[NV]** até a execução pelo
  orquestrador.

---

## 5. Confirmação de estado

- Nenhum merge realizado; branch `feature/027-production-exhaustion` com
  mudanças documentais locais, sem commit deste ciclo.
- Nenhuma tag, assinatura, publicação em loja ou configuração remota alterada;
  `v0.1.0-rc6` é apenas uma proposta.
- `specs/013`, `specs/026`, bug de origem, `scripts/verify.sh`,
  `scripts/expected-tests.txt` e `converge.md` de outras specs intactos.
- Cadeia documental completa: assess (5) → spec/plan/tasks/analyze/converge (5)
  + checklist → validate (R4) → release (R7) — 12 arquivos no total.
