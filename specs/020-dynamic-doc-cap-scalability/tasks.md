# Tasks: 020-dynamic-doc-cap-scalability

- [x] T001: Update `MAX_DOC_BYTES` in `crates/player/src/interruption.rs` from 93,636 to 1,048,576.
- [x] T002: Update `MAX_DOC_BYTES` in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` from 93,636 to 1,048,576.
- [x] T003: Update comments and references in `crates/session/src/cli.rs`.
- [x] T004: Run test suite via `cargo nextest run --workspace`.
- [x] T005: Execute `./scripts/verify.sh` to confirm Gates 1 to 8.
- [x] T006: Document convergence report in `specs/020-dynamic-doc-cap-scalability/converge.md`.
