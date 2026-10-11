# Tasks: Reference Device Realignment

**Input**: Design documents from `specs/003-reference-device/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/amendment-record.md, quickstart.md
**Tests**: Spec-requested independent tests are doc-consistency checks (grep/read per contract). Each story's checks are run FIRST (must FAIL on current text), then re-run after edits (must PASS) — RED→GREEN for docs.
**Organization**: Tasks grouped by user story. Docs-only amendment; no code, no deps, no constitution change.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2)
- Exact file paths in every description

## Phase 1: Setup (Baseline Evidence)

**Purpose**: Capture pre-edit baseline so RED is observed, not assumed

- [X] T001 Record baseline evidence in a scratch note (not committed): current `specs/001-android-media-shell/plan.md` Reference-device paragraph verbatim, both ≤3 s bound sites verbatim, `specs/002-fr-024-synthesis-adapter/spec.md:117` divergence note verbatim, and `git status --short` output

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Confirm contract sites still match the files before any edit lands

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T002 Verify all four contract sites in `specs/003-reference-device/contracts/amendment-record.md` still match current file text (Site A/B in `specs/001-android-media-shell/plan.md`, Site C in `specs/002-fr-024-synthesis-adapter/spec.md`); if any site drifted, STOP and update the contract before editing

**Checkpoint**: Contract confirmed current — user story edits can now proceed

---

## Phase 3: User Story 1 — Gate verdicts measured on the phone in hand (Priority: P1) 🎯 MVP

**Goal**: `001/plan.md` names Redmi 2201117TY as the normative reference device and reconciles both device-coupled bounds to ≤10 s cold-start (FR-001, FR-002)

**Independent Test**: Open `specs/001-android-media-shell/plan.md`, confirm the Reference-device paragraph names Redmi 2201117TY specs; run `grep` checks per contract Sites A/B; SC-002 re-judgment of the 9 s × 5 measurement recorded un-asterisked

### Checks for User Story 1 (run FIRST — must FAIL before edits) ⚠️

- [X] T003 [US1] Run contract Site A/B checks against `specs/001-android-media-shell/plan.md` and confirm RED: `grep -c "A78"` ≥ 1 in reference-device meaning, `grep "2201117TY"` → 0 hits, bound sites still read ≤3 s

### Implementation for User Story 1

- [X] T004 [US1] Rewrite the Reference-device paragraph in `specs/001-android-media-shell/plan.md` per contract Site A (FR-001: Redmi 2201117TY, Snapdragon 680 `bengal`, 4× A73 + 4× A53, ~5.8 GB RAM, API 33; drop A78 class and slower-device-dismissal rule)
- [X] T005 [US1] Reconcile both device-coupled bound sites in `specs/001-android-media-shell/plan.md` per contract Site B (FR-002: Testing latency site and Performance Goals site → ≤10 s cold-start; leave ≤1 s / ±5 s / ≤2 s device-independent bounds untouched) — same file as T004, run after T004
- [X] T006 [US1] Record the SC-002 re-judgment of the 9 s × 5 cold-start measurement against the ≤10 s bound in the amendment commit message and (if a verdict log owns it) as an explicit follow-up statement — never by editing old measurement rows in `specs/002-fr-024-synthesis-adapter/SC-009-GATE-RECORD.md`
- [X] T007 [US1] Re-run contract Site A/B checks against `specs/001-android-media-shell/plan.md` and confirm GREEN: `grep -c "A78"` → 0 for reference-device meaning, `grep "2201117TY"` → ≥1, both bound sites read ≤10 s

**Checkpoint**: At this point, US1 is complete — one reference device named, bounds reconciled, verifiable by grep alone

---

## Phase 4: User Story 2 — No silent drift between the two specs (Priority: P2)

**Goal**: The 002 "Divergence noted" annotation is closed by follow-up note and the amendment is recorded with maintainer acceptance (FR-004, FR-003)

**Independent Test**: Grep both specs for the reference-device definition and confirm one story; confirm the 002 annotation is resolved/superseded by note with commit pointer

### Checks for User Story 2 (run FIRST — must FAIL before edit) ⚠️

- [X] T008 [P] [US2] Confirm RED for divergence closure: `specs/002-fr-024-synthesis-adapter/spec.md` around line 117 has the "Divergence noted" sentence with no closure note below it

### Implementation for User Story 2

- [X] T009 [US2] Append the divergence-closure follow-up note below the intact annotation in `specs/002-fr-024-synthesis-adapter/spec.md` per contract Site C (FR-004: condition no longer holds + 001 amendment commit hash + date; original sentence verbatim above it)
- [X] T010 [US2] Commit the amendment per contract Site D (FR-003: message carries maintainer acceptance, date 2026-10-11, rationale, preserved-history pointer, `d5d6b45` precedent shape; diff touches only contracted sites plus `specs/003-reference-device/` artifacts)

**Checkpoint**: At this point, US1 AND US2 are complete — specs agree, history preserved, audit trail closed

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Full validation and repo hygiene

- [X] T011 [P] Run all four `specs/003-reference-device/quickstart.md` scenarios end-to-end and record results (SC-001 one-device grep, SC-002 re-judgment presence, SC-003 no-hypothetical-class grep, Site D commit-message check)
- [X] T012 Confirm docs-only scope: `git status --short` shows changes exclusively in `specs/001-android-media-shell/plan.md`, `specs/002-fr-024-synthesis-adapter/spec.md`, and `specs/003-reference-device/` — no crate, gate script, or constitution file touched; no `verify.sh` gate behavior changed

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories
- **User Stories (Phase 3–4)**: Depend on Foundational; US1 (P1) first (MVP), US2 (P2) after — different files (`001/plan.md` vs `002/spec.md`), so T009 can start in parallel with Phase 3 once T002 passes, but T010 (commit) requires both stories done
- **Polish (Phase 5)**: Depends on all user stories complete

### Within Each User Story

- Checks MUST run and FAIL before edits (T003 before T004/T005; T008 before T009)
- Same-file edits sequential (T004 → T005, both in `001/plan.md`)
- Re-run checks GREEN after edits (T007; T011 covers Site C)

### Parallel Opportunities

- T008 [P] (read-only check on `002/spec.md`) can run parallel with Phase 3 edits (different file)
- T011 [P] (quickstart validation) is independent of nothing pending — runs in Polish phase
- T004/T005 must NOT be parallel (same file)

---

## Parallel Example: User Story 1 + User Story 2 checks

```bash
# While US1 edits proceed in 001/plan.md, a second agent can independently:
Task: "T008 Confirm RED for divergence closure in specs/002-fr-024-synthesis-adapter/spec.md (read-only)"
# Different file, no dependency on T004/T005 — safe in parallel after T002.
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 (T001) + Phase 2 (T002)
2. Complete Phase 3 (T003 → T004 → T005 → T006 → T007)
3. **STOP and VALIDATE**: grep checks GREEN per contract Sites A/B
4. US1 alone delivers the normative fix; US2 is audit-trail closure

### Incremental Delivery

1. Setup + Foundational → contract confirmed
2. US1 → independently verifiable by grep (MVP)
3. US2 → closure note + amendment commit
4. Polish → quickstart all-green, scope confirmed docs-only

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to US1/US2 for traceability
- Checks fail before edits, pass after (docs RED→GREEN)
- Supersede-by-note everywhere; never rewrite prior text or past verdict rows
- Constitution: no amendment (fixes no device class, no numeric bound)
