# Research: dynamic-doc-cap-scalability

- **Slug**: `dynamic-doc-cap-scalability`
- **Date**: 2026-11-11
- **Confidence**: Very High

## 1. Technical Baseline Analysis
- **Current Definition**:
  - `crates/player/src/interruption.rs`: `pub const MAX_DOC_BYTES: usize = 93636;`
  - `crates/player/src/queue.rs`: `pub const MAX_CHAPTER_BYTES: usize = crate::interruption::MAX_DOC_BYTES;`
  - `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt`: `const val MAX_DOC_BYTES = 93636`
- **Memory Consumption Invariants**:
  - Ingestion: Reads the document stream into a byte buffer, calculates Blake3 hash, writes a single snapshot file (`filesDir/snapshots/<hex>.md`). A 1 MiB file consumes ~1 MiB transient heap in JVM and ~1 MiB in flash storage.
  - Rust Parsing & Timeline: Headings and paragraphs are parsed into lightweight indices/ranges. Even a 1 MiB markdown file generates ~2,000 to 5,000 AST nodes (~200 KiB RAM).
  - Synthesis Audio Buffer: Handled via `AudioChunkCache` (Spec 017), which is strictly capped at 32 MiB of PCM chunks using an LRU eviction policy. The audio buffer does NOT scale with document size.
  - Playback clock & timeline: Driven by `PositionClock` using fractional seconds `f64`. A 30-hour document (~108,000 seconds) has zero precision loss in IEEE-754 `f64` (53 bits of mantissa holds integer seconds up to 285 million years).

## 2. Risk Assessment
- **OOM Risk**: Negligible. Modern Android devices (minSdk 29, Android 10+) allocate a minimum JVM heap of 192 MiB to 512 MiB per process. A 1 MiB file load is trivial.
- **Fail-Closed Guarantee**: Oversized documents (> 1 MiB) will still trigger `FormatUndeliverable` fail-closed protection as required by `FR-018`.
- **Sync with Verification Gates**: Tests that assert exact values or boundaries must be updated to avoid regression.
