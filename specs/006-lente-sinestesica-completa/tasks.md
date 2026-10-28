---
description: "Task list for Lente Sinestésica Completa (feature 006)"
feature: "Lente Sinestésica Completa"
branch: "006-lente-sinestesica-completa"
feature_dir: "/home/luis/development/mdreader/specs/006-lente-sinestesica-completa"
---

# Tasks: Lente Sinestésica Completa

**Input**: `specs/006-lente-sinestesica-completa/plan.md`, `spec.md`, `checklists/`, `research/` + `specs/005-adopt-design-sona/contracts/` (status-surface.md, surface-audit.md), `data-model.md`, `clarifications.md`.
**Prerequisites**: Rust 1.85+ Edition 2024, Tauri 2.12.1, cargo nextest, Android SDK API 29+.
**Tests**: TDD required; RED-phase tests block implementation per user directive (§1). Property-based where invariants apply (§SonaToken census, C-H1/C-H4). Shared tooling (LSP scaffold, cargo test) runs on ONE agent at a time; other agents queue (§parallelism rule).
**Organization**: Per user story (US1 / US2 / US3 = P1 each per `spec.md` §User Scenarios). Foundational phase (Phase 2) must complete before ANY story.
**Hooks**: `.specify/extensions.yml` exists but `hooks: {}` — no pre/post hooks registered; nothing to execute or skip. Setup script (`setup-tasks.sh --json`) returned `FEATURE_DIR=/home/luis/development/mdreader/specs/006-lente-sinestesica-completa`; `AVAILABLE_DOCS=[]` (directory listing confirms `plan.md`, `spec.md`, `checklists/`, `clarifications.md`, `research/` present — tasks derived from those + `005` contracts/data-model).
**Constitution gates**: S1 (2 surfaces), S3 (closed token set), S5 (movement ≤1 loop, reduced-motion guard), S6 (notification accent-only), §VII coordinated edit required for contract/token changes.

## Execution Rules (mandatory — apply to all subagents)

0. **Scaffold first**: Before handwritten CSS/HTML edits, generate via LSP / existing scaffold (e.g., `.card::before`, `@media(prefers-reduced-motion:reduce)` block at `status_view.html:247`, `#documentTitle`, `.subtitle`). Do NOT hand-write what scaffold can produce.
1. **TDD blocking**: For each US phase, write RED-phase property-based/invariant tests FIRST (tests NOT passing). Implementation tasks (model/service/HTML) must NOT start until all RED test tasks for that phase are written and verified non-passing.
2. **Parallel [P]**: Only on disjoint files with no incomplete dependency. Shared tooling (LSP, `cargo test`, `adb`) queued; NEVER kill/interrupt a running tool from another agent/thread.
3. **Verification only by verification tasks**: Final Phase tasks (T027–T030) are the only attestation of completeness. Agents must NOT self-attest.
4. **Deterministic commits**: After EACH file edit finishes, atomic conventional commit with detailed edit summary (e.g., `feat(status): add decorative wave pseudo-element with reduced-motion guard; update title card aria; justify token reuse per §VII`).

## Dependency Order (stories independent after Phase 2)

```
Phase 1 (Setup) ──► Phase 2 (Foundational / BLOCKING) ──► Phase 3 (US1) ──► Phase 4 (US2) ──► Phase 5 (US3) ──► Phase 6 (Verification)
                         │                            │                    │                  │
                         └── Contract/data-model updates (T010, T011) ────────┘                    └── Surface audit / device trial (T033, T035)
```
- US1, US2, US3 are independent after Phase 2 (no cross-story file overlap except contract/data-model — already done in Phase 2).
- Within each story: Tests (RED) → Scaffold (LSP) → Implementation → Verification.
- Parallel opportunities within US1: T015 (wave CSS, `.card::before`), T016 (title card HTML/JS, `#documentTitle`/`.subtitle`), T017 (layout/controls, `.controls`/`.speed-dots`) — disjoint file regions; can run in parallel once T015–T017 RED tests written.

## Implementation Strategy (MVP first, incremental)

- **MVP = US1 only** (complete visual on StatusView): decorative wave + title card + control refinement + reduced-motion guard. Delivers proposal intent without new surfaces.
- Increment 2 (US2): confirm controls / rate-blind / banner behavior untouched (regression guard).
- Increment 3 (US3): notification purity + surface inventory + contract audit (cross-cutting).
- No new surfaces (S1 preserved); no RMS instrumentation (no `rms`/`reactive` strip); static SVG/CSS preferred; animations only with `prefers-reduced-motion: reduce` guard; decorative `aria-hidden="true"`; title card `role="region"` + `aria-label`.

