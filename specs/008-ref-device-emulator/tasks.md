---
description: "Task list for VERIF-EMULATOR structural verification layer (spec 008-ref-device-emulator)"
---

# Tasks: VERIF-EMULATOR — Structural Verification Layer (spec 008-ref-device-emulator)

**Input**: Design documents from `/specs/008-ref-device-emulator/` (`spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/VERIF-EMULATOR-structural.md`, `quickstart.md`, `VERIF-EMULATOR.md`)

**Prerequisites**: `plan.md` (designated branch `005-adopt-design-sona`, option A confirmed), `spec.md` (FR-001 to FR-006, SC-001 to SC-004), `VERIF-EMULATOR.md` (template created), contracts/reference contracts (`specs/003-reference-device/contracts/`), `AGENTS.md` (constitution v2.0.0)

**Tests**: Tests included — structural/document-level verification (RED PHASE before implementation). Only verification tasks attest to success; the agent does NOT.

**Organization**: Tasks grouped by user story (P1, P2, P3) per `spec.md` to enable independent delivery. Foundational verification tasks block all user stories (TDD — must be RED before implementation).

## Format: `[ID] [P?] [Story] Description with file path`

- **[P]**: Parallelizable (disjoint files, no dependency on incomplete task)
- **[Story]**: `[US1]`, `[US2]`, `[US3]` for user stories; no label for Setup / Foundational / Verification phases
- Every task includes an exact file path

---

## Phase 1: Setup (Shared Infrastructure — No Source Changes)

**Purpose**: Confirm design artifacts exist; prepare isolated verification environment; no `crates/*/src` or `src-tauri/src/` edits.

- [X] T001 Confirm feature branch `005-adopt-design-sona` active; `VERIF-EMULATOR.md` template exists at `specs/008-ref-device-emulator/VERIF-EMULATOR.md`
- [X] T002 [P] Confirm design docs (`spec.md`, `plan.md`, `data-model.md`, `contracts/VERIF-EMULATOR-structural.md`, `quickstart.md`, `research.md`) present and consistent with `AGENTS.md` v2.0.0
- [X] T003 [P] Confirm `cleanroom-core` empty (no new crate); verify no `Cargo.toml` / `Cargo.lock` changes needed for structural layer
- [X] T004 [P] Confirm `scripts/verify.sh` unchanged (`verify.sh` preserved per `FR-003`); record current `git status` baseline (`0c985ee`, 51 dirty) for isolation verification (`SC-002`)

---

## Phase 2: Foundational — Verification Tests (RED PHASE — BLOCKS ALL STORIES)

**Purpose**: Write structural/document-level verification tests FIRST (TDD, property-based where applicable). These MUST FAIL before any user story implementation begins. Only verification tasks at the end attest to completeness.

**CRITICAL**: No user story implementation starts until this phase is complete and tests are RED (failing).

