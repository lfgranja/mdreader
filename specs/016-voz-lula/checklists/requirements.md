# Specification Quality Checklist: 016-voz-lula

**Purpose**: Validate specification completeness and quality before planning/implementation
**Created**: 2026-11-20
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] Focused on user/system value (selectable pt-BR homage voice; core TTS function)
- [x] Grounded in evidence (faber asset/handoff convention; engine error semantics)
- [x] All mandatory sections completed (US1–US2, FR-01..05, SC-001..005, assumptions)
- [x] Scope bounded (second explicit voice; faber remains default; offline-first)

## Requirement Completeness

- [x] Requirements testable (CLI/bridge selection; fail-closed init; handoff format)
- [x] Success criteria measurable (build/test/clippy/fmt; `InitFailed` on missing `.onnx`)
- [x] Acceptance scenarios cover selection + handoff format + fail-closed
- [x] Edge cases identified (missing `.onnx`; no silent resample/downmix)
- [x] Dependencies/assumptions identified (Piper VITS assets; espeak-ng pt-BR; offline-first)

## Feature Readiness

- [x] FR-01..05 map to the design in `plan.md` and assessment decision (`go` for specification)
- [x] Risks stated: model provenance (Q1), espeak-ng tree (Q2), legal/ethical exposure, constitution single-voice gate
- [x] Rollback constraint: no asset/code changes authorized by the assessment `go`

## Status (honest)

- [~] Design artifacts complete; **implementation DEFERRED** per `decision.md` (2026-11-10, verdict `defer`).
- [ ] No `.onnx`/`tokens.txt` in repo; placeholder tree removed (broke `exactly_one_bundled_voice_directory`).
- [ ] SC-001..005 not demonstrated (no code changes); device/selection tests `[NV]`.

## Notes

- Assessment `go` authorizes specification only; implementation prerequisites (consent/licensing/legal review, constitution amendment + `test_noscope` update, store disclosure) are recorded in `.specify/assessments/voz-lula/decision.md`.
- Failure mode MUST be `EngineError::InitFailed` when the model asset is absent — never silent fallback.