---

## Phase 1: Setup & Research (no story label)

**Purpose**: Confirm design proposal elements, metadata flow, contract state, dead-element pre-check, token set.
**Independent test**: All Phase 1 artifacts exist (research notes, contract read, pre-check list) before Phase 2 starts.

- [ ] T001 Read proposal reference `DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png`; map to existing tokens/stamps (Concha Harmônica R header, decorative waves, title card region, refined controls). File: `specs/006-lente-sinestesica-completa/research/`
- [ ] T002 Confirm `metadata_title.unwrap_or(filename)` flow to StatusView; verify `status_view.rs` sets `documentTitle` / `sectionTitle`. File: `src-tauri/src/status_view.rs`
- [ ] T003 Load contracts for coordinated edit: `specs/005-adopt-design-sona/contracts/status-surface.md` (S1–S7), `surface-audit.md` (A1–A4). File: contracts/
- [ ] T004 Pre-check `status_view.html` for dead `display:none` corpses (C-H4); record inventory count (`scaffold-inventory` `data-elements="9"`). File: `src-tauri/src/status_view.html`
- [ ] T005 [P] Load `specs/005-adopt-design-sona/data-model.md` entities (`SonaToken` 10-member closed set, `TitleCard`, `LenteCompleta`); verify no banned paints (`#5AD6A8`, `#5FD0A0`, `#B9A6F5` reserved unpainted). File: `specs/005-adopt-design-sona/data-model.md`
- [ ] T006 [P] Review `clarifications.md` (2026-10-27 session): Option C multi-axis animation rules (opacity / translate≤10px / scale≤1.05; duration ≤250ms/≤400ms; no infinite loops; area≤1/3 viewport; mandatory `prefers-reduced-motion` guard with `animation:none`). File: `specs/006-lente-sinestesica-completa/clarifications.md`

---

## Phase 2: Foundational (Blocking — MUST complete before US1/US2/US3)

**Purpose**: Property-based/invariant tests (RED), token audit, contract updates with justification, data-model updates, reduced-motion CSS guard confirmed.
**Blocking rule**: T015/T016/T017 (US1 RED) can start only after T007 (token audit) and T008 (contracts) and T009 (data-model) complete; but T007–T014 (Phase 2) block all story phases.
**Independent test**: Contract files reflect coordinated edits (§VII); token census passes (C-H1); dead-element check passes (C-H4); `@media` guard present.

### 2a. RED-phase invariant / contract tests (write first, must NOT pass yet)

- [ ] T007 [P] [RED] Write property-based contract census test `tests/contract_census.rs`: assert all audited hex values in `status_view.html:247` match `SonaToken` ledger (`--sona-void #0B0F1A`, `--sona-voice #F2F0E6`, `--sona-hush #A9B8CE`, `--sona-etwahl #8FD8E8`, `--signal-caution #FFD400`, `--signal-failure #FF4D63`, `--sona-stage #141B2D`, `--sona-boundary #55688F`, `--sona-harp #E8C46A`); assert `#B9A6F5` (celeste) and `#5AD6A8`/`#5FD0A0` (jade) absent (C-H1); assert exactly 9 inventory elements (`data-elements="9"`). File: `tests/contract_census.rs`
- [ ] T008 [P] [RED] Write reduction freeze test `tests/reduced_motion_guard.rs`: assert under `@media (prefers-reduced-motion:reduce)` the rules apply to `#progress-fill {animation:none !important}` and `.reason-banner.warning {animation:none !important}`; assert decorative pseudo-element `.card::before` has `animation:none` under reduce; assert no `infinite` loops in CSS. File: `tests/reduced_motion_guard.rs`
- [ ] T009 [RED] Write dead-element / inventory audit `tests/surface_inventory.rs`: scan `status_view.html` for `display:none` (C-H4); assert count matches inventory; assert `aria-hidden="true"` on decorative waves; assert progressbar `role="progressbar"` intact. File: `tests/surface_inventory.rs`

> **TDD BLOCK**: T007–T009 (RED contract/inventory tests) must be written (NOT passing) before T010–T014 (contract/data-model/scaffold). Do NOT start T010 until T007–T009 exist.

### 2b. Contract / data-model / token updates (coordinated edit, §VII)

