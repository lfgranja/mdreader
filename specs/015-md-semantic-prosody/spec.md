# Feature Specification: MD Semantic Prosody

**Feature Branch**: `015-md-semantic-prosody`
**Created**: 2026-11-08
**Status**: Converged (Cycle 017, 2026-11-10)
**Input**: `.specify/assessments/md-semantic-prosody/` (decision = go, defaults 80/20) + research.

## Problem
TTS reads markdown with uniform intonation. No semantic prosody. Desired: voice posture changes (pauses, rate, emphasis) per markdown semantics, natural human-like reading, without SSML.

## User Scenarios
### US1 — Structural pauses (P1)
Headings, paragraphs, lists, blockquotes, thematic breaks get appropriate pauses/boundaries.
### US2 — Rate variation (P2)
Headings and code slightly slower; emphasis slightly slower; questions/exclamations natural terminal.
### US3 — Protected zones (P1)
Code read literally (monotone, slow); no prosody exaggeration.

## Functional Requirements
- FR-PROS-001: Prosodic plan per segment (enum: Normal, Heading, Emphasis, Strong, Quote, Code, List, Question, Exclamation, ThematicBreak).
- FR-PROS-002: Plan derived from markdown semantics in chunk.rs or a new module, preserving SourceRange/resume/fingerprint.
- FR-PROS-003: Speed factor applied via GenerationConfig.speed per segment in synth.rs.
- FR-PROS-004: Thematic break → long pause (segment boundary or "." placeholder).
- FR-PROS-005: Conservative defaults (conservadora); no theatricality.
- FR-PROS-006: Coordinated with tts-symbol-fatigue: classify structure → derive prosodic plan → generate spoken text (normalization). Shared protected zones.
- FR-PROS-007: .md scope; .txt neutral.
- FR-PROS-008: Deterministic.

## Non-Goals
SSML, engine swap, full AST, per-word pitch/volume (beyond speed), user config beyond default (future).

## Success Criteria
- SC-PROS-001: Headings/code get distinct speed factors in synthesized output.
- SC-PROS-002: Thematic breaks produce pause.
- SC-PROS-003: Invariants preserved.
- SC-PROS-004: Device listening test (optional).