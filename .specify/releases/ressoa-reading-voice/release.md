# Release (R7): 018 Ressoa Reading Voice

- **Date**: 2026-11-20 (artifact backfilled)
- **Version**: v0.5.0 (proposed in `specs/018-ressoa-reading-voice/spec.md`)
- **Validation**: R4 PASS for documentary checks; dynamic gates [NV] (`.specify/validations/ressoa-reading-voice/validate.md`)

## Summary

Introduces a closed, curated, offline pt-BR reading-voice catalog with `pt_BR-faber-medium` as stable default, validated CC0 alternative(s) or documented faber-only contingency, deterministic selection/cycle/persistence across synthesis runtime, CLI, session store and Tauri bridge, and per-voice provenance with a non-imitative Ressoa identity.

## Changelog (IDs)

- Added: FR-018-001..011, SC-018-001..005, closed `VoiceId`/`VoiceSpec` catalog, CLI `voices` / `synthesize --voice`, session `last_voice`, bridge diagnose/inventory/select/cycle.
- Changed: `adapter-synthesis` build manifest and engine access; `src-tauri` voice inventory/diagnostics; voice change restarts production at authoritative resume position.
- Removed: none (hardcoded single-voice default retained as catalog default).
- Conditional: alternative voice bundled only if quality/runtime/packaging validation passes; otherwise tuned-faber-only catalog with the same API.

## Rollback

Revert the feature commit: restore the previous single-voice `adapter-synthesis` assets/build manifest and CLI, remove session `last_voice` migration and bridge voice operations, and restore prior expected-tests census. No remote configuration or network capability is introduced, so no remote rollback is needed.

## Residual risk / [NV]

- Extra bundled ONNX assets increase APK size/extraction/RAM; runtime instance strategy validated during implementation (FR-018-005).
- Auditory quality and device/gate-9 results are environment-dependent; [NV] at artifact creation.
