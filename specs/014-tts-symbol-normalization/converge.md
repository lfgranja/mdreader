# Converge: 014 TTS Symbol Normalization

Status: implemented (host). `normalize.rs` + wiring in `synth.rs`. 7 unit tests
passing. Clippy clean. Fmt clean. Full adapter-synthesis suite: 15 integration +
7 unit tests pass. Mode toggle backend plumbing (T005) done; UI toggle recorded
as follow-up. Invariant tests (T006) added. Device listening test (SC-SYM-005)
optional/deferred. T232/T233 untouched. R4/R7 not claimed.

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