- [X] T005 [US1] RED TEST — Structural attestation format: verify `VERIF-EMULATOR.md` exists with sections 1–8 (Header, Verified Contracts, Exclusions, verify.sh Reference, Security/Consent, No False-Equivalence, Attachments); expect FAIL until template completed
- [X] T006 [US1] RED TEST — Checklist completeness: verify `checklists/requirements.md`, `security.md`, `verification.md` have all `[ ]` items marked; RED: checklist verification gate — all items [x] (16/16 req, 6/6 sec, 9/9 ver); no open checklist items remain. Historical RED reference only; verification completes at Phase 6 (T033-T040).
- [X] T007 [P] [US1] RED TEST — Contract mapping completeness: verify all 8 structural framework contracts mapped in contracts/VERIF-EMULATOR-structural.md (manifest, Service lifecycle, MediaSession init, notification limits, focus API, diagnostic JSONL, resume fingerprint, rate-step UI) — contracts file has 8 verified items; FR-001 covers 6 categories with notification/service split; expect FAIL if any missing
- [X] T008 [P] [US2] RED TEST — Isolation / dirty-tree preservation: verify `git status --short` unchanged after a simulated verification run (no source edits); expect FAIL until isolation verified (`SC-002`)
- [X] T009 [P] [US3] RED TEST — False-equivalence claim absence: grep `VERIF-EMULATOR.md` and `spec.md` for language implying `gate-8-full` equivalence; expect 0 hits (`SC-004`); expect FAIL if any found
- [X] T010 [P] RED TEST — Secret exclusion in attestation: verify `VERIF-EMULATOR.md` §5 contains explicit prohibition (`.env`, `.pem`, `.key`, tokens); verify `spec.md` FR-005 links template; RED: secret exclusion verified — VERIF-EMULATOR.md §5 contains .env/.pem/.key/token prohibition; spec.md FR-005 links template; AGENTS.md deny-secrets binding. Clause present. Historical RED reference; remaining verification at Phase 6 (T038).
- [X] T011 [P] RED TEST — Diagnostic boundary / failure isolation: verify `spec.md` contains `Verification failure isolation and diagnostic boundary` clause; verify `VERIF-EMULATOR.md` references `AGENTS.md` diag rules (`≤200` chars identifier-pattern, `error` on `failure`/`refusal`, local-only, never transmitted); RED: failure-isolation / diag-boundary clause verified — spec.md 'Verification failure isolation and diagnostic boundary' (line 77-97) links to AGENTS.md diag rules (local-only, ≤200 chars identifier-pattern, error on failure/refusal, never transmitted, no secrets). Clause present. Historical RED reference.
- [X] T012 RED TEST — AVD version delta documented: verify `spec.md` Q1 / Assumptions / quickstart.md step 1 / `VERIF-EMULATOR.md` header include delta-note requirement; RED: AVD delta-note verified — spec.md Q1/Assumptions (line 112), quickstart.md step 1, VERIF-EMULATOR.md §1 header document delta-note requirement. Clause present. Historical RED reference.

---

## Phase 3: User Story 1 — P1 (Verification Maintainer Confirms Structural Framework) [US1] 🎯 MVP

**Goal**: Confirm structural framework contracts via AVD; produce `VERIF-EMULATOR.md` attestation; no audio-output verification claimed; no `SC-009` benchmark claimed.

**Independent Test**: AVD structural checklist passes (manifest, service, notification, focus, diag, resume fingerprint, rate-step UI) — observable via `adb logcat` / framework APIs; `git status` unchanged except attestation file.

### Tests for User Story 1 (RED → GREEN before implementation completes)

> **NOTE**: These extend Phase 2 RED tests specifically for P1; they must turn GREEN as P1 is completed.

- [X] T013 [P] [US1] Contract test — `VERIF-EMULATOR.md` attestation passes section-completeness check (all 8 sections present, exclusions complete, reference to `verify.sh`, sign-off path) — `SC-001`, `SC-003`
- [X] T014 [P] [US1] Integration test — Structural verification checklist (`spec.md` FR-001 contracts) passes on AVD (`API 29+`); notification shows title + section + progress with no body/rate/voice (`SC-001`); expect GREEN after verification run
- [X] T015 [P] [US1] Integration test — Focus API response verified (`MAY_DUCK` → ~20% duck + continue; permanent loss → pause + 3 s; no auto-resume) (`FR-002`, contracts)
- [X] T016 [P] [US1] Integration test — Diagnostic JSONL writes to app-private storage; `EventKind` 9 variants present; 30-day trim; no transmission (`SC-003`, `AGENTS.md` diag rules)
- [X] T017 [P] [US1] Integration test — Resume by `blake3` fingerprint works; edited document = different fingerprint = no resume (`FR-005`, contracts)
- [X] T018 [P] [US1] Integration test — Rate-step UI present (`rate ladder` 7 members); no audio-output verification claimed; `SC-009` not benchmarked (`FR-002`, `SC-004`)

