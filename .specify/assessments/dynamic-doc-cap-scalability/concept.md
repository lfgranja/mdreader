# Concept: dynamic-doc-cap-scalability

- **Slug**: `dynamic-doc-cap-scalability`
- **Date**: 2026-11-11

## 1. Solution Architecture
- **Canonical Constant**:
  - `crates/player/src/interruption.rs`: update `MAX_DOC_BYTES` to `1_048_576` (`1 MiB`).
  - `crates/player/src/queue.rs`: `MAX_CHAPTER_BYTES` remains mapped to `MAX_DOC_BYTES` (`1_048_576`).
  - `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt`: update `MAX_DOC_BYTES` to `1_048_576`.
- **Refusal Mechanics**:
  - Documents <= 1,048,576 bytes are ingested normally.
  - Documents > 1,048,576 bytes are refused with `FormatUndeliverable`.
- **Verification Invariant**:
  - Property test `crates/player/tests/prop_invalid_chapter_skipping.rs` continues to verify that files exceeding `MAX_CHAPTER_BYTES` are skipped with `FormatUndeliverable`.
