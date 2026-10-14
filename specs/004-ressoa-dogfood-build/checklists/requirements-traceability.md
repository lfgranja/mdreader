# Requirements Traceability & Testability Checklist: Ressoa Dogfood Build

**Purpose**: Verify that every Functional Requirement (FR) traces to at least one Success Criterion (SC), every SC is measurable, every wire-format/interface FR has a corresponding contract, every User Story has happy-path + edge-case acceptance scenarios, and the Edge Cases section covers the mandated scenarios. This ensures no requirement is untestable and no success criterion is unverifiable.

**Created**: 2026-10-13

**Source files**: `spec.md`, `contracts/surface.md`, `contracts/earcon.md`, `contracts/ingest.md`, `contracts/diagnostic-trace.md`

---

## 1. FR → SC Traceability

Every FR must map to at least one SC. An FR with no corresponding SC is a gap.

| FR | Description | Mapped SC(s) | Traces? |
|----|-------------|--------------|---------|
| FR-001 | Screen-off + locked reading continues | SC-001 (30-min commute, screen off, locked) | [x] |
| FR-002 | Lock-screen controls (play/pause/skip, no Stop) | SC-001 (lock-screen controls only), SC-006 (lock-screen surface) | [x] |
| FR-003 | First audible speech ≤3s | SC-002 (10/10 trials, 3s bound) | [x] |
| FR-004 | Duck on transient interruption (MAY_DUCK) | SC-003 (10/10 per interruption class) | [x] |
| FR-005 | Pause on silence-demanding/permanent, 3s reason, never auto-resume | SC-003 (10/10 per interruption class) | [x] |
| FR-006 | Pause ≤1s on device removal | SC-003 (headphone removal class) | [x] |
| FR-007 | Resume at/just-before interruption (≤30s/≤60s, sentence-start snap + fixed 5s cue) | SC-003 (outcome matches mandated one, which includes replay bounds) + `contracts/resume-algorithm.md` (behavioral algorithm) | [x] |
| FR-008 | BLAKE3 content-keyed position | SC-005 (moved/renamed 10/10 resume offer) | [x] |
| FR-009 | 3s choice dialog on deliberate-pause reopen | SC-005 (3s window, silence continues) | [x] |
| FR-010 | No resume for changed content / stop / finished | SC-005 (edit→no resume, stop→start from beginning) | [x] |
| FR-011 | Lock-screen shows only title/section/progress | SC-006 (10/10 inspections, zero leaks) | [x] |
| FR-012 | 7 speed steps, saturation, persistence, pitch | SC-007 (persistence + pitch + 10/10 per rung) | [x] |
| FR-013 | Determinate/indeterminate progress (sentence-level parsed units) | SC-006 (progress field correct in 10/10 inspections) | [x] |
| FR-014 | Dismissal refused while playing | SC-006 (dismissal refused 10/10) | [x] |
| FR-015 | Offline playback, zero network voice code | SC-009 (15-min session, airplane mode, 10/10 trials, zero network voice errors) | [x] |
| FR-016 | Earcon + haptic confirmation per action | SC-004 (blind 10/10 per confirmation type) | [x] |
| FR-017 | Diagnostic record (30-day, 10MB, prose-free) | SC-008 (window, cap, zero prose) | [x] |
| FR-018 | Document ingest (picker, md/txt, BLAKE3, snapshot, trim) | SC-005 (hash-match resume after move/rename) + 4 dedicated FR-018 acceptance scenarios | [x] |
| FR-019 | In-app screen fields (title/section/progress/speed/transport/Stop) | SC-010 (10/10 inspections, Stop clears position 10/10, pt-BR strings) | [x] |

**FR → SC gaps**: None. All 19 FRs trace to at least one SC.

- **FR-015** → SC-009 (added): 15-minute session, airplane mode, 10/10 trials, zero network voice errors.
- **FR-019** → SC-010 (added): 10/10 inspections, Stop clears position 10/10, pt-BR strings verified.

---

## 2. SC Testability

Every SC must be measurable: numeric threshold, count, or clear pass/fail condition.

| SC | Measurable? | Threshold / Count |
|----|-------------|-------------------|
| SC-001 | [x] | 30-min duration, ≥2h fixture, zero unlocks, every speed 0.75x–4.0x |
| SC-002 | [x] | 3 seconds, 10/10 trials, cold start |
| SC-003 | [x] | 10/10 per interruption class (4 classes), no unlock, no auto-resume |
| SC-004 | [x] | 10/10 per confirmation type (6 types), blind |
| SC-005 | [x] | 10/10 reopens (moved/renamed), 10/10 (edited→no resume), 3s window |
| SC-006 | [x] | 10/10 inspections, zero leaks, dismissal refused 10/10 |
| SC-007 | [x] | Persistence across sessions, pitch ≤50 cents, 10/10 per rung (7 rungs) |
| SC-008 | [x] | 30-day window, 10MB cap, zero prose entries |
| SC-009 | [x] | 15-min session, airplane mode, 10/10 trials, zero network voice errors |
| SC-010 | [x] | 10/10 inspections, Stop clears position 10/10, pt-BR strings |

All 10 SCs are measurable. No gaps.

---

## 3. Contract Coverage

Every FR that defines a wire format, payload, or interface must have a corresponding contract section.

