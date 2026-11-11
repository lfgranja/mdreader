# Convergence Report: 020-dynamic-doc-cap-scalability

- **Feature**: 020-dynamic-doc-cap-scalability
- **Date**: 2026-11-11
- **Status**: Converged (100%)

## 1. Summary of Changes
- Increased document and chapter byte budget from 93,636 (~91.4 KiB) to 1,048,576 bytes (1 MiB / 1,024 KiB) in both:
  - `crates/player/src/interruption.rs` (`MAX_DOC_BYTES`)
  - `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` (`MAX_DOC_BYTES`)
- Harmonized property-based test `crates/player/tests/prop_invalid_chapter_skipping.rs` to validate against the 1 MiB boundary.
- All functional invariants, fail-closed mechanics (`FormatUndeliverable`), and memory bounds are fully preserved.

## 2. Invariants Check
- Mono f32 @ 22050 Hz: Untouched.
- Closed set without `_ =>`: Untouched.
- Rust playback authority: Untouched.
- Audio buffer bound: Guaranteed by Spec 017 `AudioChunkCache` (32 MiB LRU).
