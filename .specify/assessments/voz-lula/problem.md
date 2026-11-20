# Problem Definition: 016-voz-lula

- **Slug**: voz-lula
- **Created**: 2026-11-20
- **Inputs used**: intake.md | research.md

## Problem Statement

The synthesis adapter currently offers a single default voice (`faber`, CC0). There is no explicitly selectable Brazilian voice honouring President Lula da Silva; the app therefore cannot offer the requested cultural/accessibility differentiator, and no fail-closed voice-selection surface exists for a second voice.

## Affected Users & Stakeholders

- **Users**: Brazilian readers wanting a recognizable, respectful pt-BR reading voice.
- **Stakeholders**: maintainer (voice assets/gates), legal/compliance (synthetic voice of a public figure), store review.

## Goals

- Support selecting between `faber` (default) and `lula` via CLI `--voice` and Tauri `set_voice` (FR-01/FR-04).
- Reuse the canonical mono f32 @ 22 050 Hz handoff with no silent resample (FR-02).
- Fail closed with `EngineError::InitFailed` when the Lula `.onnx` asset is absent (FR-03).
- Follow the faber asset-directory convention `assets/voices/pt_BR-lula-medium/` (FR-05).

## Non-Goals

- No network fetch at synthesis time; no change to the default `faber` voice.
- No model training/donation within this artifact set; no constitution/gate change here (those are implementation prerequisites).

## Success Metrics

- SC-001..004: workspace build/test/clippy/fmt clean with Lula voice support (even without `.onnx`).
- SC-005: selecting `lula` with absent `.onnx` yields `InitFailed` (fail-closed).

## Cost of Inaction

No culturally distinctive pt-BR voice option; the core reading value is unaffected, so the cost is a missed differentiator rather than a functional regression.

## Open Questions

- Q1 model provenance; Q2 espeak-ng tree reuse; consent/licensing/constitution prerequisites (research.md + `specs/016-voz-lula/decision.md`).
