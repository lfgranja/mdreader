# Release (R7): 002 FR-024 Synthesis Adapter

- **Date**: 2026-11-20 (artifact backfilled from `specs/002-fr-024-synthesis-adapter/`)
- **SemVer**: v0.2.0 (minor feature slice folded into the v0.1.0-rc release line; no standalone app bump)
- **Validation**: R4 PASS for documentary checks (`.specify/validations/fr-024-synthesis-adapter/validate.md`); dynamic gates/benchmark [NV]

## Summary

Adds the real offline synthesis adapter (`sherpa-onnx` + Piper pt-BR `faber` default) behind the 001 shell: mono f32 @ 22050 Hz, paragraph/heading resume, bounded chunks, named engine failures, and selector-invariant fallback with consent/SC-044 explicitly deferred.

## Changelog (IDs)

- **Added**: FR-024 + derived FR-024a (≤5000-byte paragraph/heading chunks, resume-alignment points), FR-024b (named `EngineError` on any non-mono-f32/22050 Hz delivery), FR-024c (`declared_capabilities()` → `(SsmlLevel::None, AlignmentGranularity::None)` against existing `DeclaredFormat`/`EngineError` shapes).
- **Added**: 7-member `EngineError` naming (RateMismatch, ChannelMismatch, InitFailed, Unreachable, DeviceRefused, DeviceLost, StartRefused) + condition→variant table (FR-035, SC-035).
- **Added**: `adapter-synthesis` crate host tests (106/106 green recorded), `engine-core` selector wiring (`is_selectable`, `plan_for`), `crates/player/src/consume.rs` declaration-consumption branch (T059/T060/T063/T065/T067).
- **Changed**: SC-009 bound reconciled to ≤10 s cold-start on the Redmi (spec 003 amendment); recorded as a **named blocker**, never a silent pass or silent deferral.
- **Fixed**: duplicate task ID `T062-PREP` → `T086-PREP` (task-ID uniqueness gate); all text and order preserved. Host blockers B1 (alsa-lib-devel) / B2 (ndk target-gated at `player/Cargo.toml:21`) resolved (T086-PREP record).
- **Removed / deferred**: SC-044 stays `DEFERRED` (7 discharge prerequisites restated at `spec.md:120`); consent acquisition (writer/UI/verb/storage key, T075) out of Option A; `test_double` and stand-ins not selectable (`001/spec.md:89`).
- **Tasks**: T010..T086 (after cleanup); T001..T009 not present in this phase-DAG file. 80 checked, 9 named open follow-ups.

## Rollback

Revert the adapter crate + selector/consume wiring; restore the previous engine registry (no adapter registered → `test_double` admissibility per `001/spec.md:89` returns). No remote configuration, consent surface or signed build shipped. `SC-009-GATE-RECORD.md` history remains append-only.

## Residual risk / [NV]

- SC-009 device benchmark and host gates not executed at backfill — gate owner runs them.
- SC-044 DEFERRED is not a discharge; any future PASS requires the 7 prerequisites, including a second real engine.
- Remaining open tasks are verification-only (rule 5, independent verifier) or workspace-owned; none silently closed.