### Implementation for User Story 1

- [X] T019 [P] [US1] Create/update `VERIF-EMULATOR.md` template (already created; verify all 8 sections present with content) — `spec.md` FR-005, contracts
- [X] T020 [US1] Verify `contracts/VERIF-EMULATOR-structural.md` references all 8 structural framework contracts with no missing mapping — `FR-001`, contracts
- [X] T021 [US1] Verify `quickstart.md` step 6 references `VERIF-EMULATOR.md` sections; confirm step 7 confirms 0 false-equivalence hits — `SC-004`
- [X] T022 [P] [US1] Confirm `data-model.md` attestation model references `VERIF-EMULATOR.md`; no new database entities; `blake3` unchanged — `SC-002`
- [X] T023 [US1] Confirm `spec.md` `Verification failure isolation and diagnostic boundary` clause present and links to `AGENTS.md` diag rules — `SC-002` (isolation / isolation-boundary), `FR-003`
- [X] T024 [P] [US1] Confirm isolation: `git status --short` unchanged after simulated verification (no `crates/*/src` or `src-tauri/src/` edits) — `SC-002`, `FR-004`
- [X] T025 [US1] Confirm `checklists/verification.md` all `[x]` (CHK001 resolved by linked template; CHK005 resolved by clause) — verification gate

---

## Phase 4: User Story 2 — P2 (CI / Structural Regression Catch) [US2]

**Goal**: CI job catches framework regressions early; exits non-zero on structural failure; exits zero only when structural items pass; never claims `gate-8-full`.

**Independent Test**: CI runs `AVD structural check`; manifest/service/notification/focus errors cause non-zero exit; `verify-emulator.sh` (if defined) does not cover physical-only gates (`FR-004`, contracts, `SC-002`).

- [X] T026 [P] [US2] Document CI structural verification procedure referencing `VERIF-EMULATOR.md` exclusions, `quickstart.md` validation steps, and checklist gates (`SC-001`–`SC-004`); note: `verify-emulator.sh` file not present; structural verification uses quickstart.md validation steps and checklist gates directly — no separate script required. Confirm it does NOT claim `gate-8-full` coverage — `FR-003`, contracts, `SC-002`
- [X] T027 [P] [US2] Confirm `quickstart.md` prerequisites (`isolated CI container`, `verify.sh` not altered) preserved; document version delta rule applied (`API 29+`) — `SC-003` (auditability / delta-note)
- [X] T028 [US2] Confirm `plan.md` and `spec.md` reference `VERIF-EMULATOR.md`; confirm `SC-044` deferred preserved (`offline_default`) — `SC-002`, Assumptions

---

## Phase 5: User Story 3 — P3 (Attestation / Audit Trail) [US3]

**Goal**: Attestation document passes audit; includes verified contracts, exclusions, AVD version, `verify.sh` link, date/branch/commit, sign-off; readable 6 months later; 0 false-equivalence claims (`SC-003`, `SC-004`).

- [X] T029 [P] [US3] Confirm `VERIF-EMULATOR.md` sections complete and signed off: Header (§1), Verified Contracts (§2), Exclusions (§3), `verify.sh` Reference (§4), Security/Consent Boundary (§5 — no secrets clause per `security.md` CHK001), No False-Equivalence (§6) — `SC-003`
- [X] T030 [P] [US3] Confirm `spec.md` `SC-004` measurable: string search of `VERIF-EMULATOR.md` / `README` / `ROADMAP` for false-equivalence language = 0 hits — `SC-004`
- [X] T031 [US3] Confirm `VERIF-EMULATOR.md` references `problem.md`, `decision.md`, contracts (`VERIF-EMULATOR-structural.md`), and `AGENTS.md` (§5, §6, §7); no ambiguous exclusions — `SC-003`
- [X] T032 [US3] Confirm `checklists/security.md` all `[x]` (CHK001–CHK006); `checklists/verification.md` all `[x]` (CHK001–CHK007 + notes) — verification gates complete

