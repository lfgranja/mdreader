# Converge: 014 TTS Symbol Normalization

Status: implemented (host), hardened 2026-11-15. `normalize.rs` + wiring in
`synth.rs`. `normalize.rs` unit tests: 17 passing; `normalize_invariants.rs`: 8
passing. Mode toggle backend plumbing (T005) done; UI toggle recorded as
follow-up. Invariant tests (T006) extended. Device listening test (SC-SYM-005)
optional/deferred (`[NV]`). T232/T233 untouched. R4/R7 not claimed.
## Correction (2026-11-10) — MAX_CHUNK_BYTES claim

An earlier revision of this record claimed `MAX_CHUNK_BYTES` was changed from
5000 to 2000. **That claim was false**: the current tree has
`pub const MAX_CHUNK_BYTES: usize = 5_000` (`crates/adapter-synthesis/src/chunk.rs`)
and `prop_chunk::max_chunk_bytes_is_exactly_5000` still asserts 5000. No such
change is present in the working tree or in HEAD. The 2000-byte experiment was
either reverted or never committed; the record is corrected here rather than
left to imply a code state that does not exist. TTFA stabilization for spec 013
remains a device-blocked, separate concern.

## Invariants (T006)

`normalize_for_speech` is a pure function of `(text, mode)` applied only to the
TTS input inside `synth.rs::synthesize`; it never mutates `DocumentSegment`.
`crates/adapter-synthesis/tests/normalize_invariants.rs` asserts:

- segment count, `source_range`, `segment_index`, `resume_point`, `byte_length`
  are identical across Fluid and Literal modes for the same document;
- Fluid changes the spoken text while Literal is the identity;
- the blake3 document fingerprint is computed from the raw bytes by `session`
  before any normalization, so normalization cannot alter it.

## Hardening (2026-11-15) — table data cells and tilde fences

Host-side 80/20 hardening to reduce spoken Markdown marker leaks reported as
repeated "asterisco" in normal text while preserving 014 protected-zone and
metadata invariants:

- `crates/adapter-synthesis/src/normalize.rs`: `normalize_for_speech` now treats
  `~~~` fence delimiters as protected/verbatim (parity with ``` fences). Inline
  normalization was refactored into shared helpers; pipe-table delimiter rows
  remain pauses (`"."`), while pipe-table data cells are normalized (label/link
  reduction, protected inline code/URL/inline math and simple asterisk
  multiplication/intraword protections, HTML/strikethrough/emphasis stripping,
  orphan-`*` removal, zone restoration) and joined with `", "`. Structural
  line transforms are not applied inside table cells.
- Regression coverage: `normalize.rs` unit tests
  `tilde_fenced_code_is_verbatim` and
  `normalizes_table_data_cells_and_preserves_protected_zones`; extended
  `normalize_invariants.rs` (8 tests) for table-data marker stripping outside
  protected zones and tilde-fence verbatim behavior.
- Verification (2026-11-15, host): targeted normalization tests passed (3
  selected tests); `cargo nextest run -p adapter-synthesis -E 'not test(/arbitrary_documents_read_identically_from_stdin_and_input/)'`
  passed (170 tests run, 1 skipped, 4 slow). Full adapter-synthesis run hit an
  unrelated environment failure in
  `adapter-synthesis::cli_protocol arbitrary_documents_read_identically_from_stdin_and_input`
  (`proptest` temp-write `No space left on device (os error 28)` with `/tmp`
  ~99% full); no spurious regression artifact retained.
  `cargo clippy -p adapter-synthesis -p player -p session --all-targets -- -D warnings`
  passed. `scripts/expected-tests.txt` updated for 3 added normalization targets.
- Limitations: protected zones (fenced/inline code, URLs, declared math/protected
  forms) remain byte-identical/verbatim by contract and may still be spoken by
  the engine; device listening/SC-SYM-005 remains `[NV]`. Segment metadata,
  fingerprint boundaries, and `split_document` behavior unchanged.
