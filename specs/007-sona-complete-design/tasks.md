# Tasks: SONA Complete Design (007)

**Input**: Design documents from `/specs/007-sona-complete-design/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: TDD enforced — RED phase first (tests written, must FAIL before implementation). Property-based tests (proptest) used for invariants where applicable.

**Organization**: Tasks grouped by user story (US1 visual design, US2 controls + reactive waveform, US3 tokens/contracts). Parallel tasks marked [P].

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: US1, US2, US3
- Exact file paths in descriptions

## Path Conventions

- `crates/player/src/` — Rust player crate
- `src-tauri/src/` — Tauri app (status_view.html, status_view.rs, lib.rs)
- `tests/` — host-side contract tests (Rust)
- `specs/007-sona-complete-design/contracts/` — contract amendments

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm toolchain, feature branch, and baseline green.

- [x] T070 Verify feature branch `007-sona-complete-design` checked out and working tree clean (`git status`)
- [ ] T071 [P] Verify baseline gates green: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`
- [x] T072 [P] Confirm `crates/player`, `src-tauri` compile: `cargo build -p player`, `cargo build -p src-tauri`.
- [ ] T073 [P] Review amended contracts in `specs/007-sona-complete-design/contracts/` and `data-model.md` for scope accuracy before implementation

**Checkpoint**: Baseline green; no regressions introduced by planning artifacts.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before any user story. BLOCKS all stories.

**⚠️ CRITICAL**: No user story work begins until this phase is complete.

- [x] T074 Add `rms: AtomicF32` field to `SampleQueue` in `crates/player/src/backend.rs` (lock-free store; no allocation on hot path); expose `rms()` getter. Property test: RMS ≥ 0 and ≤ 1.0 for any input block.
- [x] T075 [P] Compute RMS in `SampleQueue::consume_into` using a stack accumulator over consumed speech samples; store into `AtomicF32` via `store(rms, Ordering::Relaxed)`. No heap allocation. Unit test: known sine block → expected RMS.
- [x] T076 [P] Add `CpalBackend::rms()` accessor returning `f32` from the atomic store. Unit test: returns last computed value.
- [x] T077 Add dedicated throttled RMS IPC channel in `src-tauri/src/lib.rs`: `tauri::ipc::Channel<f32>`; publish at ~20 Hz with last-wins coalescing; skip publishes while `document.hidden`. Unit test: coalescing drops intermediate values.
- [x] T078 [P] Add `#waveform` canvas element skeleton to `src-tauri/src/status_view.html` (aria-hidden, 388×80 CSS px default) — no rendering logic yet. Host test: element present in inventory.
- [x] T079 [P] Scaffold property-based test module for waveform invariants in `tests/waveform_props.rs` (proptest): layer count ∈ [4,6], per-layer opacity ∈ [0.1,0.5], opacity ramp monotonic non-decreasing back→front, DPR ∈ [1,2], fps cap ≥ 33.33 ms interval.
- [ ] T080 Create RED-phase host test file `tests/waveform_canvas_inventory.rs` asserting canvas element present, aria-hidden, and `#progress` retained (must FAIL until T078+). Mark blocking.
- [ ] T081 Create RED-phase host test file `tests/rms_channel.rs` asserting RMS field in mirror payload / dedicated channel (must FAIL until T077). Mark blocking.

**Checkpoint**: Foundation ready — RMS plumbing + IPC channel + canvas skeleton + RED tests in place. All RED tests must FAIL before proceeding.

---

## Phase 3: User Story 1 — Ver o design SONA completo (Priority: P1) 🎯 MVP

**Goal**: Decorative wave ribbons, glassmorphism title card, glow/neon, refined layout — all proposal elements visible on StatusView.

**Independent Test**: Open app, verify wave ribbons (teal→jade→violet), glassmorphism card, glow on logo + baseline, numeric rate, repeat button, no accessibility regressions.

### Tests for US1 (TDD RED phase — write FIRST, must FAIL before implementation) ⚠️

