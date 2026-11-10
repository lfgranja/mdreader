# Plan: 015 MD Semantic Prosody

## Implementation order (TDD)

1. `prosody.rs`: `ProsodyPlan` enum + `plan_for_segment` (pure, deterministic).
2. Export from `lib.rs`.
3. Wire `GenerationConfig.speed` into `synth.rs::synthesize`.
4. Unit tests for the classification and the speed map.
5. Invariant tests (`SourceRange`/resume/fingerprint unchanged).
6. Host gates.

## Critical files

- `crates/adapter-synthesis/src/prosody.rs` (new)
- `crates/adapter-synthesis/src/synth.rs` (boundary wiring)
- `crates/adapter-synthesis/src/chunk.rs` (unchanged; metadata preserved)
- `crates/adapter-synthesis/src/lib.rs` (export)
- tests: `crates/adapter-synthesis/tests/prosody_unit.rs`,
  `crates/adapter-synthesis/tests/prosody_invariants.rs`

## Approach

- Classify the segment from its raw markdown semantics BEFORE normalization
  (structure first), then derive a conservative speed factor.
- Apply the factor via `GenerationConfig.speed`; no SSML, no per-segment
  pitch/volume (not exposed by sherpa-onnx 1.13.8).
- Conservative map: `Code` 0.85, `Heading`/`Strong` 0.90,
  `Emphasis`/`Quote`/`List` 0.95, others 1.00.
- Preserve `DocumentSegment` metadata: the plan never mutates the segment.

## Non-goals

SSML, engine swap, full AST, per-word pitch/volume, user config beyond the
conservative default (future opt-in).