- [ ] T010 Update `specs/005-adopt-design-sona/contracts/status-surface.md`: document decorative wave pseudo-element on `.card::before` (static SVG preferred, reduction guard), title card region (`role="region"`), refined controls (rate value-blind, positional dots unchanged), surface count = 2. Add justification line referencing proposal alignment (non-normative) and token reuse (no new tokens unless unavoidable). File: `specs/005-adopt-design-sona/contracts/status-surface.md`
- [ ] T011 Update `specs/005-adopt-design-sona/contracts/surface-audit.md`: update A1–A4 to include decorative wave render verification (pseudo-element present, `aria-hidden`), title card aria inventory (`role="region"`, `.sr-only` empty state), reduced-motion freeze verification, no dead elements after edit. File: `specs/005-adopt-design-sona/contracts/surface-audit.md`
- [ ] T012 Update `specs/005-adopt-design-sona/data-model.md`: confirm `TitleCard` constraints — `title: String` (`metadata_title.unwrap_or(filename)`), `hasTitle: bool`, `ariaLabel: Option<String>` (populated or `"Sem título"`); `LenteCompleta` flags (waves enabled, animations guarded, reduced-motion). Document that no new token members added (reuse only). File: `specs/005-adopt-design-sona/data-model.md`

### 2c. Scaffold-first CSS guard confirmation (rule 0)

- [ ] T013 [P] Confirm existing `@media(prefers-reduced-motion:reduce)` scaffold at `status_view.html:247`; extend with decorative pseudo-element freeze rule (`.card::before {animation:none !important; transform:none !important;}`) using LSP / template insertion before hand-editing. Do NOT write guard from scratch if scaffold can extend. File: `src-tauri/src/status_view.html` (style block `id="sona-scaffold-tokens"`)
- [ ] T014 [P] Confirm `.card::before` scaffold exists at `status_view.html:67`; verify `background-image: url("DESIGN-SONA/assets/waves/waves-top.svg")`; prepare static SVG reference. Use scaffold; only add `aria-hidden` / animation rules manually. File: `src-tauri/src/status_view.html`

---

## Phase 3: US1 — Ver a Lente Sinestésica completa (P1)

**Story goal**: Header "Concha Harmônica R" + decorative wave + title card (metadata/filename) + refined controls; no numeric rate; reduced-motion frozen; a11y preserved.
**Independent test criteria** (from `spec.md` §US1 / SC-001 / SC-002 / SC-006):
- Header asset visible with correct hex; decorative `::before` visible; title card resolves `metadata_title.unwrap_or(filename)` or `—` with `.sr-only`; no jade/celeste (C-H1); rate value-blind (C-H3); notification accent-only (SC-003); reduced-motion freeze (SC-004); aria preserved (SC-002).

### 3a. RED-phase tests for US1 (blocking — must write first, NOT passing)

- [ ] T015 [US1] [RED] Write `tests/us1_sc001_contract.rs`: property-based assertions — header `#headerMark` src contains `logo-primary-concha-r.svg`; `.card::before` has `background-image` pointing to `waves-top.svg`; `#documentTitle` non-empty when doc loaded; `.subtitle` contains either metadata text or `<span aria-hidden="true">—</span>` + `<span class="sr-only">Sem título</span>`; no text node containing `#5AD6A8` or `#B9A6F5`; `.speed-dots` contains no numeric rate string; `progressFill` not painted with rate number. File: `tests/us1_sc001_contract.rs`
- [ ] T016 [US1] [RED] Write `tests/us1_reduced_motion.rs`: assert `prefers-reduced-motion: reduce` freezes decorative wave (`animation:none`), progress `#progress-fill` static, `.reason-banner.warning` static; assert no new `infinite` loops; assert decorative `aria-hidden="true"`. File: `tests/us1_reduced_motion.rs`
- [ ] T017 [US1] [RED] Write `tests/us1_a11y_inventory.rs`: assert `#progress` has `aria-valuetext` (position-only, not numeric rate); `.subtitle` `.sr-only` announces `"Sem título"` when empty; no `alert` / `assertive` misuse on interactive elements; focus ring visible per `:focus-visible` (`--sona-etwahl`). File: `tests/us1_a11y_inventory.rs`

> **TDD BLOCK**: T015–T017 must exist and verify non-passing before T018–T021 start.

### 3b. Implementation (scaffold first, then hand-edit, with commits)

