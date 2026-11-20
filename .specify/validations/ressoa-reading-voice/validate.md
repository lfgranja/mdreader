# Validation (R4): 018 Ressoa Reading Voice

- **Date**: 2026-11-20 (artifact backfilled from `specs/018-ressoa-reading-voice/`)
- **Status**: documentary checks PASS; dynamic gates [NV] (not executed during artifact backfill)

| Check | Command / Evidence | Result |
|---|---|---|
| Spec quality | `specs/018-ressoa-reading-voice/checklists/requirements.md` (all items checked) | PASS |
| Assessment gate | `.specify/assessments/ressoa-reading-voice/decision.md` verdict `go` (conditioned MVP) | PASS |
| Task completion | `specs/018-ressoa-reading-voice/tasks.md` — 27 `[x]`, 0 `[ ]` | PASS |
| Requirement coverage | FR-018-001..011 present in `spec.md`; SC-018-001..005 defined | PASS |
| Handoff invariant | spec §4 NFR-018-001 / SC-018-002 — mono f32 @ 22,050 Hz, fail-closed, Rust authority | PASS (spec/plan documented) |
| Provenance / non-imitative | spec FR-018-003/010, SC-018-004 — per-voice MODEL_CARD/PROVENANCE, no branding | PASS (spec/plan documented) |
| Host gates 1–8 | `PROPTEST_CASES=32 bash scripts/verify.sh` | [NV] — run by gate owner |
| expected-tests census | `cargo nextest list --workspace` vs `scripts/expected-tests.txt` | [NV] — run by gate owner |
| Device / gate 9 | Android build + on-device auditory check | [NV] — environment-dependent |

## Success criteria

- SC-018-001: closed catalog listable/selectable/cyclable via CLI + bridge, safe persistence/restore — recorded in spec; runtime verification [NV].
- SC-018-002: mono f32 @ 22,050 Hz for every voice; named failures, no silent resample/substitution — recorded in spec; runtime verification [NV].
- SC-018-003: authority/position/resume/fingerprint invariants and gates 1–8 — gates [NV].
- SC-018-004: provenance artifacts and absence of third-party branding — recorded in spec; asset audit [NV].
- SC-018-005: auditory comparison evidence and default-choice rationale — recorded in spec/plan §5.

## [NV]

- Host gate execution, census diff, device/gate-9, auditory comparison. These remain the gate owner's verification; no result is asserted here.
