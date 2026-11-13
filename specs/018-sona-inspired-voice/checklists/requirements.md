# Specification Quality Checklist: 018-sona-inspired-voice

- **Feature**: `specs/018-sona-inspired-voice/spec.md`
- **Assessment (R1)**: `.specify/assessments/sona-inspired-voice/decision.md` — **verdict: go** (2026-11-10)
- **Created**: 2026-11-10
- **Status**: Draft (ready for planning/implementation validation)

## Purpose

Validate that `spec.md`, `plan.md`, and `tasks.md` are complete enough to implement requester Option (B): a curated, bundled, offline, non-imitative "Sona-inspired feel" pt-BR reading voice catalog aligned to `DESIGN-SONA/`, with tuning and evolution-ready multi-voice support.

## Checklist

### Scope & Decision Traceability
- [x] Scope matches approved Option (B): curated/tuned Sona-inspired feel, bundled alternative + tuning, multi-voice evolution.
- [x] Explicitly excludes Option (A): no LoL Sona/character/VO imitation, no Riot trademarks/likeness, no voice cloning, no performer-branded identity.
- [x] Explicitly bounds Option (C) supporting behavior: earcons/prosody/normalization only as needed, no unrelated UI redesign.
- [x] Excludes user import/download/cloud/marketplace/paid voices and any `INTERNET` capability.
- [x] Assessment artifacts exist and decision is `go` with conditions carried forward.

### User Scenarios & Acceptance
- [x] Prioritized user stories cover selection/persistence (US1), cycle/restart safety (US2), synthesis contract (US3), provenance/identity (US4).
- [x] Each story is independently testable with acceptance scenarios.
- [x] Edge cases defined: unknown/malformed persisted id, missing/corrupt assets, catalog size/contingency, rapid cycling, per-voice defaults, shared espeak compatibility, Android extraction migration.

### Functional & Non-Functional Requirements
- [x] Closed bundled catalog defined (stable ids = dir names, fixed order, default `pt_BR-faber-medium`, no `Other/Unknown`).
- [x] Curated alternative constrained to 22050 Hz + CC0 + auditable provenance (target `pt_BR-cadu-medium`, fallback `pt_BR-jeff-medium`); `pt_BR-edresson-low` excluded.
- [x] Voice-aware synthesis/CLI/session/bridge/cycle semantics specified with safe restart-at-authoritative-position and no mixed-voice PCM.
- [x] Audio invariants preserved: mono `f32` @ 22,050 Hz, no silent resample/downmix, fail-closed named errors.
- [x] Capabilities/tier preserved: `OfflineDefault`, `(SsmlLevel::None, AlignmentGranularity::None)`.
- [x] Provenance requirements specified per bundled voice (source/license/checksums/defaults/files, `MODEL_CARD`/`PROVENANCE.md`).

### Plan & Tasks Readiness
- [x] `plan.md` identifies affected crates/surfaces (`adapter-synthesis`, `session`, `src-tauri`, contracts/tests/scripts/docs), milestones, verification, and contingencies.
- [x] `tasks.md` is dependency-ordered, phased, and maps tasks to stories with concrete paths (including asset/provenance, runtime refactor, persistence, bridge, tests, expected-tests/script alignment, gates/evidence).
- [x] Fallback contingencies documented: alternative-voice validation failure, sherpa-onnx multi-instance infeasibility (recreate-on-switch), shared espeak incompatibility.

### Constitution / Quality Gates
- [x] Offline-first and zero-`INTERNET` reaffirmed.
- [x] Rust authority / mirror-only boundaries respected.
- [x] Closed-set/exhaustive-match expectation stated.
- [x] Host gates 1–8 required; gate 9/device reported separately if unexecuted.
- [x] `scripts/expected-tests.txt` hygiene called out for nextest target changes.

### Clarifications & Risks
- [x] Blocking clarifications: **0** (R3 satisfied). Residuals are implementation validations with dispositions (auditory fit/default, sherpa-onnx 1.13.8 lifecycle, size/memory/load/extraction).
- [x] Risks and mitigations recorded (IP/branding, subjective fit, runtime/packaging, blast radius).

## Notes

- Final curated voice choice and default may change only with recorded auditory/runtime evidence per decision conditions; if no alternative passes, execute documented contingency (multi-voice-ready API with faber default/tuning) rather than widening scope.
- This checklist does not substitute implementation verification evidence (tests, gates, auditory comparison, provenance hashes).