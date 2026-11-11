# Implementation Plan: 020-dynamic-doc-cap-scalability

- **Feature**: 020-dynamic-doc-cap-scalability
- **Date**: 2026-11-11

## 1. Scope of Changes
1. **Rust Core**:
   - Update `MAX_DOC_BYTES` in `crates/player/src/interruption.rs`.
   - Update `crates/session/src/cli.rs` documentation references if applicable.
2. **Kotlin Shell**:
   - Update `MAX_DOC_BYTES` in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt`.
3. **Tests**:
   - Verify `crates/player/tests/prop_invalid_chapter_skipping.rs` compiles and passes.
4. **Gates**:
   - Run `./scripts/verify.sh` to ensure Gates 1 through 8 remain 100% green.