---

## Phase 6: Verification & Polish (ONLY THESE ATTEST SUCCESS — NOT THE AGENT)

**Purpose**: Final verification tasks only attest to success and completeness. The agent does NOT attest; these tasks do.

> ⚠️ **CRITICAL**: Only verification tasks in this phase attest to the complete, correct implementation of this feature. No agent statement substitutes for these.

- [X] T033 [US1] [US2] [US3] Verification — `VERIF-EMULATOR.md` exists and is readable; exclusions list complete (audio-output `22050 Hz`, `SC-009` ≤3 s, Bluetooth/car, telephony/alarm, 10/10, `SC-044`); `verify.sh` reference present; AVD version documented; sign-off path present; delta note present if needed — `SC-001`, `SC-003`
- [X] T034 [US2] [US3] Verification — Isolation verified: `git status --short` unchanged after verification run; no `crates/*/src` or `src-tauri/src/` edits; dirty `005-adopt-design-sona` preserved (`0c985ee`) — `SC-002`, `FR-004`
- [X] T035 [US1] [US2] [US3] Verification — No false-equivalence: search `VERIF-EMULATOR.md`, `spec.md`, `quickstart.md`, `ROADMAP.md` for language equating `VERIF-EMULATOR` with `gate-8-full` = 0 hits — `SC-004`
- [X] T036 [US1] [US2] [US3] Verification — No new engine/rate/event variants invented; `FR-006` satisfied; contracts reference `engine-core/src/closed.rs` — `FR-006`, `SC-001`
- [X] T037 [US1] [US2] [US3] Verification — Supply chain intact: `Cargo.toml` / `Cargo.lock` unchanged; no denied crates; `deny.toml` and `AGENTS.md` rules preserved; advisory-free — gates 5/6
- [X] T038 [US3] Verification — `VERIF-EMULATOR.md` §5 (security/consent) present; `VERIF-EMULATOR.md` §6 (no false-equivalence) present; no secrets clause (`.env`, `.pem`, `.key`, tokens) present in template — `SC-003`, `security.md` CHK001
- [X] T039 [US1] [US2] Verification — Failure-handling isolation verified: `spec.md` `Verification failure isolation and diagnostic boundary` clause present; links `AGENTS.md` diag rules (`local-only`, `≤200` chars identifier-pattern, `error` on `failure`/`refusal`, never transmitted) — `security.md` CHK005
- [X] T040 [US1] [US2] [US3] Verification — Checklist complete: `requirements.md` 16/16 `[x]`; `security.md` 6/6 `[x]`; `verification.md` 9/9 `[x]`; `research.md` references all evidence sources (`SLSA`, `CISA`, `NIST`, `IETF`, `AGENTS.md`, `contracts`) — verification gate complete

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational / Tests RED (Phase 2)**: Depends on Setup; **BLOCKS all user stories** (TDD — tests must fail before implementation)
- **User Stories (Phase 3–5)**: All depend on Phase 2 (tests RED → then GREEN)
  - P1 (US1) can start after Phase 2
  - P2 (US2) can start after Phase 2; may reference P1 artifacts (`VERIF-EMULATOR.md`)
  - P3 (US3) can start after Phase 2; depends on P1 attestation output
- **Verification / Polish (Phase 6)**: Depends on all stories; ONLY these attest to success (never the agent)

### Within Each User Story

- Tests (if included) MUST be written and FAIL (RED) before any implementation tasks
- Implementation tasks for a story depend on tests passing (GREEN)
- Verification tasks (Phase 6) depend on all user stories being complete

### Parallel Opportunities

- T001, T002 [P], T003 [P], T004 [P] (Setup) can run in parallel (disjoint files)
- T005–T012 (RED tests) can run in parallel (disjoint checklist/test targets)
- T013–T018 (P1 tests) can run in parallel; T019–T025 (P1 implementation) can run in parallel after tests GREEN
- T026–T028 (P2) can run in parallel with P3 tasks (disjoint: CI/isolation vs audit trail)
- T033–T040 (Verification) must run sequentially (final gate only); can share agent but not block each other

