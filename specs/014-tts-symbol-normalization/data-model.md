# Data Model: 014 TTS Symbol Normalization

## New types
- `NormalizationMode`: enum { `Fluid`, `Literal` }. Default `Fluid`. Derive Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize.
- `normalize_for_speech(text: &str, mode: NormalizationMode) -> String`.

## Unchanged
- `DocumentSegment`: content, source_range, segment_index, resume_point, byte_length — untouched.
- `SourceRange`, fingerprint (blake3) — untouched.
- CLI chunk payload — metadata only, no content.

## Invariants
- Normalization is pure function of (text, mode); no I/O, no state.
- Idempotent: normalize(normalize(x, m), m) == normalize(x, m).
- Protected substrings byte-identical in Fluid mode.