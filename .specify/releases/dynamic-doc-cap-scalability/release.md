# Release (R7): 020 Dynamic Document Cap Scalability

- **Date**: 2026-11-20 (artifact backfilled)
- **Version**: v0.1.0 runtime-limit change (no app SemVer bump recorded in spec)
- **Validation**: R4 static PASS; gates [NV] (`.specify/validations/dynamic-doc-cap-scalability/validate.md`)

## Summary

Raises the per-document and per-chapter byte budget from the spec-004 dogfood threshold (`93,636` B) to `1,048,576` B (1 MiB) consistently in the Rust player and the Android shell, so books, large docs and PKMS notes are accepted while oversize inputs continue to fail closed.

## Changelog (IDs)

- Added: FR-020-001..005; 1 MiB cap in `player::interruption::MAX_DOC_BYTES`, `player::queue::MAX_CHAPTER_BYTES` (alias) and `DocIngest.kt::MAX_DOC_BYTES`.
- Changed: property/unit tests asserting the cap (notably `crates/player/tests/prop_invalid_chapter_skipping.rs`); supporting doc references.
- Removed: temporary ~91.4 KiB dogfood threshold.

## Rollback

Revert the three constants to `93_636` / `93636` and restore the previous test expectations. No state migration, schema change or remote configuration is involved.

## Residual risk / [NV]

- Runtime memory safety at 1 MiB relies on the Spec 017 32 MiB `AudioChunkCache` bound; on-device stress is [NV].
- Host gates 1–8 and expected-tests census are [NV] at artifact creation.
