# Feature Specification: 020-dynamic-doc-cap-scalability

- **Feature**: 020-dynamic-doc-cap-scalability
- **Title**: Increase Document and Chapter Byte Budget to 1 MiB
- **Status**: Implemented / Ready for Verification
- **Created**: 2026-11-11
- **Assessment**: `.specify/assessments/dynamic-doc-cap-scalability/decision.md` (Verdict: `go`)

## 1. Context & Motivation
Spec 004 hardcoded `MAX_DOC_BYTES = 93636` (~91.4 KiB) as a temporary dogfooding threshold based on a 2-hour test fixture (`fixture-pt-2h.md`). In real use, markdown files containing complete books, extensive documentation, or large PKMS notes easily exceed 100 KiB, triggering `FormatUndeliverable` refusals. Now that Spec 017 has bounded the runtime memory buffer with an LRU cache (`AudioChunkCache` 32 MiB), the raw file length can safely be expanded to 1,048,576 bytes (1 MiB) without risk of memory bloat.

## 2. Functional Requirements (FR)
- **FR-020-001**: `crates/player/src/interruption.rs::MAX_DOC_BYTES` MUST be set to `1_048_576` (1 MiB).
- **FR-020-002**: `crates/player/src/queue.rs::MAX_CHAPTER_BYTES` MUST remain harmonized and equal to `MAX_DOC_BYTES` (`1_048_576`).
- **FR-020-003**: `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt::MAX_DOC_BYTES` MUST be set to `1_048_576`.
- **FR-020-004**: Any document or individual chapter exceeding `1_048_576` bytes MUST continue to fail closed with `InterruptionReason::FormatUndeliverable`.
- **FR-020-005**: All property-based tests and unit tests verifying document size limits MUST be updated to the new 1 MiB threshold.