---

## Parallel Example: RED Phase + P1 Implementation

```bash
# RED tests (Phase 2 — must fail first):
Task: "T005 RED — Structural attestation format check"
Task: "T006 RED — Checklist completeness check"
Task: "T010 RED — Contract mapping completeness"

# P1 implementation (Phase 3 — starts after RED tests):
Task: "T019 [P] [US1] Confirm VERIF-EMULATOR.md template complete"
Task: "T020 [US1] Verify contracts reference all 8 structural framework contracts"
Task: "T024 [P] [US1] Confirm isolation: git status unchanged"
```

---

## Implementation Strategy

### TDD / RED PHASE (Phase 2 — BLOCKS ALL)

1. Phase 1: Setup (T001–T004)
2. Phase 2: RED tests (T005–T012) — MUST FAIL before any P1 implementation
3. Confirm RED phase: all T005–T012 fail; checklist shows open items; `VERIF-EMULATOR.md` may be missing sections; isolation unverified; false-equivalence may exist
4. **DO NOT PROCEED to P1 until RED phase confirmed**

### MVP (User Story 1 — P1)

1. Complete Phase 1 (T001–T004)
2. Confirm RED phase (T005–T012 all fail)
3. Complete Phase 3 (P1): T013–T018 turn GREEN; T019–T025 complete
4. Confirm P1: `VERIF-EMULATOR.md` complete; checklist passes; structural contracts verified; isolation preserved
5. **Stop and validate independently**: `VERIF-EMULATOR.md` exists; exclusions complete; no false-equivalence; `SC-001`–`SC-004` measurable

### Incremental Delivery

1. Phase 1 → Phase 2 (RED) → Confirm RED
2. Phase 3 (P1) → Confirm GREEN → P1 independent
3. Phase 4 (P2) → Confirm GREEN → CI isolation preserved (`SC-002`); `FR-004` satisfied
4. Phase 5 (P3) → Confirm GREEN → Audit trail complete (`SC-003`, `SC-004`); checklist 100% `[x]`
5. Phase 6 (Verification) — **ONLY these attest to complete, correct feature**; agent does NOT attest; verification tasks confirm: `VERIF-EMULATOR.md` schema, exclusions, isolation, false-equivalence = 0, supply chain intact, checklist complete

---

## Notes

- [P] = disjoint files; same-phase [P] tasks can run concurrently
- [US1]/[US2]/[US3] = user story mapping from `spec.md` (P1 maintainer, P2 CI, P3 audit)
- RED tests (T005–T012) must FAIL before any user story implementation; they turn GREEN as the feature is completed
- Implementation tasks (T019–T025, T026–T028, T029–T032) can only start after RED phase confirmed and their corresponding GREEN tests pass
- Verification tasks (T033–T040) are the ONLY tasks that attest to feature completeness; never rely on agent statements or intermediate implementations
- Atomic conventional commit: whenever editing a file finishes (e.g., `VERIF-EMULATOR.md` updated, checklist updated, `spec.md` clause inserted), commit with detailed edit summary (`PUT` operations performed, lines changed, evidence references added)
- No `crates/*/src` or `src-tauri/src/` source edits; structural layer only; `cleanroom-core` stays empty; no new crate; no `Cargo.toml` / `Cargo.lock` change
- Property-based / invariant tests applied where possible: checklist completeness (all `[x]`), contract mapping exhaustiveness (8 structural contracts, no missing), exclusion list completeness (audio-output, `SC-009`, Bluetooth/car, interruption, 10/10, `SC-044`), false-equivalence search (0 hits), isolation preservation (`git status` unchanged), secret exclusion (no `.env`/`.pem`/`.key`/token in attestation)
