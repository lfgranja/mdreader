# Release (R7): 014 TTS Symbol Normalization

- **Date**: 2026-11-20
- **Version**: v0.1.0 synthesis-quality increment (no app SemVer bump; internal normalization)
- **Validation**: R4 PASS (`.specify/validations/tts-symbol-normalization/validate.md`)

## Summary

Adds deterministic, idempotent Markdown-symbol normalization at the TTS synthesis boundary: Fluid mode strips emphasis/heading/list/thematic markers from spoken text while protected zones and all segment/range/resume/fingerprint contracts remain byte-identical.

## Changelog (IDs)

- **Added**: `crates/adapter-synthesis/src/normalize.rs` (`NormalizationMode`, `normalize_for_speech`); `crates/adapter-synthesis/tests/normalize_invariants.rs`; 17 unit + 8 invariant tests.
- **Changed**: `synth.rs::synthesize` normalizes spoken text before `generate_with_config`; 2026-11-15 hardening adds `~~~` fence parity and pipe-table cell normalization (T010).
- **Removed**: literal marker leakage in Fluid spoken text ( asterisco/heading/bullet noise).
- **Unchanged**: `DocumentSegment.content`, `chunk.rs` metadata projection, source ranges, resume points, byte lengths, blake3 fingerprints.

## Rollback

Revert the normalization commits: remove the `normalize_for_speech` call from `synth.rs::synthesize` (restore verbatim `segment.content`) and drop `normalize.rs` / `normalize_invariants.rs` tests. No voice/model assets, remote configuration, or persisted data change; fully reversible by code revert.

## Residual risk / [NV]

- Device listening (SC-SYM-005) optional and not performed; T005 Literal-mode UI toggle deferred as a follow-up.
- Broader host gates 1–8 and live CI pending at artifact creation; T232/T233 untouched.
