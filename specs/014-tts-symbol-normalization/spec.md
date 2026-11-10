# Feature Specification: TTS Symbol Normalization (markdown → speech)

**Feature Branch**: `014-tts-symbol-normalization`
**Created**: 2026-11-08
**Status**: Converged (Cycle 017, 2026-11-10)
**Input**: `.specify/assessments/tts-symbol-fatigue/` (decision = go, 2026-11-08) + deep research.

## Problem / Evidence Summary

Ressoa/mdreader is a TTS audiobook reader for Markdown. Today `adapter-synthesis::synth::synthesize` passes `segment.content` verbatim to Piper `faber` (eSpeak-NG phonemizer). eSpeak/Piper has no markdown mode, so `*`, `**`, `***`, `#`, bullets, thematic breaks are read literally ("asterisco, asterisco, asterisco"), causing listening fatigue (user report 2026-11-07). SSML is unavailable (`SsmlLevel::None`), so normalization must be deterministic plain-text transformation in Rust, preserving all segment/range/resume/fingerprint contracts.

## User Scenarios & Testing

### US1 — Fluid default (P1)
As a reader, when a document with markdown emphasis/lists/headings/thematic breaks is played, I hear natural speech without repeated "asterisco".

Independent Test: synthesize a representative markdown doc in Fluid mode; assert no `*`/`#`/bullet markers remain in the spoken text; listen for fatigue reduction.

Acceptance:
1. Given `*itálico*`, `**negrito**`, `***`, `# heading`, `* item`, when synthesized in Fluid mode, Then markers are stripped/converted and inner text preserved.
2. Given the same doc, Then no `*` or `#` characters reach the TTS input.

### US2 — Protected zones (P1)
As a reader, code, URLs, and simple math are read verbatim (not mangled).

Acceptance:
1. Given fenced/inline code, URLs/paths, `2*3`, when normalized, Then those substrings are unchanged.

### US3 — Thematic break pause (P2)
As a reader, a thematic break (`---`/`***`) produces a brief pause/section boundary, not "dash dash dash".

### US4 — Literal mode (P2, if toggle enabled)
As a technical reader, I can choose Literal/Verbatim mode to hear markdown as-is.

## Functional Requirements

- **FR-SYM-001**: Deterministic normalization function `normalize_for_speech(text, mode) -> String` in `adapter-synthesis`, idempotent.
- **FR-SYM-002**: Applied at synthesis boundary in `synth.rs::synthesize` (before `generate_with_config`); `DocumentSegment.content` unchanged.
- **FR-SYM-003**: Fluid mode (default): strip emphasis markers (`*`, `_`), heading markers (`#`), list bullets, thematic-break markers (convert to pause/boundary); keep inner text.
- **FR-SYM-004**: Protected zones (verbatim): fenced code, inline code, URLs/paths, simple math patterns.
- **FR-SYM-005**: Mode configurable: Fluid default + Literal optional (settings toggle). If settings plumbing out of scope for first cut, fixed Fluid with documented behavior and follow-up task.
- **FR-SYM-006**: `.md` scope only; `.txt` minimal normalization (URLs maybe).
- **FR-SYM-007**: Preserve `SourceRange`, `resume_point`, `segment_index`, `byte_length`, fingerprint (normalization only affects spoken text).
- **FR-SYM-008**: CLI chunk metadata unchanged (chunk projects segment metadata only, never content).
- **FR-SYM-009**: Determinism: same input + mode + version → same output.

## Non-Goals
- SSML (SsmlLevel::None).
- Engine/model swap.
- Full markdown AST parser.
- Complete math/HTML normalization.
- .txt full normalization.

## Success Criteria
- SC-SYM-001: Corpus (representative markdown) in Fluid mode contains no `*`/`#`/bullet markers in normalized TTS input.
- SC-SYM-002: Protected-zone substrings byte-identical after normalization.
- SC-SYM-003: Idempotency property test passes.
- SC-SYM-004: Range/resume/fingerprint invariants preserved (host tests).
- SC-SYM-005: Device listening test confirms reduced fatigue (subjective, optional).
EOF
echo written