- [ ] T018 [P] [US1] Implement decorative wave CSS/SCSS on `.card::before` (static SVG reference + subtle allowed animation only with guard): keep `background-image` scaffold; add `animation: wave-subtle 400ms ease-in-out` ONLY outside `@media(reduce)`; add `animation: none !important` inside reduce; add `aria-hidden="true"` on decorative container if needed (already scaffold). Use LSP / scaffold extension; commit after edit. File: `src-tauri/src/status_view.html`
- [ ] T019 [P] [US1] Implement title card data binding: `status_view.rs` reads upstream `metadata_title` and falls back to `filename` for `#documentTitle`; `.subtitle` (`#sectionTitle`) renders `—` with `.sr-only` when empty. Confirm `overflow:hidden; text-overflow:ellipsis; white-space:nowrap; flex:min-width:0` for long filenames (clarification Option C). Use scaffold `#documentTitle` / `.subtitle`; commit after edit. Files: `src-tauri/src/status_view.html` + `src-tauri/src/status_view.rs`
- [ ] T020 [US1] Implement refined control/layout: preserve `.controls` (transport + stop buttons), `.speed-control` (rate dots only, positional — no numeric rate text painted), `.progress` bar. Confirm no new interactive elements; confirm banner `.reason-banner` fixed-height. File: `src-tauri/src/status_view.html`
- [ ] T021 [US1] Add title card region semantics: `.card` already `role="region"` at `status_view.html:251`; confirm `aria-label` is populated from `documentTitle` or falls back to `"Lente Sinestésica"`; confirm empty-state `.sr-only` announces `"Sem título"`. File: `src-tauri/src/status_view.html`

> **Parallel wave**: T018, T019, T020 can run in parallel (disjoint regions: `.card::before` style, `#documentTitle`/`.subtitle` markup + JS, `.controls` layout). Shared tooling (LSP) queued; do NOT interrupt running LSP session.

---

## Phase 4: US2 — Controles e progresso permanecem funcionais (P1)

**Story goal**: Complete visual does not break controls / progress / earcon / rate-blind / banner behavior.
**Independent test criteria** (from `spec.md` §US2 / SC-003 / SC-005):
- Transport/rate changes emit earcon (existing); rate remains value-blind (positional indicator only); warning/failure banner unchanged (static failure, TTL warning, no new alert/obtrusive elements); no regression in notification purity (SC-003).

### 4a. RED-phase regression tests

- [ ] T022 [US2] [RED] Write `tests/us2_rate_blind.rs`: assert `.speed-dots` is positional only (no numeric label); assert `aria-valuetext` on `#progress` does not contain rate number; assert transport button `aria-label` unchanged. File: `tests/us2_rate_blind.rs`
- [ ] T023 [US2] [RED] Write `tests/us2_banner_states.rs`: assert `.reason-banner` has static `animation:none` under failure (`#signal-failure` `#FF4D63`); assert `.reason-banner.warning` uses `#signal-caution` `#FFD400` with TTL (≤3s); assert no new `alert` / `assertive`; assert no auto-disappearing banner. File: `tests/us2_banner_states.rs`

> **TDD BLOCK**: T022, T023 must be written (RED) before T024, T025.

### 4b. Implementation / verification

- [ ] T024 [P] [US2] Confirm controls interactive: focus rings (`--sona-etwahl`) visible on `.primary`, `.secondary`, `.stop`; key order unchanged; no new focusable decorative elements. File: `src-tauri/src/status_view.html`
- [ ] T025 [P] [US2] Confirm progressbar `aria-valuetext` remains positional (e.g., `"velocidade 1,0"` is position descriptor, not numeric rate mapping — verify no `1.25×` or `2.0` painted in `#progressLabel`). File: `src-tauri/src/status_view.html`
- [ ] T026 [US2] Confirm notification / earcon unchanged: `MediaMirror.kt` (accent-only) not modified; no body/rate/voice/mark added; `earcon-mapping.md` unchanged except justification note if needed. File: `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` (read-only verification; edit only if regression found — justify per §VII)

---

## Phase 5: US3 — Sem regressão na pureza de notificação/contratos (P1)

**Story goal**: Notification remains accent-only; surface inventory valid; contracts / data-model consistent; no token creep; dead CSS removed.
**Independent test criteria** (from `spec.md` §US3 / SC-003 / SC-005 / SC-006):
- Only `--sona-voice` accent; no body/rate/voice/mark (SC-003); surface inventory unchanged (`C-H4` passes); aria inventory correct (`C-H5`); reduced-motion freeze at load + after dynamic toggle (SC-006); no dead `display:none` elements.

### 5a. RED-phase purity / inventory / contract tests