- [ ] T082 [P] [US1] Host test `tests/visual_ux_census.rs`: hex census over `status_view.html` allows jade/celeste only in sanctioned scopes (waveform ribbon draw + decorative wave); rate/title/notification/glow must not contain jade/celeste. Property test: for every hex occurrence, scope tuple ∈ allowlist.
- [ ] T083 [P] [US1] Host test `tests/glassmorphism_contract.rs`: assert `backdrop-filter: blur(24px)` present, overlay `rgba(20,27,45,0.65)`, Doppelrand borders, solid `--sona-stage` fallback. Must FAIL initially.
- [ ] T084 [P] [US1] Host test `tests/glow_scope.rs`: assert glow `box-shadow` only on header mark and waveform baseline with alpha 0.20 (band 0.15–0.25); no other elements. Must FAIL initially.
- [ ] T085 [P] [US1] Host test `tests/decorative_wave.rs`: assert `wave-subtle` one-shot ≤400 ms, base opacity 0.80, peak 0.90, reduced-motion `animation: none; opacity: 0.80`. Must FAIL initially.
- [ ] T086 [P] [US1] Host test `tests/overflow_empty_title.rs`: assert long-filename overflow rules and empty-state glyph `—` + `.sr-only` "Sem título". Must FAIL initially.

**Checkpoint (RED)**: All US1 tests written and FAILING. Do not proceed to implementation until confirmed failing.

### Implementation for US1

- [x] T087 [P] [US1] Implement decorative wave ribbons in `src-tauri/src/status_view.html` CSS: `.card::before` with `waves-top.svg`, opacity 0.80 base / 0.90 peak, `wave-subtle 400ms ease-in-out` one-shot, reduced-motion guard. (T082, T085)
- [x] T088 [P] [US1] Implement glassmorphism title card in `src-tauri/src/status_view.html`: `backdrop-filter: blur(24px) saturate(120%)`, overlay `rgba(20,27,45,0.65)`, Doppelrand borders, fallback solid `--sona-stage`. (T083)
- [x] T089 [P] [US1] Implement glow/neon in `src-tauri/src/status_view.html`: `box-shadow: 0 0 20px rgba(143,216,232,0.20)` on header mark, `0 0 10px rgba(232,196,106,0.20)` on waveform baseline. (T084)
- [x] T090 [US1] Wire title card binding in `src-tauri/src/status_view.rs`: `metadata_title.unwrap_or(filename)`, empty state `—` + `.sr-only`. (T086)
- [x] T091 [US1] Update `DESIGN-SONA/tokens.md` version bump + sanctioned scope notes for jade/celeste. (T082)
- [x] T092 [US1] Run US1 host tests; confirm all PASS (GREEN). Commit atomically with detailed summary.

**Checkpoint**: US1 fully functional and independently testable.

---

## Phase 4: User Story 2 — Controles completos e waveform reativo (Priority: P1)

**Goal**: Repeat control, numeric rate display, reactive RMS waveform (canvas smooth ribbons + integrated playhead), accessibility (aria-hidden canvas + hidden progressbar).

**Independent Test**: Verify all 5 controls present and functional; waveform reacts to playback state; repeat restarts current section; aria inventory valid; no accessibility regressions.

### Tests for US2 (TDD RED phase — write FIRST, must FAIL before implementation) ⚠️

