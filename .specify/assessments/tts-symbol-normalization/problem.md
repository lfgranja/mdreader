# Problem Definition: TTS Symbol Normalization

- **Slug**: tts-symbol-normalization
- **Created**: 2026-11-20
- **Inputs used**: intake.md | research.md

## Problem Statement

`adapter-synthesis::synth::synthesize` sends `segment.content` verbatim to Piper `faber`/eSpeak-NG, which has no Markdown mode: emphasis, headings, bullets and thematic breaks are spoken literally, producing listener fatigue. With SSML unavailable, a deterministic plain-text normalization is needed at the synthesis boundary without touching segment/range/resume/fingerprint contracts.

## Affected Users & Stakeholders

- **Users**: Markdown audiobook readers — need natural spoken text without repeated marker noise.
- **Stakeholders**: maintainer — owns synthesis adapter, chunk/resume/fingerprint invariants and constitution gates.

## Goals

- Add an idempotent `normalize_for_speech(text, mode)` in `adapter-synthesis` (FR-SYM-001), applied only to spoken text in `synth.rs::synthesize` (FR-SYM-002).
- Strip Markdown markers in Fluid default while preserving protected zones (FR-SYM-003/004) and all metadata (FR-SYM-007/008).
- Keep output deterministic (FR-SYM-009) and scope to `.md` (FR-SYM-006).

## Non-Goals

- No SSML, engine/model swap, full Markdown AST parser, complete math/HTML normalization, or full `.txt` normalization.
- No changes to `chunk.rs` metadata projection, `SourceRange`/`resume_point`/`segment_index`/`byte_length` or blake3 fingerprint flow.

## Success Metrics

- Corpus in Fluid mode contains no `*`/`#`/bullet markers in normalized TTS input (SC-SYM-001).
- Protected-zone substrings byte-identical; idempotency property test passes (SC-SYM-002/003).
- Range/resume/fingerprint invariants preserved in host tests (SC-SYM-004); device listening (SC-SYM-005) optional `[NV]`.

## Cost of Inaction

The app's core TTS function keeps producing fatiguing literal marker speech with no SSML mitigation path.

## Open Questions

- Literal/Verbatim mode settings toggle (FR-SYM-005) deferred as a follow-up; fixed Fluid default shipped.