- [ ] T027 [US3] [RED] Write `tests/us3_notification_purity.rs`: assert notification payload (or `MediaMirror` output) contains exactly `--sona-voice` (`#F2F0E6`) accent color; assert no `text`/`body`/`rate`/`voice` fields; assert no `#B9A6F5` / `#5AD6A8` in notification style. File: `tests/us3_notification_purity.rs`
- [ ] T028 [US3] [RED] Write `tests/us3_surface_inventory.rs`: scan `status_view.html` after all edits; assert `data-elements` count unchanged (9); assert no `display:none`; assert `.card::before`, `#headerMark`, `#documentTitle`, `.subtitle`, `.controls`, `.speed-control`, `#progress`, `.sr-only` live region, `.reason-banner`, `.version-footer` all visible (no hidden corpses). File: `tests/us3_surface_inventory.rs`
- [ ] T029 [US3] [RED] Write `tests/us3_contract_audit.rs`: assert `contracts/status-surface.md` and `contracts/surface-audit.md` contain justification lines; assert `data-model.md` records no new token members; assert commit message references §VII for any token/contract change (verify via `git log --grep`). File: `tests/us3_contract_audit.rs`

> **TDD BLOCK**: T027–T029 must be written (RED) before T030–T032.

### 5b. Implementation / audit / commit

- [ ] T030 [P] [US3] Update notification contract if needed (`contracts/status-surface.md` §S6): confirm accent-only (`#F2F0E6`); document that no new fields added. If no change, record "no change — verification only" with justification. File: `specs/005-adopt-design-sona/contracts/status-surface.md`
- [ ] T031 [P] [US3] Remove dead CSS (C-H4): delete any `display:none` rules for removed elements; confirm `scaffold-inventory` count accurate; confirm byte-identical S2 check (no accidental content change in non-edited sections). File: `src-tauri/src/status_view.html`
- [ ] T032 [P] [US3] Atomic conventional commit per edit (repo root; files: `src-tauri/src/status_view.html`, `src-tauri/src/status_view.rs`, `specs/005-adopt-design-sona/contracts/status-surface.md`, `specs/005-adopt-design-sona/contracts/surface-audit.md`, `specs/005-adopt-design-sona/data-model.md`): `feat(006): decorative wave + title card + reduced-motion guard; contracts updated per §VII; token reuse justified; C-H1/C-H4 verified`. Each file edit requires its own commit with detailed summary (rule 4). Do NOT batch unrelated edits into one commit without detailed summary. File: repo root (commit messages)
---

## Phase 6: Verification & Cross-Cutting (only these attest completeness — rule 3)

**Purpose**: Only Phase 6 tasks attest success. Agents must NOT self-attest before this phase.
**Independent test**: All SC-001–SC-007 pass; device trial SC-006 passes; contracts consistent; surface audit clean.

- [ ] T033 [VERIFICATION] Run surface audit checklist (`specs/006-lente-sinestesica-completa/checklists/ux-visual.md`, `checklists/contracts-consistency.md`, `checklists/edge-cases.md`): verify SC-001 (elements present / C-H1 / C-H3 / C-H4 / C-H5 / aria), SC-002 (accessibility), SC-003 (notification), SC-004 (reduced-motion), SC-005 (inventory), SC-006 (device trial), SC-007 (audit). File: checklists/
- [ ] T034 [VERIFICATION] Run property-based tests (T007–T029): `cargo nextest run --tests us1_sc001_contract ...`. Confirm RED → GREEN transition documented in commit messages. File: `tests/` + workspace root
- [ ] T035 [VERIFICATION] Device trial (SC-006): install APK on API 29+ device; `adb shell settings put global animator_duration_scale 0`; verify decorative pseudo-element renders; reduced-motion freezes decorative/progress/banner; title card displays with document metadata / filename / empty-state (`.sr-only` `"Sem título"`); section title correct; focus order unchanged; no dead elements (`C-H4`); notification accent-only (`SC-003`). File: on-device / `tests/us3_surface_inventory.rs` replay
- [ ] T036 [VERIFICATION] Confirm single surface (S1): inspect `status_view.html` + Android manifest / `MediaMirror`; confirm no new route / hidden handler / third surface; surface count = 2 (StatusView + notification). File: `src-tauri/src/status_view.html` + `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt`
- [ ] T037 [VERIFICATION] Confirm reduced-motion dynamic toggle: load with `reduce`, then toggle OS animation settings while running; assert freeze activates (wave `animation:none`, progress static, banner static) — evidence for SC-006 / clarification Q3. File: device / `tests/us1_reduced_motion.rs`
- [ ] T038 [VERIFICATION] Final `cargo fmt --all -- --check`; `cargo clippy --workspace -- -D warnings`; `cargo deny check advisories licenses bans sources`; confirm `Cargo.lock` committed (`--locked`). File: workspace root
- [ ] T039 [VERIFICATION] Atomic final commit summarizing Phase 6 results: which SC passed, which contracts updated (§VII justification), which tokens reused vs added (none expected), dead-CSS removed, device trial result, test transition RED→GREEN. File: repo root (commit message)
- [ ] T040 [VERIFICATION] If any contract/token addition WAS unavoidable (should not occur per plan), create justification record in `DESIGN-SONA/tokens.md` + commit message + `specs/005-adopt-design-sona/contracts/status-surface.md` update; then re-run T033–T039. Do NOT skip justification.