- [ ] T093 [P] [US2] Host test `tests/rms_instrumentation.rs`: assert `AtomicF32` RMS in `SampleQueue::consume_into`, no allocation on hot path, known sine block → expected RMS. Property test: RMS ∈ [0,1] for any input. Must FAIL initially.
- [ ] T094 [P] [US2] Host test `tests/rms_channel.rs`: assert dedicated throttled channel ~20 Hz, last-wins coalescing, skip while hidden. Must FAIL initially.
- [ ] T095 [P] [US2] Host test `tests/waveform_canvas.rs`: assert canvas `aria-hidden="true"`, no `role="img"`, smooth ribbons (5 layers, bezier CP, opacity ramp), integrated playhead (center line + knob), no knob in indeterminate. Must FAIL initially.
- [ ] T096 [P] [US2] Host test `tests/progress_integration.rs`: assert `#progress` sr-only `role="progressbar"` with `aria-valuenow/min/max/valuetext`, updated by `renderProgress()`; live region populated on ≥5% delta or state transition. Must FAIL initially.
- [ ] T097 [P] [US2] Host test `tests/repeat_control.rs`: assert repeat button present, `aria-label="Repetir seção"`, restarts current section, earcon = SkipBack (Allegro-trás), haptic id 4. Must FAIL initially.
- [ ] T098 [P] [US2] Host test `tests/numeric_rate.rs`: assert painted numeric rate `1.25×` in speed-control area only; `aria-valuetext` retains positional descriptor. Must FAIL initially.
- [ ] T099 [P] [US2] Host test `tests/waveform_performance.rs`: assert 30 fps cap (33.33 ms gate), pause on `visibilitychange`, freeze under reduced motion, DPR cap 2×, static placeholder fallback, 0 heap allocations per redraw. Must FAIL initially.
- [ ] T100 [P] [US2] Property-based test `tests/waveform_props.rs` (proptest): layer count ∈ [4,6]; per-layer opacity ∈ [0.1,0.5] and monotonic non-decreasing; DPR ∈ [1,2]; fps interval ≥ 33.33 ms; RMS ∈ [0,1]. Must FAIL initially.

**Checkpoint (RED)**: All US2 tests written and FAILING. Do not proceed to implementation until confirmed failing.

### Implementation for US2

- [x] T101 [P] [US2] Implement RMS instrumentation in `crates/player/src/backend.rs`: `AtomicF32` store in `consume_into` (stack accumulator), `rms()` accessor. (T093)
- [x] T102 [P] [US2] Implement throttled RMS IPC channel in `src-tauri/src/lib.rs` (~20 Hz, last-wins coalescing, skip while hidden). (T094)
- [x] T103 [P] [US2] Implement canvas waveform in `src-tauri/src/status_view.html`: 5-layer smooth ribbons (bezier CP at 1/3 & 2/3, per-layer opacity 0.15→0.35, mirrored, teal→jade→violet), white center line + 6 px playhead, `aria-hidden="true"`, IPC-event-driven redraw, 30 fps gate, pause on `visibilitychange`, freeze under reduced motion, DPR cap 2×, static placeholder fallback. (T095, T099, T100)
- [x] T104 [P] [US2] Implement hidden progressbar aria in `src-tauri/src/status_view.html`: sr-only `#progress` with `role="progressbar"`, update `aria-valuenow`/`aria-valuetext` in `renderProgress()`; populate `#liveRegion` on ≥5% delta or state transition. (T096)
- [x] T105 [P] [US2] Implement repeat button in `src-tauri/src/status_view.html`: `<button id="repeat">`, `aria-label="Repetir seção"`, tab order, focus ring; wire `repeat` action in `src-tauri/src/lib.rs` → restart current section + `EarconKind::SkipBack` + haptic id 4. (T097)
- [x] T106 [P] [US2] Implement numeric rate display in `src-tauri/src/status_view.html`: text element `1.25×` in speed-control area next to dots; `aria-valuetext` retains positional descriptor. (T098)
- [x] T107 [US2] Expose RMS field in `crates/player/src/backend.rs` and `src-tauri/src/status_view.rs` via the dedicated channel (T101, T102); no "if needed" — required by FR-008 IPC contract.
- [x] T108 [US2] Run US2 host tests; confirm all PASS (GREEN). Commit atomically with detailed summary.

**Checkpoint**: US2 fully functional and independently testable.

---

## Phase 5: User Story 3 — Tokens expandidos e contratos atualizados (Priority: P1)

**Goal**: Token set expansion (jade/celeste promoted with sanctioned scopes), contract amendments per §VII, surface inventory 9→10, census/audit updates.

**Independent Test**: Surface inventory valid (10 elements); no dead corpses; notification accent-only; chroma census scoped; contracts updated.

