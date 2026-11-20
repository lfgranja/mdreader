# Idea Intake: 016-voz-lula — TTS voice honouring Lula da Silva

- **Slug**: voz-lula
- **Created**: 2026-11-20
- **Source**: `specs/016-voz-lula/spec.md` (requirement recorded 2026-11-08) + `specs/016-voz-lula/decision.md`
- **Type**: feature / differentiator

## Idea (as captured)

> "Requirement to implement reading using a voice that honours Brazilian President Luiz Inácio Lula da Silva in the mdreader (Ressoa) TTS audiobook reader." — `specs/016-voz-lula/spec.md`
> "Quero selecionar uma voz de síntese que homenageie o Presidente Luiz Inácio Lula da Silva, para que a leitura de livros audiobooks use uma voz brasileira reconhecível e respeitosa." — US1

## Restated

Offer a second, explicitly selectable Piper VITS voice (`lula`) alongside the default `faber`, delivering mono f32 @ 22 050 Hz handoff and failing closed (`EngineError::InitFailed`) when its `.onnx` asset is absent, configurable via CLI (`--voice`) and Tauri bridge (`set_voice`).

## Origin & Context

- **Raised by**: product requirement, 2026-11-08; captured as spec 016 with spec/plan/tasks.
- **Trigger**: voice synthesis is the app's core function; a recognizable, respectful Brazilian voice is a cultural/accessibility differentiator.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: model provenance (training vs. donation) — see research.md; asset absent from repo.]
- [NEEDS CLARIFICATION: legal/ethical consent and constitution single-voice gate — see research.md.]
