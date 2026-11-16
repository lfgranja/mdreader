---
description: "Task list for feature implementation"
---

# Tasks: Adotar o DESIGN-SONA (Option A subset)

**Input**: Design documents from `/specs/005-adopt-design-sona/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md
**Tests**: TDD binding per Constitution Principle III. Tests FIRST (RED) before implementation. Property-based testing (`proptest`) where applicable.
**Organization**: Tasks grouped by user story (US1–US4). Setup/Foundational/Polish/Verification have no story labels.
**Branch**: `pr/005-sc001-sc002-sc007-tests` (HEAD `8a4dd32e`) — tests for SC-001/SC-002/SC-007 already added.

- [X] T001 Verify on correct branch `pr/005-sc001-sc002-sc007-tests` (or `005-adopt-design-sona`), record HEAD SHA in state. File: `.specify/memory/STATE.md`
- [X] T002 Run `scripts/verify.sh` gates 1–8 on current tree; record results. Must be green or document existing state. File: `scripts/verify.sh`
- [X] T003 Audit anchors per plan: `crates/player/src/backend.rs`, `crates/player/src/earcon.rs`, `crates/diag/src/writer.rs`, `src-tauri/src/status_view.html`, `src-tauri/src/status_view.rs`, `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` (:701-744), `src-tauri/tauri.conf.json`, `DESIGN-SONA/`, `scripts/verify.sh`, `scripts/expected-tests.txt`. Record findings. File: `.specify/memory/STATE.md`
- [X] T004 Create `src-tauri/src/surface_audit.rs` with closed auditee resolver (A1: `status_view.html`, `status_view.rs` relevant, `MediaMirror.kt` :701-744 slice, `DESIGN-SONA/*` audited, `tauri.conf.json`, `logo-primary-concha-r.svg`). Add `pub mod surface_audit;` to `src-tauri/src/lib.rs`. Scaffold via LSP first. Files: `src-tauri/src/surface_audit.rs`, `src-tauri/src/lib.rs`
- [X] T005 [RED] [P] Add served-identity twin test `surface_audit_served_identity_binds_status_view_as_bundle` in `src-tauri/src/surface_audit/served_identity.rs` asserting byte-identity between `src-tauri/src/status_view.html` and served bundle entry (`frontendDist/index.html`). Must fail initially. Files: `src-tauri/src/surface_audit/served_identity.rs`, `src-tauri/tauri.conf.json`
- [X] T006 Implement packaging wiring via `src-tauri/build.rs`: copy `src/status_view.html` → `dist/index.html` at build time with `cargo:rerun-if-changed`; ensure bundle serves restyled surface. Update build script. Files: `src-tauri/build.rs`, `src-tauri/tauri.conf.json`
## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: US1–US4 for user story phases; none for Setup/Foundational/Polish/Verification
- Every task includes exact file paths
- **BLOCKING**: Gates later tasks until complete
- **Scaffold-first**: Use LSP/code actions before hand-writing
- **Deterministic commits**: After editing any file, commit via `scripts/commit-hook.sh <file> "<summary>"`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Branch/state validation, baseline gates, anchor audit.

- [X] T001 Verify on correct branch `pr/005-sc001-sc002-sc007-tests` (or `005-adopt-design-sona`), record HEAD SHA in state. File: `.specify/memory/STATE.md`
- [X] T002 Run `scripts/verify.sh` gates 1–8 on current tree; record results. Must be green or document existing state. File: `scripts/verify.sh`
- [X] T003 Audit anchors per plan: `crates/player/src/backend.rs`, `crates/player/src/earcon.rs`, `crates/diag/src/writer.rs`, `src-tauri/src/status_view.html`, `src-tauri/src/status_view.rs`, `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` (:701-744), `src-tauri/tauri.conf.json`, `DESIGN-SONA/`, `scripts/verify.sh`, `scripts/expected-tests.txt`. Record findings. File: `.specify/memory/STATE.md`

**Checkpoint**: Baseline known, anchors verified.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Surface audit infrastructure + packaging wiring (byte-identity). **MUST complete before any user story.**

- [X] T004 Create `src-tauri/src/surface_audit.rs` with closed auditee resolver (A1: `status_view.html`, `status_view.rs` relevant, `MediaMirror.kt` :701-744 slice, `DESIGN-SONA/*` audited, `tauri.conf.json`, `logo-primary-concha-r.svg`). Add `pub mod surface_audit;` to `src-tauri/src/lib.rs`. Scaffold via LSP first. Files: `src-tauri/src/surface_audit.rs`, `src-tauri/src/lib.rs`
- [X] T005 [RED] [P] Add served-identity twin test `surface_audit_served_identity_binds_status_view_as_bundle` in `src-tauri/src/surface_audit/served_identity.rs` asserting byte-identity between `src-tauri/src/status_view.html` and served bundle entry (`frontendDist/index.html`). Must fail initially. Files: `src-tauri/src/surface_audit/served_identity.rs`, `src-tauri/tauri.conf.json`
- [X] T006 Implement packaging wiring via `src-tauri/build.rs`: copy `src/status_view.html` → `dist/index.html` at build time with `cargo:rerun-if-changed`; ensure bundle serves restyled surface. Update build script. Files: `src-tauri/build.rs`, `src-tauri/tauri.conf.json`
- [X] T007 [RED] [P] Add C-H1 chroma census test in `src-tauri/src/surface_audit/chroma.rs` over 10 hex set (8 sona + 2 signals), checking both progress branches (determinate/indeterminate), fail on off-set/vacancy. Files: `src-tauri/src/surface_audit/chroma.rs`
- [X] T008 [RED] [P] Add C-H2 role-exclusivity test in `src-tauri/src/surface_audit/exclusivity.rs` (focus ring only `:focus-visible`, warning/failure exclusivity). Files: `src-tauri/src/surface_audit/exclusivity.rs`
- [X] T009 [RED] [P] Add C-H6 movement census in `src-tauri/src/surface_audit/movement.rs` (≤1 loop, reduced-motion freeze visible, pulse TTL bounded). Files: `src-tauri/src/surface_audit/movement.rs`
- [X] T010 [RED] [P] Add C-H7 contrast solver in `src-tauri/src/surface_audit/contrast.rs` (WCAG 2.1 AA, pure function). Files: `src-tauri/src/surface_audit/contrast.rs`
- [X] T011 [RED] [P] Add C-H5 aria inventory in `src-tauri/src/surface_audit/aria.rs` (progressbar/aria-valuetext, hidden status region, no numeric paint, no alert+polite). Files: `src-tauri/src/surface_audit/aria.rs`
- [X] T012 Add diag event kinds `EarconTruncation`, `EarconEmissionFailure` per coordinated §VII edit: update `crates/diag/src/writer.rs`, `crates/diag/src/scan.rs`, contract schema, `crates/diag/CHANGELOG.md`, add ADR reference `docs/adr/adr-002-diag-§VII.md` if needed. Files: `crates/diag/src/writer.rs`, `crates/diag/src/scan.rs`, `crates/diag/*`, `docs/adr/adr-002-diag-§VII.md`

**Checkpoint**: Audit scaffolding + twins RED; §VII diag extension coordinated.

---

## Phase 3: User Story 1 - Identidade visual/sonora na notificação (Priority: P1) 🎯 MVP

**Goal**: MediaStyle uses only accent `#F2F0E6` (`--sona-voice`), title/section/progress only, warning/failure via static reason + earcons, no pulsing dot in notification.
**Independent Test**: `surface_audit_accent_only_constants` green; device inspection per quickstart §3.1.

### Tests for User Story 1
- [X] T013 [P] [US1] Add/verify `surface_audit_accent_only_constants` in `src-tauri/src/surface_audit/accent.rs` asserting MediaMirror constants use `0xFFF2F0E6` only for accent, no body/rate/brand. Files: `src-tauri/src/surface_audit/accent.rs`, `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt`

### Implementation for User Story 1
- [X] T014 [US1] Verify `MediaMirror.kt` constants block `:701-744` (accent `DESIGN_SONA_PROGRESS_FILL = 0xFFF2F0E6`, KDoc retagged). No manifest/permission changes. Files: `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt`

**Checkpoint**: US1 host leg green.

---

## Phase 4: User Story 2 - Confirmar controles por earcon sem olhar (Priority: P1)

**Goal**: 7 confirmation types with correct mapping, burst `latest wins` with truncation, ≤100ms first-consumed, typed diag events on failure.
**Independent Test**: Property tests for burst/latency green; SC-002 protocol ready.

### Tests for User Story 2
- [X] T015 [P] [US2] Add `prop_earcon_burst_latest_wins` in `crates/player/tests/` or unit tests (bursts 2–3 in 100–150ms). Files: `crates/player/tests/*`, `crates/player/src/backend.rs`
- [X] T016 [P] [US2] Add latency oracle test asserting ≤100ms first-consumed-sample. Files: `crates/player/tests/*`, `crates/player/src/backend.rs`

### Implementation for User Story 2
- [X] T017 [US2] Implement burst policy in `crates/player/src/backend.rs` (`enqueue_earcon`): truncate/substitute "latest wins", add counters, emit typed diag events (`EarconTruncation`, `EarconEmissionFailure`), max 1 re-emission on failure, narration never ducked. Files: `crates/player/src/backend.rs`, `crates/player/src/earcon.rs`
- [X] T018 [US2] Ensure grammar/assertions match contracts (`contracts/earcon-mapping.md`, E1–E7). Files: `crates/player/src/earcon.rs`, `specs/005-adopt-design-sona/contracts/earcon-mapping.md`

**Checkpoint**: US2 earcon behavior verified by tests.

---

## Phase 5: User Story 3 - Superfície in-app com tokens DESIGN-SONA (Priority: P2)

**Goal**: Restyle `status_view.html` to closed token set, role exclusivity, aria semantics, movement rules, served bundle identity.
**Independent Test**: C-H1..C-H7 green; SC-001 host leg green.

### Tests for User Story 3
- [X] T019 [P] [US3] Ensure all surface-audit twins (T005,T007-T011) pass after implementation. Files: `src-tauri/src/surface_audit/*`

### Implementation for User Story 3
- [X] T020 [US3] Restyle `src-tauri/src/status_view.html`: apply 10-hex palette (8 sona + 2 signals), focus ring `:focus-visible` with `--sona-etwahl`, reduced-motion guard freezing pill visibly, indeterminate solid pill (no shimmer gradient), role/aria (progressbar, hidden status region, no numeric paint), remove dead elements, preserve single-column. Files: `src-tauri/src/status_view.html`
- [X] T021 [US3] Update `src-tauri/src/status_view.rs` if needed for inventory/wiring. Files: `src-tauri/src/status_view.rs`
- [X] T022 [US3] Align DESIGN-SONA docs/tokens if needed (`tokens.md`, `components.md`, `DESIGN-SONA.md`, `preview.html`) with normalized v2.2.0; add `CHANGELOG.md`, `VERSION` if missing per T014/T028 in plan. Files: `DESIGN-SONA/*`

**Checkpoint**: US3 host leg green, bundle serves restyled surface.

---

## Phase 6: User Story 4 - Lente Sinestésica estática (Priority: P3)

**Goal**: Concha Harmônica "R" header, static metadata card, static surfaces only, no RMS/buffer ring, no excluded screens reachable.
**Independent Test**: SC-006 host checks green.

### Implementation for User Story 4
- [X] T023 [P] [US4] Add Concha Harmônica "R" header to `status_view.html` using `DESIGN-SONA/assets/logo/logo-primary-concha-r.svg` (no jade/celeste paints). Files: `src-tauri/src/status_view.html`, `DESIGN-SONA/assets/logo/logo-primary-concha-r.svg`
- [X] T024 [US4] Add static metadata card with `--sona-stage` background, ensure no reactive elements. Files: `src-tauri/src/status_view.html`
- [X] T025 [US4] Verify no navigation to excluded screens (Cockpit Eyes-Free, Console Braun & Rams, Sumário Editorial Suíço, Matriz de Telemetria). Files: `src-tauri/src/status_view.html`, `src-tauri/src/status_view.rs`

**Checkpoint**: US4 static Lente present.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [X] T026 Update `scripts/verify.sh` to include surface-audit and earcon-mapping checks (fail-closed). Update `scripts/expected-tests.txt` with new tests. Files: `scripts/verify.sh`, `scripts/expected-tests.txt`
- [X] T027 Run full host verification: `cargo test`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`, `scripts/verify.sh`. Fix issues. Files: workspace
- [X] T028 Property tests pass: `cargo test` for surface_audit, earcon burst/latency. Files: `src-tauri/`, `crates/player/`
- [X] T029 Device trial sheets prepared per quickstart §3 (SC-001/002/003/006/007) — deferred execution noted. Files: `specs/005-adopt-design-sona/quickstart.md`

**Checkpoint**: Host gates green (where applicable). Device trials ready.

---

## Phase 8: Verification (Attestation)

**Purpose**: Only these tasks attest completeness.

- [X] T030 [VERIFY] All host checks pass: surface-audit (C-H1..C-H8), earcon-mapping, contrast, aria, movement, accent, latency. Evidence in test output. Files: all modified
- [X] T031 [VERIFY] Bundle identity: served `dist/index.html` byte-identical to `src-tauri/src/status_view.html`. Files: `dist/index.html`, `src-tauri/src/status_view.html`
- [X] T032 [VERIFY] No `_ =>`, no `unwrap/expect/panic/todo/dbg/print_*` violations introduced. `cargo clippy` clean. Files: modified files
- [X] T033 [VERIFY] Update STATE.md and prepare for `/speckit.converge` with evidence summary. Files: `.specify/memory/STATE.md`

- [X] T034 [P] [HIGH] Complete restyle of `src-tauri/src/status_view.html` per FR-008/SC-001 (10-hex closed set, role exclusivity `:focus-visible`, aria inventory, movement rules, reduced-motion freeze, solid indeterminate pill) (partial)
- [X] T035 [P] [HIGH] Verify and complete earcon burst/truncation policy in `crates/player/src/backend.rs` per FR-003/SC-002 (latest wins, emit `EarconTruncation`/`EarconEmissionFailure`, ≤100ms latency) with passing property tests (partial)
- [X] T036 [P] [HIGH] Verify MediaStyle constraints in `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` per FR-002/SC-003 (accent-only `#F2F0E6`, title/section/progress only, no body/rate/brand) (partial)
- [X] T037 [MEDIUM] Add Concha Harmônica "R" header to `src-tauri/src/status_view.html` per FR-004/US4 (no jade/celeste paints) (partial)
- [X] T038 [MEDIUM] Ensure served bundle identity passes: verify `dist/index.html` byte-identical to `src-tauri/src/status_view.html` via surface-audit test (partial)
**Done When**: T030–T033 complete, host verification green, device trial plan ready.

---

## Dependencies & Execution Order

### Story Dependencies
- US1 (notification): independent after Phase 2
- US2 (earcons): independent after Phase 2  
- US3 (in-app tokens): depends on Phase 2 (audit infrastructure + bundle wiring)
- US4 (static Lente): depends on US3

### Parallel Opportunities
- Phase 2 tests T005,T007-T011 can run in parallel [P]
- Phase 3/4 tests can run in parallel [P]
- US3/US4 implementation can be parallelized where files disjoint

### MVP Scope
**MVP = US1 + US2** (notification accent + earcon confirmations) — core driving-safety features. US3/US4 add in-app visual identity.

---

## Notes
- Tests for SC-001/SC-002/SC-007 already exist in branch `pr/005-sc001-sc002-sc007-tests` (`8a4dd32e`) — integrate and ensure they pass.
- Device legs (SC-002/003/006/007 trials) are deferred per plan; host verification is required.
- All changes must follow closed-set discipline (§VII). No catch-alls.
- Commit via `scripts/commit-hook.sh` after each file edit.

## Phase 9: Convergence

- [X] T034 [P] [HIGH] Complete restyle of `src-tauri/src/status_view.html` per FR-008/SC-001 (10-hex closed set, role exclusivity `:focus-visible`, aria inventory, movement rules, reduced-motion freeze, solid indeterminate pill) (partial)
- [X] T035 [P] [HIGH] Verify and complete earcon burst/truncation policy in `crates/player/src/backend.rs` per FR-003/SC-002 (latest wins, emit `EarconTruncation`/`EarconEmissionFailure`, ≤100ms latency) with passing property tests (partial)
- [X] T036 [P] [HIGH] Verify MediaStyle constraints in `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` per FR-002/SC-003 (accent-only `#F2F0E6`, title/section/progress only, no body/rate/brand) (partial)
- [X] T037 [MEDIUM] Add Concha Harmônica "R" header to `src-tauri/src/status_view.html` per FR-004/US4 (no jade/celeste paints) (partial)
- [X] T038 [MEDIUM] Ensure served bundle identity passes: verify `dist/index.html` byte-identical to `src-tauri/src/status_view.html` via surface-audit test (partial)


## Phase 10: Convergence

- [X] T039 [VERIFY] Verify SC-001 host checks pass (C-H1..C-H6): chroma census 10-hex, role exclusivity, contrast WCAG 2.1 AA, aria inventory, movement census, reduced-motion freeze. Run `cargo test` for surface_audit tests. Files: `src-tauri/src/surface_audit/*`, `src-tauri/src/status_view.html`
- [X] T040 [VERIFY] Verify SC-002 host checks pass (C-H7/C-H8): earcon burst latest-wins property tests and latency oracle ≤100ms. Run `cargo test -p player`. Files: `crates/player/src/backend.rs`, `crates/player/tests/*`
- [X] T041 [VERIFY] Verify SC-003/US1: MediaStyle accent-only constraints pass (`surface_audit_accent_only_constants`). Files: `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt`, `src-tauri/src/surface_audit/accent.rs`
- [X] T042 [VERIFY] Verify US4/SC-006: Concha Harmônica "R" header present, static metadata card with `--sona-stage`, no excluded screens reachable. Files: `src-tauri/src/status_view.html`
- [X] T043 [VERIFY] Verify bundle identity: `dist/index.html` byte-identical to `src-tauri/src/status_view.html` and served-identity test passes. Files: `src-tauri/tauri.conf.json`, `src-tauri/build.rs`, `src-tauri/src/surface_audit/served_identity.rs`
- [X] T044 [VERIFY] Run full host verification gates 1-8: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace` (matches `scripts/expected-tests.txt`), `./scripts/verify.sh`. Update `scripts/expected-tests.txt` if needed. Files: workspace
