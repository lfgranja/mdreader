# Validation (R4): 015 MD Semantic Prosody

- **Date**: 2026-11-20
- **Status**: PASS (host deterministic; device listening `[NV]`)
- **Validation**: R4 evidence record for `specs/015-md-semantic-prosody/`

## Checks

| Check | Command / Evidence | Result |
|---|---|---|
| Spec quality | `specs/015-md-semantic-prosody/checklists/requirements.md` | PASS |
| Assess gate | `.specify/assessments/md-semantic-prosody/decision.md` = `go` | PASS |
| Prosody plan tests | `prosody.rs` unit tests (13) | PASS |
| Invariants | `crates/adapter-synthesis/tests/prosody_invariants.rs` (4) | PASS (deterministic; metadata unchanged) |
| Speed map | per-segment `GenerationConfig.speed` in `synth.rs` (FR-PROS-003) | PASS (host: distinct heading/code factors) |
| Thematic break | segment boundary / "." pause (FR-PROS-004) | PASS (host) |
| Protected zones | code monotone/slow, no prosody exaggeration (FR-PROS-006 shared zones) | PASS |
| Format/lint | `cargo fmt`, `cargo clippy` | PASS (converge 2026-11-10/15) |
| Host gates 1–8 | `PROPTEST_CASES=32 bash scripts/verify.sh` | `[NV]` at artifact creation |
| Device listening | SC-PROS-004 subjective | `[NV]` (optional) |

## Success criteria (spec IDs)

- **SC-PROS-001** (distinct heading/code speed factors): PASS (host).
- **SC-PROS-002** (thematic breaks produce pause): PASS (host).
- **SC-PROS-003** (invariants preserved): PASS (`prosody_invariants.rs`).
- **SC-PROS-004** (device listening): `[NV]` optional.

## Requirement coverage

FR-PROS-001 (plan enum), FR-PROS-002 (derive from markdown semantics, preserve ranges), FR-PROS-003 (speed in `synth.rs`), FR-PROS-004 (thematic break pause), FR-PROS-005 (conservative defaults), FR-PROS-006 (coordinated with 014 normalization, shared protected zones), FR-PROS-007 (`.md` scope, `.txt` neutral), FR-PROS-008 (deterministic) — all addressed; T001..T007 complete per `converge.md`.

## [NV] / residual risk

- No per-segment pitch/volume, no SSML, no user config (out of scope); speed-only lever per sherpa-onnx 1.13.8.
- Device listening and full gates/CI pending at artifact creation; T232/T233 untouched.