### Tests for US3 (TDD RED phase — write FIRST, must FAIL before implementation) ⚠️

- [ ] T109 [P] [US3] Host test `tests/chroma_scoped.rs`: assert celeste/jade appear only in sanctioned scopes (waveform ribbon draw code + decorative wave CSS/SVG); rate/title/notification/glow must not contain them. Property test: every hex occurrence's scope tuple ∈ allowlist. Must FAIL initially.
- [ ] T110 [P] [US3] Host test `tests/surface_inventory_count.rs`: assert `data-elements="10"` and 10-element inventory present; no `display:none` corpses. Must FAIL initially.
- [ ] T111 [P] [US3] Host test `tests/aria_inventory_007.rs` + `tests/notification_purity.rs`: assert canvas `aria-hidden`, `#progress` sr-only progressbar, live region, no `role="img"`, notification accent-only (MediaStyle block contains only `#F2F0E6`, no structural change). Must FAIL initially.
- [ ] T112 [P] [US3] Host test `tests/movement_census_007.rs`: assert ≤1 continuous loop (pill only), no new `@keyframes` for waveform, reduced-motion guard on canvas. Must FAIL initially.
- [ ] T113 [P] [US3] Host test `tests/byte_identity_007.rs`: assert SHA-256 of `status_view.html` == served bundle hash + file size. Must FAIL initially.

**Checkpoint (RED)**: All US3 tests written and FAILING. Do not proceed to implementation until confirmed failing.

### Implementation for US3

- [ ] T114 [P] [US3] Update `src-tauri/src/surface_audit/chroma.rs` and `tests/contract_census.rs` with scope-aware allowlist for jade/celeste sanctioned scopes. (T109)
- [ ] T115 [P] [US3] Update `src-tauri/src/status_view.html` `data-elements` from 9 to 10. (T110)
- [ ] T116 [P] [US3] Update inventory/aria test expectations in `tests/` for 10 elements + new canvas/repeat. (T110, T111)
- [ ] T117 [P] [US3] Update `DESIGN-SONA/tokens.md` (bump to v2.3.0, sanctioned scope notes) + `DESIGN-SONA/VERSION` + `CHANGELOG.md` (record jade/celeste promotion and sanctioned scopes). (T109)
- [ ] T118 [US3] Implement scope-aware census in `src-tauri/src/surface_audit/chroma.rs` + `tests/contract_census.rs` per amended contracts; verify consistency between code and `contracts/status-surface.md` / `contracts/surface-audit.md`. (T109–T113)
- [ ] T119 [US3] Run US3 host tests; confirm all PASS (GREEN). Commit atomically with detailed summary.

**Checkpoint**: US3 fully functional and independently testable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Verification tasks that attest to success and completeness. ONLY these attest.

