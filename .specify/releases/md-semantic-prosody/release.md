# Release (R7): 015 MD Semantic Prosody

- **Date**: 2026-11-20
- **Version**: v0.1.0 synthesis-quality increment (no app SemVer bump; internal prosody)
- **Validation**: R4 PASS (`.specify/validations/md-semantic-prosody/validate.md`)

## Summary

Adds conservative per-segment prosody to Markdown TTS: a deterministic prosodic plan (normal/heading/emphasis/strong/quote/code/list/question/exclamation/thematic-break) drives `GenerationConfig.speed` at synthesis time, yielding structural pauses and rate variation without SSML or metadata changes.

## Changelog (IDs)

- **Added**: `prosody.rs` with the plan enum and conservative speed map; `prosody_invariants.rs` (4 tests); 13 prosody unit tests.
- **Changed**: `synth.rs` applies per-segment `GenerationConfig.speed`; pipeline order fixed as classify → plan → normalize → synthesize (structure derived before 014 normalization).
- **Unchanged**: `SourceRange`, `segment_index`, `resume_point`, blake3 fingerprint; protected code zones stay monotone/slow (no exaggeration).

## Rollback

Revert the prosody commits: remove the `GenerationConfig.speed` override from `synth.rs::synthesize` (restore default speed) and drop `prosody.rs` / `prosody_invariants.rs`. No voice/model assets, remote configuration, or persisted data change; fully reversible by code revert.

## Residual risk / [NV]

- Device listening (SC-PROS-004) optional and not performed; speed-only lever (no pitch/volume/SSML/user config) is an explicit scope limit.
- Full host gates 1–8 and live CI pending at artifact creation; T232/T233 untouched.
