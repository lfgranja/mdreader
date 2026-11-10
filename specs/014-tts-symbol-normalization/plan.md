# Plan: 014 TTS Symbol Normalization

## Implementation order (TDD)
1. Add normalization module + unit tests (RED).
2. Wire into synth.rs boundary (GREEN).
3. Protected zones + thematic break pause.
4. Mode toggle (if in scope) else fixed Fluid.
5. Integration tests (range/resume/fingerprint invariants).
6. Host gates.

## Critical files
- `crates/adapter-synthesis/src/normalize.rs` (new)
- `crates/adapter-synthesis/src/synth.rs` (boundary wiring)
- `crates/adapter-synthesis/src/chunk.rs` (unchanged; metadata preserved)
- `crates/adapter-synthesis/src/lib.rs` (export normalize + NormalizationMode)
- tests: `crates/adapter-synthesis/tests/`

## Approach
- New module with `NormalizationMode { Fluid, Literal }` and `normalize_for_speech`.
- Fluid: regex/heuristic transforms for emphasis, headings, bullets, thematic breaks; protected zones via scanning.
- Literal: identity.
- Wire: in `synthesize`, compute `spoken = normalize_for_speech(&segment.content, mode)`; pass `&spoken` to `generate_with_config`. Keep segment.content for ranges.
- Mode default Fluid; toggle deferred to follow-up if settings plumbing not trivial (record in tasks).
- Thematic break: convert `***`/`---`/`___` line to empty/pause (segment boundary) — handled by chunking? If chunk produces a segment for the break, normalize to empty → synth fails (empty). Instead, normalize thematic break to a short placeholder or skip segment. Decision: normalize thematic-break-only segment content to "" and skip synthesis in synth (return empty samples is InitFailed). Better: handle in chunking by merging break into adjacent segment as pause (".")? Simpler: normalize `***` line to a period "." to create a brief pause. Use "." for thematic breaks.
- Idempotency: normalize(normalize(x)) == normalize(x).

## Non-goals
SSML, engine swap, full AST, .txt full normalization, settings UI (unless trivial).