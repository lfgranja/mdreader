# Release (R7): 020 PocketTTS Voice Cloning Spike

- **Date**: 2026-11-20 (artifact backfilled)
- **Version**: Spike/decision release — no app SemVer bump, no shipped model or voice change
- **Validation**: R4 PASS for documentary + cited measurement checks (`.specify/validations/pockettts-voice-cloning-spike/validate.md`)

## Summary

Executed a device-first 80/20 spike to decide whether offline cloned pt-BR voice (Ressoa) can meet Ressoa's TTFA/RTF/memory bar on `2201117TY` (SD680). The measured matrix fails every gate dimension, so the decision is **NO-GO for integration**; Piper pt-BR remains the default. No production code, handoff, permission or shipped asset changed.

## Changelog (IDs)

- Added: FR-PKT-001..011, SC-PKT-001..006; SDD artifacts (spec/plan/tasks/checklist/assessment/validate/release) in `specs/020-pockettts-voice-cloning-spike/` and `.specify/`.
- Evidence: `measurements.md` (9 configs × cold/warm, reference hash, commands, NO-GO rationale), `PLAN-80-20.md` (phases 0–3, gate definition). Both preserved intact.
- Decision: `go` for the spike, **NO-GO** for Phase 2 integration; Phase 3 fallbacks recorded (Piper prewarm/handover profiling; custom/bake Piper only with rights).
- Removed: none.

## Rollback

Nothing to roll back in production: the spike used an external harness and on-device temporary files (`/data/local/tmp/*`). Removing the new SDD artifacts and temporary host harness outputs restores the prior doc state; no app state, schema, manifest or remote configuration was modified.

## Residual risk / [NV]

- End-to-end `play_to_audio_ms` with a cloned voice not measured [NV]; synthesis-only evidence is already disqualifying for this profile.
- Bundle ↔ sherpa runtime exact compatibility and license/consent clearing remain unresolved preconditions for any future revival.
- Stronger hardware / newer quantized bundles may alter the result [NV]; requires a new spike with the same gate.