| FR | Interface / Payload | Contract Section | Covered? |
|----|----------------------|------------------|----------|
| FR-002 | Lock-screen controls (play/pause/skip, no Stop, rate-step) | `contracts/surface.md` § Lock-screen surface | [x] |
| FR-005 | Reason lines table + warning earcon shape | `contracts/surface.md` § Fixed reason lines + `contracts/earcon.md` § Warning | [x] |
| FR-007 | Resume replay bounds (≤30s/≤60s, sentence-start snap) | SC-003 (outcome matches mandated one, which includes replay bounds) + `contracts/resume-algorithm.md` (behavioral algorithm) | [x] |
| FR-009 | 3s resume choice dialog behavior | `contracts/surface.md` § Resume | [x] |
| FR-011 | Lock-screen field whitelist | `contracts/surface.md` § Lock-screen surface (Fields) | [x] |
| FR-012 | Speed ladder (7 rungs, saturation, persistence) | `contracts/surface.md` § Speed | [x] |
| FR-013 | Progress payload (determinate/indeterminate, template) | `contracts/surface.md` § Progress + Payload JSON | [x] |
| FR-016 | Earcon grammar + haptic patterns | `contracts/earcon.md` (full contract) | [x] |
| FR-017 | Diagnostic record schema + trim + scan | `contracts/diagnostic-trace.md` (full contract) | [x] |
| FR-018 | Ingest flow (picker → snapshot → BLAKE3) | `contracts/ingest.md` (full contract) | [x] |
| FR-019 | In-app screen fields + Stop control | `contracts/surface.md` § Single minimal in-app screen | [x] |

**Contract coverage gaps**: None. All 11 interface/payload FRs have corresponding contract sections.

- **FR-007** → `contracts/resume-algorithm.md` (added): behavioral algorithm specifying sentence-start snap, 5–10s cue, total backward cap, never-mid-word, never-doc-start, degrade fallback, atomic write, and measurement.

---

## 4. Acceptance Scenario Coverage

Every User Story must have acceptance scenarios covering the happy path AND at least one edge case.

| US | Happy Path Scenario | Edge Case Scenario(s) | Covers Both? |
|----|---------------------|----------------------|--------------|
| US1 — Listen through commute, screen off | Scenario 1: screen off → playback continues | Scenario 2: lock-screen controls functional; Scenario 3: first speech ≤3s | [x] |
| US2 — Survive interruptions | Scenario 1: transient → duck, keep going | Scenario 2: call/alarm → pause + reason; Scenario 3: headphone removal → pause ≤1s; Scenario 4: resume position bounds | [x] |
| US3 — Never lose your place | Scenario 1: reopen moved/renamed → resume offer | Scenario 2: content changed → no resume; Scenario 3: explicit stop → start from beginning | [x] |
| US4 — Lock screen respects attention | Scenario 1: only title/section/progress shown | Scenario 2: speed persists + pitch unchanged; Scenario 3: indeterminate progress; Scenario 4: rate-step without value | [x] |

All 4 User Stories have both happy-path and edge-case coverage. No gaps.

---

## 5. Edge Case Coverage

The Edge Cases section in `spec.md` must cover the 8 mandated scenarios.

| Edge Case | Covered By | Present? |
|-----------|-----------|----------|
| Audio focus denial (FocusDenied/FocusDelayed) | FR-005 (closed set), `contracts/surface.md` reason table | [x] |
| Transient-then-permanent interruption (latest wins) | FR-005 (latest event wins, paused + no auto-resume) | [x] |
| Lock-screen dismissal while playing | FR-014 (dismissal refused), `contracts/surface.md` dismissal rule | [x] |
| Kill/power-loss mid-sentence | FR-007 (≤60s replay), FR-009 (lock-screen resume control) | [x] |
| Second document while first playing/paused | FR-018 (auto-save + switch, no auto-start, bookmark preserved) | [x] |
| Lock-screen privacy setting (hide notification contents) | FR-011 + `contracts/surface.md` (obeys platform setting, no workaround) | [x] |
| Undeliverable voice (VoiceUnavailable/FormatUndeliverable/DecodeFailed) | FR-005 + FR-018 (closed-set failure, no substitution) | [x] |
| No-sections document (fallback ±10s, indeterminate progress, no title) | FR-002 (fallback ±10s), FR-013 (indeterminate + no title), `contracts/surface.md` | [x] |

All 8 mandated edge cases are covered. No gaps.

---

## Notes

### Pass/Fail Counts

| Section | Items | Pass ([x]) | Fail ([ ] ) | Pass Rate |
|---------|-------|------------|-------------|-----------|
| 1. FR → SC Traceability | 19 | 19 | 0 | 100% |
| 2. SC Testability | 10 | 10 | 0 | 100% |
| 3. Contract Coverage | 11 | 11 | 0 | 100% |
| 4. Acceptance Scenario Coverage | 4 | 4 | 0 | 100% |
| 5. Edge Case Coverage | 8 | 8 | 0 | 100% |
| **Total** | **52** | **52** | **0** | **100%** |

### Gaps Requiring Attention

None. All 3 previously identified gaps have been resolved:

1. **FR-015 (offline playback)** → SC-009 added: 15-minute session, airplane mode, 10/10 trials, zero network voice errors.
2. **FR-019 (in-app screen)** → SC-010 added: 10/10 inspections, Stop clears position 10/10, pt-BR strings verified.
3. **FR-007 (resume replay bounds)** → `contracts/resume-algorithm.md` added: behavioral algorithm with sentence-start snap, 5–10s cue, total backward cap, never-mid-word, never-doc-start, degrade fallback, atomic write, and measurement.

### Recommendations

- **Proceed to `/speckit.tasks`** — all traceability gaps resolved, 100% pass rate achieved.
