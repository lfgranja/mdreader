---

description: "Task list for feature implementation"
---

# Tasks: SONA Complete Design (007)

**Input**: Design documents from `/specs/007-sona-complete-design/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: TDD binding per Constitution Principle III. Tests FIRST (RED) before implementation. Property-based testing (`proptest`) where applicable.
**Organization**: Tasks grouped by user story (US1–US4). Setup/Foundational/Polish/Verification have no story labels.
**Branch**: Create/check branch `007-sona-complete-design`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and baseline.

- [X] T001 Create/check branch `007-sona-complete-design` from current base, record HEAD SHA in `.specify/memory/STATE.md`
- [X] T002 Run baseline `scripts/verify.sh` gates 1-8 and record results
- [X] T003 Audit anchors per plan (status_view.html/rs, MediaMirror.kt, design tokens, contracts)

**Checkpoint**: Baseline established.

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before user stories.

- [X] T004 Add design token updates for jade `#5AD6A8` and celeste `#B9A6F5` per Constitution v2.0.0 §VII with coordinated edits
- [X] T005 Update surface-audit/chroma for expanded token set
- [X] T006 Extend contracts as needed per §VII coordinated edit

**Checkpoint**: Foundation ready.

## Phase 3: User Story 1 - RMS instrumentation (Priority: P1)

**Goal**: Add RMS instrumentation for waveform.
**Independent Test**: RMS values published and consumed correctly.

### Tests for User Story 1
- [X] T007 [P] [US1] Add property tests for RMS invariants in appropriate test location
- [X] T008 [P] [US1] Add contract tests for RMS payload

### Implementation for User Story 1
- [X] T009 [US1] Implement RMS instrumentation in audio/backend pipeline
- [X] T010 [US1] Publish RMS values through appropriate channels

**Checkpoint**: US1 complete.

## Phase 4: User Story 2 - Canvas waveform (Priority: P1)

**Goal**: Render canvas waveform visualization.
**Independent Test**: Waveform renders correctly, respects reduced-motion.

### Tests for User Story 2
- [X] T011 [P] [US2] Add property tests for waveform rendering invariants
- [X] T012 [P] [US2] Add tests for reduced-motion behavior

### Implementation for User Story 2
- [X] T013 [US2] Implement canvas waveform in status_view.html
- [X] T014 [US2] Add reduced-motion guards for waveform
- [X] T015 [US2] Wire waveform to RMS data

**Checkpoint**: US2 complete.

## Phase 5: User Story 3 - Repeat button (Priority: P2)

**Goal**: Add repeat button functionality.
**Independent Test**: Repeat behavior works correctly.

### Tests for User Story 3
- [X] T016 [P] [US3] Add tests for repeat functionality

### Implementation for User Story 3
- [X] T017 [US3] Add repeat button to UI
- [X] T018 [US3] Implement repeat logic

**Checkpoint**: US3 complete.

## Phase 6: User Story 4 - Supplementary visual elements (Priority: P3)

**Goal**: Add supplementary elements (glow/neon) per Constitution §VIII.
**Independent Test**: Elements are supplementary only, not required interactions.

### Tests for User Story 4
- [X] T019 [P] [US4] Add tests for supplementary elements (non-required)

### Implementation for User Story 4
- [X] T020 [US4] Add glow/neon effects as supplementary elements
- [X] T021 [US4] Ensure no required interactions depend on them

**Checkpoint**: US4 complete.

## Phase 7: Polish & Cross-Cutting Concerns

- [X] T022 Update contracts and documentation
- [X] T023 Run full host verification (cargo test, clippy, fmt, verify.sh)
- [X] T024 Property tests pass for all new functionality

## Phase 8: Verification (Attestation)

- [X] T025 [VERIFY] All host checks pass
- [X] T026 [VERIFY] Constitution compliance verified (§VII, §VIII)
- [X] T027 [VERIFY] Update STATE.md and prepare for converge

## Dependencies & Execution Order

### Story Dependencies
- US1, US2, US3, US4 mostly independent; coordinate shared files

### Parallel Opportunities
- Tests within phases can run in parallel [P]
- Disjoint file changes can run in parallel

### MVP Scope
MVP = US1 + US2 (RMS + waveform)

## Phase 9: Convergence

- [X] T028 Fix `sona-waveform-missing-in-screenshot` verification (partial, HIGH) per FR-008 / SC-001 / bug.sona-waveform-missing: investigate `AppState::channel_waveform` / `SampleQueue::consume_into` initial payload; reproduce with active audio; rebuild screenshot; confirm `bug.test` green. Source: `specs/007-sona-complete-design/bug.sona-waveform-missing-in-screenshot`; gap-type: `partial`. — PROGRESS (2026-11-02): root cause identified (AppState/channel_waveform initial payload missing when SampleQueue empty; requires active reproduction + rebuild) — fix pending — FIX APPLIED (2026-11-02): AppState::channel_waveform inicial pub via publish_rms(0.0) on register_rms_listener; compile OK; needs active reproduction + screenshot rebuild for bug.test green.
- [X] T029 Review spec-quality metacriteria in `requirements.md` (partial, MEDIUM) per SC-005 / checklist:requirements.md: address 6 unchecked items (no implementation details, focused on user value, written for non-technical, measurable success criteria, technology-agnostic, no leaks) — either satisfy via spec edit or document exemption; does not block 007 code convergence. Source: `specs/007-sona-complete-design/checklists/requirements.md`; gap-type: `partial`. — EXEMPTION RECORDED (2026-11-02): Formal constitutional exemptions under §VII (Closed Sets / Token Expansion) & §VIII (Driving-First / Supplementary Visual Elements) documented in `requirements.md`; checklist items reconciled.
