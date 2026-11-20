# Validation (R4): 004 Ressoa Dogfood Build

- **Date**: 2026-11-20 (artifact backfilled from `specs/004-ressoa-dogfood-build/`)
- **Status**: documentary/static checks PASS; dynamic gates, gate-9 build and device trials [NV]

| Check | Command / Evidence | Result |
|---|---|---|
| Spec quality | `specs/004-ressoa-dogfood-build/checklists/requirements-traceability.md` + `requirements.md` (all items checked) | PASS |
| Assessment gate | `.specify/assessments/ressoa-dogfood-build/decision.md` verdict `go` (Option A only) | PASS |
| Task completion | `tasks.md` — 173 `[x]`, 0 `[ ]` (T001..T173) | PASS |
| Requirement coverage | FR-001..019 + SC-001..010 present in `spec.md` | PASS |
| Surface purity | FR-011/FR-019 + SC-006/SC-010 — lock-screen & in-app show title/section/progress (+ in-app speed/transport) only, no prose/time/rate/voice | PASS (contract; device [NV]) |
| Resume authority | FR-007/FR-008/FR-013 — BLAKE3 keyed, last-spoken atomic every 20 s, determinate-only progress | PASS (contract; device [NV]) |
| Closed-set interruption | FR-005 8 reasons ↔ ≤8-word pt-BR lines; FR-004 duck; FR-006 removal ≤1 s | PASS (spec documented; device [NV]) |
| Diagnostics | FR-017/SC-008 — 30 d/10 MB, oldest-first, app-private, no prose, `adb pull` only | PASS (contract; device [NV]) |
| Offline | FR-015/SC-009 — zero network-dependent voice code | PASS (contract; device [NV]) |
| pt-BR strings | all user-visible strings pt-BR; English identifiers diag-only (clarification 2026-10-14) | PASS (contract; device [NV]) |
| Host gates 1–8 | `PROPTEST_CASES=32 bash scripts/verify.sh` | [NV] — gate owner |
| Gate 9 (APK/AAB) | Rust compile + Gradle sideload build | [NV] — environment-dependent |
| Device trials | SC-001..SC-010 (30-min commute, 10/10 classes, blind confirmations) | [NV] — environment-dependent |

## Success criteria

- SC-001: 30-min locked session, zero unlocks, valid at 4.0× — device [NV].
- SC-002/SC-003/SC-004: ≤3 s cold start, interruption outcomes, blind earcon/haptic identification 10/10 — device [NV].
- SC-005/SC-009: hash resume on move/rename, none on edit/stop, 15-min airplane session — device [NV].
- SC-006/SC-010: surface purity 10/10, Stop clears position — device [NV].
- SC-008: diagnostics within 30 d/10 MB, zero prose — device [NV].

## [NV]

- Host gate execution, gate-9 build, device/10/10 trials, crate-level dynamic tests. No runtime result asserted here; this dogfood build is the artifact that supplies the missing measurements once run.
