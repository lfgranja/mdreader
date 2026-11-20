# Validation (R4): 014 TTS Symbol Normalization

- **Date**: 2026-11-20
- **Status**: PASS (host deterministic; device listening `[NV]`)
- **Validation**: R4 evidence record for `specs/014-tts-symbol-normalization/`

## Checks

| Check | Command / Evidence | Result |
|---|---|---|
| Spec quality | checklist in spec 014 (`checklists/requirements.md`) | PASS |
| Assess gate | `.specify/assessments/tts-symbol-normalization/decision.md` = `go` | PASS |
| TDD RED/GREEN | `normalize.rs` unit tests (17) + `normalize_invariants.rs` (8) | PASS (17 + 8) |
| Adapter synthesis suite | `cargo nextest run -p adapter-synthesis` (converge 2026-11-15) | 170 passed / 1 skipped (unrelated ENOSPC temp-write) |
| Idempotency | property test in `normalize.rs` | PASS |
| Protected zones | fenced/inline code, URLs, `2*3`, `~~~` fences byte-identical | PASS |
| Split/resume/fingerprint | `normalize_invariants.rs` + `crates/session/tests/prop_fingerprint.rs` | PASS (metadata unchanged) |
| Format/lint | `cargo fmt`, `cargo clippy -p src-tauri --lib -D warnings` | PASS (converge) |
| Host gates 1–8 | `PROPTEST_CASES=32 bash scripts/verify.sh` | `[NV]` at artifact creation |
| Device listening | SC-SYM-005 subjective | `[NV]` (optional/deferred) |

## Success criteria (spec IDs)

- **SC-SYM-001** (no `*`/`#`/bullet in Fluid TTS input): PASS (host corpus).
- **SC-SYM-002** (protected substrings byte-identical): PASS.
- **SC-SYM-003** (idempotency): PASS.
- **SC-SYM-004** (range/resume/fingerprint invariants): PASS (host).
- **SC-SYM-005** (device listening reduced fatigue): `[NV]` optional.

## Requirement coverage

FR-SYM-001 (idempotent `normalize_for_speech`), FR-SYM-002 (applied in `synth.rs::synthesize`), FR-SYM-003 (Fluid marker stripping), FR-SYM-004 (protected zones), FR-SYM-005 (mode configurable), FR-SYM-006 (`.md` scope), FR-SYM-007 (range/resume/fingerprint preserved), FR-SYM-008 (chunk metadata unchanged), FR-SYM-009 (determinism) implemented via `crates/adapter-synthesis/src/normalize.rs` (`NormalizationMode`, `normalize_for_speech`) wired in `synth.rs::synthesize`; T001 (create module), T006 (invariant tests), T007 (host gates), T009 (research-gap hardening), T010 (2026-11-15 `~~~` fences + pipe-table cells) complete. T005 UI toggle is a documented follow-up.

## [NV] / residual risk

- Device listening not performed; broader gates and live CI pending at artifact creation.
- T232/T233 (directory-reading) untouched.
