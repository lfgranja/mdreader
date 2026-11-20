# Validation (R4): 001 Android Media Shell

- **Date**: 2026-11-20 (artifact backfilled from `specs/001-android-media-shell/`)
- **Status**: documentary/static checks PASS; dynamic gates and device trials [NV] (not executed during backfill — gate owner runs them)

| Check | Command / Evidence | Result |
|---|---|---|
| Spec quality | `specs/001-android-media-shell/checklists/requirements.md` (all items checked) | PASS |
| Assessment gate | `.specify/assessments/android-media-shell/decision.md` verdict `go` | PASS |
| Task completion | `tasks.md` — 140 `[x]`, 11 `[ ]` (open follow-ups incl. T232/T233) | PASS (open items named, not silent) |
| Requirement coverage | FR-001..045 (incl. lettered variants) + SC-001..057 present in `spec.md` | PASS |
| Single-authority invariant | SC-035 — no playback logic in Android/Tauri surface; FR-002/FR-003 Rust owns position/buffering | PASS (contract + host check defined; runtime [NV]) |
| Closed sets | FR-044 + SC-057 — rate/focus/failure/resume sets enumerated, no `Other`/`Unknown` | PASS (spec documented) |
| FGS declaration | SC-006 automated manifest/permission check; FR-006..FR-009a bootstrapping | PASS (test defined; run [NV]) |
| Bundle/surface purity | FR-028e/SC-053 non-dismissable while playing; FR-045/SC-056 no prose in diagnostics | PASS (spec documented; device [NV]) |
| Host gates 1–8 | `PROPTEST_CASES=32 bash scripts/verify.sh` | [NV] — gate owner |
| Test census | `cargo nextest list --workspace` vs `scripts/expected-tests.txt` | [NV] — gate owner |
| Device trials | SC-001..SC-005, SC-009, SC-011..SC-014 (10/10 on reference phone) | [NV] — environment-dependent |

## Success criteria

- SC-001: 5-min screen-off continuity 10/10 — recorded in spec; device [NV].
- SC-002: headphone-removal pause ≤1 s 10/10 — recorded in spec; device [NV].
- SC-003/SC-004: duck gain 15–25% + lock-screen transport 10/10 — device [NV].
- SC-006: automated FGS declaration + permissions — test defined, run [NV].
- SC-007/SC-011/SC-012: killed-service recovery pauses, never auto-resumes — device [NV].
- SC-009: first audible ≤3 s (amended to ≤10 s cold-start by spec 003) — device [NV].
- SC-035: zero playback logic in surface — host census defined, run [NV].
- SC-057: every governed closed set enumerated — PASS (static).

## [NV]

- Host gate execution, expected-tests census, device/10/10 trials, gate 9. No runtime result is asserted here.
- 11 open follow-up tasks remain in `tasks.md` (incl. T232/T233 — surface directory/chapter controls, now spec 023).
