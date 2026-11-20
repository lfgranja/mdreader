# Validation (R4): 016-voz-lula — TTS voz Lula da Silva

- **Date**: 2026-11-20
- **Status**: DEFERRED / NOT IMPLEMENTED (design artifacts validated; implementation gated)
- **Validation**: R4 evidence record for `specs/016-voz-lula/`

## Checks

| Check | Command / Evidence | Result |
|---|---|---|
| Spec quality | `specs/016-voz-lula/checklists/requirements.md` | PASS (design checklist complete) |
| Assess gate | `.specify/assessments/voz-lula/decision.md` = `go` (specification only) | PASS |
| Implementation decision | `specs/016-voz-lula/decision.md` = `defer` (2026-11-10) | DEFERRED (authoritative for code) |
| FR-01..05 code evidence | no adapter/CLI/bridge changes for 016 | N/A — not implemented |
| Model asset | `pt_BR-lula-medium/*.onnx` / `tokens.txt` absent (placeholder removed 2026-11-10) | N/A — precondition unmet |
| SC-001 build | `cargo build --workspace` with Lula support | `[NV]` (not implemented) |
| SC-002 tests | `cargo nextest run --workspace` + voice-selection tests | `[NV]` |
| SC-003 clippy | `cargo clippy --workspace --all-targets -- -D warnings` | `[NV]` |
| SC-004 fmt | `cargo fmt --all -- --check` | `[NV]` |
| SC-005 fail-closed | select `lula` with absent `.onnx` → `InitFailed` | `[NV]` (no selection seam yet) |
| Constitution gate | `exactly_one_bundled_voice_directory` / `test_noscope` | unchanged (single faber voice) |

## Success criteria (spec IDs)

- SC-001..SC-005: **not demonstrated** — implementation deferred; no code, CLI, bridge or asset changes for 016.

## Requirement coverage

FR-01 (voice selection), FR-02 (canonical 22 050 Hz mono f32 handoff), FR-03 (fail-closed `InitFailed`), FR-04 (CLI `--voice` + Tauri `set_voice`), FR-05 (`pt_BR-lula-medium` asset convention) are specified in `spec.md`/`plan.md` but not implemented. No T IDs are assigned in `specs/016-voz-lula/tasks.md`.

## [NV] / gating conditions

Implementation remains deferred pending documented consent (or a fictional CC0 voice), licence + provenance + legal review, a constitution amendment + `test_noscope` gate update, and synthetic-content store disclosure — see `specs/016-voz-lula/decision.md`. No host gates or device validation runs are claimed.
