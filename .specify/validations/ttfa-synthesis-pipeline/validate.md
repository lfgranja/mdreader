# Validation (R4): 013 TTFA Synthesis Pipeline Scheduling

- **Date**: 2026-11-20
- **Status**: PARTIAL (host deterministic PASS; device TTFA proof `[NV]`)
- **Validation**: R4 evidence record for `specs/013-ttfa-synthesis-pipeline/`

## Checks

| Check | Command / Evidence | Result |
|---|---|---|
| Spec quality | `specs/013-ttfa-synthesis-pipeline/checklists/requirements.md` | PASS (items checked) |
| Assess gate | `.specify/assessments/ttfa-synthesis-pipeline/decision.md` = `go` (Option A) | PASS |
| TDD RED/GREEN | `ttfa_013_tests` hooks in `src-tauri/src/lib.rs` (T001/T003/T007/T013/T018) | PARTIAL (host hook-driven; full startup matrix pending) |
| Host gates (targeted) | `cargo nextest run -p src-tauri --lib playback_watch_tests ttfa_013_tests` | 8 passed (converge 2026-11-07) |
| Measured startup test | `cargo nextest run -p src-tauri --test test_play_to_audio_measured` | 4 passed |
| Watch decoupling | `playback_watch_tick` synthesis-free; signal/pump/background only (FR-TTFA-003) | PARTIAL (code + targeted tests) |
| Generation/cancel guards | invalidation on switch/stop/teardown (FR-TTFA-007) | PARTIAL (host tests; full side-effect matrix T003/T018 pending) |
| Invariants | fingerprint/resume/SourceRange/`samples_before_entry` unchanged (FR-TTFA-005) | PARTIAL (targeted regressions) |
| Format/lint | `cargo fmt --all -- --check`, `cargo clippy -p src-tauri --lib -D warnings` | PASS (targeted, converge 2026-11-07) |
| Device TTFA | fresh `play_to_audio_ms` on 2201117TY (baseline pairing) | `[NV]` — blocked for proof |
| Full host gates 1–8 | `PROPTEST_CASES=32 bash scripts/verify.sh` | `[NV]` at artifact creation |
| PR/CI merge | live CI | `[NV]` |

## Success criteria (spec IDs)

- **SC-TTFA-001** (material TTFA improvement / eliminate >45 s extremes): PARTIAL — code refactor present; no fresh paired device capture, not claimed proven.
- **SC-TTFA-002** (no underrun, 60 s cap): PARTIAL — `MAX_PREROLL_SECS`/`ResidentQueue` preserved; 5+ min device smoke `[NV]`.
- **SC-TTFA-003** (`playback_watch_tick` no synth CPU): PARTIAL — structural/test evidence (host).
- **SC-TTFA-004** (regressions absent + new cancel/offset tests): PARTIAL — targeted suites green; T003/T007/T013/T018 full matrix open.
- **SC-TTFA-005** (device resume/fingerprint/controls smoke): `[NV]`.

## Requirement coverage

- FR-TTFA-001 (worker offload), FR-TTFA-002 (first useful audio), FR-TTFA-004 (60 s bound/backpressure), FR-TTFA-006 (FR-001/005 safety), FR-TTFA-008 (measurements): addressed by generation-safe bounded producer in `src-tauri/src/lib.rs`, verified by `ttfa_013_tests` and `test_play_to_audio_measured` (PARTIAL where end-to-end device proof is outstanding).

## [NV] / residual risk

- Device TTFA claim requires rebuilt install + controlled smoke + fresh `play_to_audio_ms` on 2201117TY; baseline pairing unavailable in host evidence.
- Out of scope per spec: Option B intra-model VITS streaming, voice/model changes, T232/T233 completion (hooks preserved, non-regression only).