---

## Parallel Execution Example (per story, disjoint files)

**US1 parallel wave (after T015–T017 RED written)**:
- Agent A (scaffold/tool queued): T018 decorative wave CSS (`.card::before`) — `src-tauri/src/status_view.html`
- Agent B (waits for LSP if shared): T019 title card HTML/JS (`#documentTitle`, `.subtitle`) — `src-tauri/src/status_view.html` + `status_view.rs`
- Agent C (waits for LSP if shared): T020 control/layout (`.controls`, `.speed-dots`, `.progress`) — `src-tauri/src/status_view.html`
> Shared tooling (LSP code-action generation, `cargo test`) runs on ONE agent; others queue; NEVER interrupted.

**US2 / US3 can start independently after Phase 2** (no file overlap with US1 implementation after contracts/data-model done).

---

## Implementation Notes (per task, for subagent execution)

- **Rule 0 (scaffold)**: Before writing `.card::before` animation rules by hand, use LSP / existing `.card::before` block and `@media` scaffold to extend. Before writing title card binding, use existing `#documentTitle` / `.subtitle` scaffold.
- **Rule 1 (TDD)**: Each US phase starts with RED tests (T015/T016/T017 for US1; T022/T023 for US2; T027/T028/T029 for US3). Implementation tasks (T018–T021 / T024–T025 / T030–T031) must NOT begin until all RED tests for that phase exist. Tests need not pass (RED is correct state before implementation).
- **Rule 2 ([P])**: Marked on disjoint-file tasks (wave CSS, title card, layout) and contract/data-model updates. Subagents must reuse same subagent for context-sharing tasks (e.g., same agent for T018–T020 if using shared LSP session); disjoint-file tasks can spawn concurrent agents.
- **Rule 3 (verification)**: Only T033–T040 attest. No agent reports "done" before T033; no self-attestation before Phase 6.
- **Rule 4 (commits)**: Every file edit (T018, T019, T020, T024, T025, T030, T031, T008, T009, T010, T011) ends with an atomic conventional commit with detailed edit description (what changed, why, which constraint / SC / contract section satisfied, token justification if applicable).

---

## Task Count Summary

- Phase 1 (Setup): 6 tasks (T001–T006)
- Phase 2 (Foundational): 8 tasks (T007–T016) — 3 RED tests + 3 contract/data-model + 2 scaffold
- Phase 3 (US1): 7 tasks (T015–T021) — 3 RED + 3 implementation + 1 semantics
- Phase 4 (US2): 5 tasks (T022–T026) — 2 RED + 3 verification/implementation
- Phase 5 (US3): 6 tasks (T027–T032) — 3 RED + 3 audit/commit
- Phase 6 (Verification): 8 tasks (T033–T040)
- **Total: 39 tasks**
- **Parallel opportunities**: T001/T002 (research); T007/T008/T009 (RED tests); T018/T019/T020 (US1 disjoint regions); T022/T023 (US2 RED); T027/T028/T029 (US3 RED); T030/T031 (US3 audit/disjoint contracts)
- **MVP scope**: Phase 1 + Phase 2 + Phase 3 (US1 only) = T001–T021 (25 tasks); delivers proposal visual with reduced-motion guard, no new surfaces, contracts updated.
- **Story breakdown**: US1 = 7 tasks (3 RED + 4 impl); US2 = 5 tasks (2 RED + 3 impl/verify); US3 = 6 tasks (3 RED + 3 audit/commit).
- **All tasks follow checklist format**: checkbox (`- [ ]`), sequential ID (`T###`), optional `[P]`, required `[US#]` for story phases, exact file paths, no missing fields.
