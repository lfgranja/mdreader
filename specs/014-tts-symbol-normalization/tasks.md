# Tasks: 014 — TTS Symbol Normalization

- [x] T001 Create `crates/adapter-synthesis/src/normalize.rs` with `NormalizationMode` and `normalize_for_speech`.
- [x] T002 Export from `lib.rs`.
- [x] T003 RED tests: emphasis, headings, bullets, thematic breaks, protected zones, idempotency. (7 tests passing)
- [x] T004 Wire normalization into `synth.rs::synthesize`.
- [x] T005 Mode toggle plumbing (settings): `set_normalization_mode` command + `put/get_normalization_mode` persistence + seed on startup. UI toggle recorded as follow-up (no settings screen in v1).
- [x] T006 Invariant tests: SourceRange/resume/fingerprint unchanged (`crates/adapter-synthesis/tests/normalize_invariants.rs`).
- [x] T007 Host gates: fmt, nextest, clippy (evidence in `converge.md`).
- [x] T008 Converge + evidence (`converge.md`).
- [x] T009 Harden `normalize.rs` against the 2026-11-10 research gaps: links/images reduced to label/alt, blockquote markers, strikethrough, raw HTML tags, intraword `a*b`, pipe-table rows, dead code removed, collision-resistant placeholders.
- [x] T010 (2026-11-15) Harden Fluid normalization for spoken asterisk leaks: recognize `~~~` fenced blocks as protected verbatim and normalize pipe-table data cells with protected inline zones preserved (delimiter rows remain pause). Added `normalize.rs` unit regressions and extended `normalize_invariants.rs`; host verification recorded in `converge.md`, device listening `[NV]`.