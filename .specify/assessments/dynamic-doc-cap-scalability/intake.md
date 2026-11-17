# Intake: dynamic-doc-cap-scalability

- **Slug**: `dynamic-doc-cap-scalability`
- **Origin**: Issue discovery & architectural audit post-Spec 019.
- **Requester**: Dark Software Factory Autonomous Optimizer
- **Date**: 2026-11-11
- **Status**: Intake Complete / Ready for Research

## 1. Summary
With Spec 019 enabling Android system-wide ingress (`ACTION_VIEW` and `ACTION_SEND`), users can now open markdown files from obsidian vaults, local storage, and shares. However, `DocIngest.kt`, `crates/player/src/interruption.rs`, and `crates/player/src/queue.rs` enforce a hardcoded ceiling of `MAX_DOC_BYTES = 93_636` bytes (~91.4 KiB, roughly 3h of audio at 1.0x).
Real-world technical documents, academic papers, books, and long PKMS notes frequently exceed 100 KiB (e.g. 200 KiB to 2 MiB), causing instant and unhelpful `FormatUndeliverable` refusals.

## 2. Problem Statement
The 93,636-byte limit was originally established in Spec 004 as `62424 * 1.5` based strictly on a single 2-hour test fixture (`fixture-pt-2h.md`). While fine for early dogfooding, in a production-ready application this constitutes an unnecessary barrier. With Spec 017's bounded LRU `AudioChunkCache` (32 MiB) and streaming segment chunking, memory consumption is decoupled from the raw file byte length, rendering a 91 KiB cap an artificial friction point.

## 3. High-Level Requirements
1. Expand the maximum single document and chapter byte cap from 93,636 bytes to a production-grade threshold of 1,048,576 bytes (1 MiB, ~30–35 hours of audio, easily accommodating full-length books and extensive PKMS notes).
2. Synchronize the constant across Kotlin (`DocIngest.MAX_DOC_BYTES`) and Rust (`crates/player/src/interruption.rs`, `crates/player/src/queue.rs`).
3. Ensure all tests, property tests, and contracts are harmonized without regressing memory guarantees.
