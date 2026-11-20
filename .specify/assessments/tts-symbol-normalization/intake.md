# Idea Intake: TTS Symbol Normalization (markdown → speech)

- **Slug**: tts-symbol-normalization
- **Created**: 2026-11-20
- **Source**: user report 2026-11-07 (markdown markers read literally by eSpeak/Piper) + `.specify/assessments/tts-symbol-fatigue/` + `specs/014-tts-symbol-normalization/spec.md`
- **Type**: improvement

## Idea (as captured)

> "eSpeak/Piper has no markdown mode, so `*`, `**`, `***`, `#`, bullets, thematic breaks are read literally ('asterisco, asterisco, asterisco'), causing listening fatigue." — `specs/014-tts-symbol-normalization/spec.md` (Problem / Evidence Summary)
> "SSML is unavailable (`SsmlLevel::None`), so normalization must be deterministic plain-text transformation in Rust, preserving all segment/range/resume/fingerprint contracts." — same

## Restated

Introduce a deterministic, idempotent `normalize_for_speech(text, mode)` in `adapter-synthesis`, applied at the synthesis boundary so spoken text drops Markdown marker noise while segment metadata, ranges, resume anchors and fingerprints stay byte-identical.

## Origin & Context

- **Raised by**: user listening-fatigue report, 2026-11-07; recorded as spec 014 with existing `tts-symbol-fatigue` assessment (`go`, 2026-11-08).
- **Trigger**: `adapter-synthesis::synth::synthesize` passes `segment.content` verbatim to Piper `faber` (eSpeak-NG phonemizer) with no markdown awareness.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: Literal/Verbatim mode plumbing — resolved in research: fixed Fluid default first, settings toggle as follow-up (FR-SYM-005).]
