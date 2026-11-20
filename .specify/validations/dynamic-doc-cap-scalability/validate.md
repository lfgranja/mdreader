# Validation (R4): 020 Dynamic Document Cap Scalability

- **Date**: 2026-11-20 (artifact backfilled from `specs/020-dynamic-doc-cap-scalability/`)
- **Status**: static checks PASS (measured from source); dynamic gates [NV]

| Check | Command / Evidence | Result |
|---|---|---|
| Spec quality | `specs/020-dynamic-doc-cap-scalability/checklists/requirements.md` | PASS |
| Assessment gate | `.specify/assessments/dynamic-doc-cap-scalability/decision.md` verdict `go` | PASS |
| Task completion | `specs/020-dynamic-doc-cap-scalability/tasks.md` — 6 `[x]`, 0 `[ ]` | PASS |
| FR-020-001 | `crates/player/src/interruption.rs:15` → `pub const MAX_DOC_BYTES: usize = 1_048_576;` | PASS (measured) |
| FR-020-002 | `crates/player/src/queue.rs:227` → `MAX_CHAPTER_BYTES = crate::interruption::MAX_DOC_BYTES` | PASS (measured) |
| FR-020-003 | `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt:85` → `const val MAX_DOC_BYTES = 1048576` | PASS (measured) |
| FR-020-004 | `DocIngest.kt` refusal path for `bytes.size > MAX_DOC_BYTES` → `REFUSAL_FORMAT_UNDELIVERABLE`; queue oversize chapter error | PASS (measured) |
| FR-020-005 | `crates/player/tests/prop_invalid_chapter_skipping.rs:27-28` asserts `MAX_CHAPTER_BYTES == MAX_DOC_BYTES == 1_048_576` | PASS (measured) |
| Memory bound | Spec 017 `AudioChunkCache` 32 MiB remains the runtime buffer bound | PASS (design documented) |
| Host gates 1–8 | `PROPTEST_CASES=32 bash scripts/verify.sh` | [NV] — gate owner |
| expected-tests census | `cargo nextest list --workspace` vs `scripts/expected-tests.txt` | [NV] — gate owner |

## Success criteria

- SC (implicit): documents/chapters up to 1 MiB accepted; above cap fails closed with `FormatUndeliverable`; Rust and Kotlin caps harmonized; property/unit tests updated. Static evidence measured above; runtime/gate confirmation [NV].

## [NV]

- Full host gate execution and census diff; no on-device memory stress measurement performed in this artifact.