- [ ] T120 [P] Run full host gate suite: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo test --doc`, `cargo deny check advisories licenses bans sources && cargo audit`.
- [ ] T121 [P] Run surface-audit checks: chroma (scoped), inventory (10), aria, movement, byte-identity, RMS channel, canvas performance.
- [ ] T122 [P] Run `scripts/verify.sh` (gates 1–8) if available; report any DEFERRED (owner=device-build) items.
- [ ] T123 Device trial (gate 9): build `cargo tauri android build --target aarch64`; install on API-29+ device; verify all proposal elements, reduced-motion, notification purity, repeat, waveform reactivity, glassmorphism fallback, canvas pause/freeze. Record 10/10 trial results.
- [ ] T124 Run quickstart.md validation scenarios (SC-001..SC-006) end-to-end.
- [ ] T125 [P] Documentation: update `DESIGN-SONA/VERSION`, `CHANGELOG.md`, constitution sync-impact report if version bump applied.
- [ ] T126 Final atomic conventional commit with detailed summary of all edits (per deterministic hook rule).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies.
- **Phase 2 (Foundational)**: Depends on Phase 1; BLOCKS all user stories.
- **Phase 3 (US1)**: Depends on Phase 2.
- **Phase 4 (US2)**: Depends on Phase 2; can run in parallel with US1 after foundational (different files mostly; shared `status_view.html` requires coordination).
- **Phase 5 (US3)**: Depends on Phase 2; can run in parallel with US1/US2 (contract/test files disjoint from HTML implementation mostly).
- **Phase 6 (Polish)**: Depends on all desired user stories complete.

### User Story Dependencies

- **US1 (P1)**: Independent after Foundational.
- **US2 (P1)**: Independent after Foundational; shares `status_view.html` with US1 → coordinate edits (reuse same subagent for HTML changes).
- **US3 (P1)**: Independent after Foundational; touches contract/test files, disjoint from HTML implementation.

### TDD Blocking Rule

- Within each user story, RED-phase test tasks (T082–T086, T093–T100, T109–T113) MUST be written and FAILING before any implementation task (T087+, T101+, T114+) begins.
- Property-based tests (proptest) used for invariants: RMS ∈ [0,1], layer count ∈ [4,6], opacity monotonic, DPR ∈ [1,2], fps interval ≥ 33.33 ms, sanctioned scope enforcement.

### Parallel Opportunities

- Phase 1: T071, T072, T073 in parallel.
- Phase 2: T075, T076, T078, T079 in parallel (T074, T077 depend on them).
- Phase 3: T082–T086 in parallel (RED); T087–T089 in parallel (impl, disjoint CSS rules but same file → coordinate); T090, T091 parallel.
- Phase 4: T093–T100 in parallel (RED); T101–T106 in parallel (impl, disjoint concerns); T107 depends on T101–T102.
- Phase 5: T109–T113 in parallel (RED); T114–T117 in parallel (impl).
- Phase 6: T120, T121, T122, T125 in parallel; T123, T124 sequential (device/manual).

### Parallel Example: User Story 2

```bash
# RED phase (all parallel, different test files):
Task: "T093 Host test rms_instrumentation.rs"
Task: "T094 Host test rms_channel.rs"
Task: "T095 Host test waveform_canvas.rs"
Task: "T096 Host test progress_integration.rs"
Task: "T097 Host test repeat_control.rs"
Task: "T098 Host test numeric_rate.rs"
Task: "T099 Host test waveform_performance.rs"
Task: "T100 Property test waveform_props.rs"

# Implementation phase (parallel, disjoint concerns):
Task: "T101 RMS instrumentation in backend.rs"
Task: "T102 Throttled IPC channel in lib.rs"
Task: "T103 Canvas waveform in status_view.html"
Task: "T104 Hidden progressbar aria in status_view.html"  # same file as T103 -> coordinate
Task: "T105 Repeat button in status_view.html"            # same file -> coordinate
Task: "T106 Numeric rate display in status_view.html"     # same file -> coordinate
```

**Note**: Tasks sharing `status_view.html` (T087–T090, T103–T106) MUST reuse the same subagent to avoid file conflicts. Use subagent-driven development in waves.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1 (visual design)
4. **STOP and VALIDATE**: Test US1 independently (host census + device)
5. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. US1 (visual design) → Test independently → Deploy/Demo (MVP!)
3. US2 (controls + reactive waveform) → Test independently → Deploy/Demo
4. US3 (tokens/contracts) → Test independently → Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:
1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: US1 (visual design, owns status_view.html CSS)
   - Developer B: US2 (RMS plumbing + canvas JS, coordinates on status_view.html)
   - Developer C: US3 (contracts/tests, disjoint files)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies (or disjoint concerns)
- [Story] label maps task to specific user story for traceability
- Each user story independently completable and testable
- Verify tests FAIL before implementing (RED phase)
- Commit after each task or logical group (atomic conventional commit per deterministic hook)
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same-file conflicts, cross-story dependencies that break independence
- All code generated by LSP/template scaffolds (e.g., proptest macros, tauri channel boilerplate) must be written via LSP/code actions before hand-writing
- Verification tasks (Phase 6) are the ONLY attestation of success/completeness
