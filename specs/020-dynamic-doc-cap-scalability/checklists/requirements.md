# Requirements Checklist: 020-dynamic-doc-cap-scalability

- [x] CHK001: Rust `MAX_DOC_BYTES` updated to `1_048_576` in `crates/player/src/interruption.rs`.
- [x] CHK002: Rust `MAX_CHAPTER_BYTES` harmonized with `MAX_DOC_BYTES` in `crates/player/src/queue.rs`.
- [x] CHK003: Kotlin `DocIngest.MAX_DOC_BYTES` updated to `1_048_576` in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt`.
- [x] CHK004: All test suites and property-based tests pass with zero regressions.
- [x] CHK005: Gates 1 to 8 in `./scripts/verify.sh` pass cleanly (`VERIFY_EXIT=0`).
