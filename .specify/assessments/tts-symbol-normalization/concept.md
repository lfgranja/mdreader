# Concept: TTS Symbol Normalization

- **Slug**: tts-symbol-normalization
- **Created**: 2026-11-20
- **Recommended option**: Option A

## Options

### Option A — Pure deterministic inline normalization at the synthesis boundary
- **Sketch**: `normalize_for_speech(text, mode) -> String` in `crates/adapter-synthesis/src/normalize.rs`, called inside `synth.rs::synthesize` just before `generate_with_config`. Strip emphasis/heading/list/thematic markers in Fluid mode using collision-resistant placeholders to protect fenced/inline code, URLs/paths and simple math. `DocumentSegment.content` untouched.
- **Appetite**: small–medium
- **Trade-offs**: Removes marker fatigue with no engine/model change and no metadata risk. Cost is a hand-rolled inline normalizer (not an AST), maintained with property/invariant tests.
- **Rabbit holes**: protected-zone collisions, `~~~` vs ``` fences, pipe-table cells, intraword `a*b`; all covered by T009/T010 hardening.

### Option B — Use SSML / engine-level markdown mode
- **Sketch**: Ask sherpa-onnx/Piper to interpret SSML or markdown.
- **Appetite**: large/blocked
- **Trade-offs**: `SsmlLevel::None`; engine swap out of scope. Rejected.

### Option C — Do nothing
- **Sketch**: Keep verbatim synthesis.
- **Appetite**: none
- **Trade-offs**: Zero risk, zero value; listening fatigue persists.

## Recommendation

Option A. It is deterministic, host-testable, reversible, and preserves every resume/fingerprint/chunk invariant; already implemented and hardened per `converge.md`.

## Out of Scope (recommended option)

- SSML, engine/model swap, full AST parser, complete math/HTML normalization, `.txt` full normalization, UI mode toggle (follow-up).

## Assumptions to Validate

- Normalization is a pure function of `(text, mode)` (verified: `normalize_invariants.rs`).
- Fingerprint computed from raw bytes before normalization (verified: `session` flow, `converge.md`).
- Protected zones byte-identical across modes (verified: invariant tests).
