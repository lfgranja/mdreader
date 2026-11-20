# Release (R7): 013 TTFA Synthesis Pipeline Scheduling

- **Date**: 2026-11-20
- **Version**: v0.1.0 infrastructure/TTFA follow-up (no app SemVer bump; internal scheduling change)
- **Validation**: R4 PARTIAL (`.specify/validations/ttfa-synthesis-pipeline/validate.md`)
- **Target device**: 2201117TY (Android 13 / SDK 33) — device proof `[NV]`

## Summary

Moves CPU-bound Piper/sherpa-onnx synthesis off the Tauri/JNI/Tokio-async tick path onto a generation-guarded blocking producer, prioritizes first-useful audio for resume offsets, and keeps playback-watch synthesis-free while preserving the 60 s resident-queue bound and all resume/fingerprint invariants.

## Changelog (IDs)

- **Added**: generation identity/wake guards, bounded generation-safe producer helpers and a test-only synthesis hook seam in `src-tauri/src/lib.rs`; `ttfa_013_tests` host coverage (T001/T003/T007/T013/T018).
- **Changed**: `play_document` startup ordering (prepare/open backend → produce/enqueue first useful generation-safe audio → land/publish Playing); `playback_watch_tick` reduced to signal/pump/background request; `request_background_production` via `tauri::async_runtime::spawn_blocking` serialized by `producer_driving`.
- **Removed**: synchronous `synthesize` calls from the watch tick path (FR-TTFA-003).
- **Closed / carried**: FR-TTFA-001..008 specified; SC-TTFA-001..005 tracked; T232/T233 hooks preserved (not completed, non-regression only).

## Rollback

Revert the scheduling commits in `src-tauri/src/lib.rs` (restore the prior synchronous `play_document`/`stream_synthesis` and `playback_watch_tick` paths) and drop the `ttfa_013_tests` additions. No voice/model assets, remote configuration, or persisted data are changed; the change is in-process and reversible by code revert.

## Residual risk / [NV]

- Device TTFA improvement (SC-TTFA-001/005) unproven until rebuilt install + controlled smoke with fresh paired `play_to_audio_ms` capture.
- Full stale-commit / FR-001/005 failure matrix (T003/T018) and high-offset first-useful proof (T013) remain open; watch path validated by code + targeted host tests only.
- 5+ minute underrun smoke and full host gates 1–8 not re-run at artifact creation; CI/merge pending.
