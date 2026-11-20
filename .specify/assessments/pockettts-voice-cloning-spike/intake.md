# Idea Intake: PocketTTS Voice Cloning Spike (020)

- **Slug**: pockettts-voice-cloning-spike
- **Created**: 2026-11-15 (artifact backfilled 2026-11-20)
- **Source**: `specs/020-pockettts-voice-cloning-spike/PLAN-80-20.md`, `specs/020-pockettts-voice-cloning-spike/measurements.md`
- **Type**: research/decision spike (device-first go/no-go)

## Idea (as captured)

> "Goal: decidir viabilidade de **voz clonada offline pt-BR** com **TTFA competitivo** no `2201117TY` (Redmi Note 11, SD680, Android 13) para mdreader." — `specs/020-pockettts-voice-cloning-spike/PLAN-80-20.md:3`

> "**GO** para spike de integração: no 2201117TY, com referência Sona 20 s e texto pt-BR curto/longo, **TTFA 1º áudio audível** (cold e warm, engine/voz em cache) ≤ ~1551 ms **e** RTF < 1 sustentado **e** RSS/estabilidade aceitáveis. **NO-GO**: latência/memória pior que baseline ou instável → manter Piper" — `PLAN-80-20.md` "Decisão gate"

## Restated

Before touching the production synthesis path, measure whether PocketTTS zero-shot cloning of the Ressoa pt-BR reference can meet Ressoa's TTFA/RTF/memory bar on the reference low-end Android device. Run the 80/20 plan (Phases 0–3), record all metrics in `measurements.md`, then either integrate behind a controlled voice extension (GO) or keep Piper and record cheaper fallbacks (NO-GO).

## Origin & Context

- **Raised by**: 020 80/20 device-first plan, 2026-11-15.
- **Trigger**: the Ressoa identity requires a pt-BR reading voice; PocketTTS + sherpa-onnx claim zero-shot pt support, but SD680 TTFA was unverified.
- **Baseline**: `play_to_audio_ms = 1551 ms` attested (T051/T052); SC-002 3000 ms.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: SD680 TTFA/RTF for PocketTTS pt — resolved in research/measurements: RTF 2.03–6.56, first chunk 6.7–26.1 s → NO-GO]
- [NEEDS CLARIFICATION: bundle ↔ sherpa 1.13.8 compatibility — still [NÃO VERIFICADO] beyond the executed PoC]
- [NEEDS CLARIFICATION: license/consent clearing for gated weights and reference voice — unresolved precondition, not assumed]
