# Idea Research: PocketTTS Voice Cloning Spike (020)

- **Slug**: pockettts-voice-cloning-spike
- **Created**: 2026-11-15 (artifact backfilled 2026-11-20)
- **Evidence confidence (overall)**: high for negative result (on-device matrix measured), medium for bundle/runtime compatibility and licensing

## Users & Demand

- Ressoa readers need a calm pt-BR reading voice; clonedvoice is one candidate path to a distinctive Ressoa voice — [source: `PLAN-80-20.md` Contexto] (confidence: high as motivation)
- Any new voice must not regress offline TTFA on low-end devices (SC-002 / 1551 ms baseline) — [source: `PLAN-80-20.md` Decisão gate; `measurements.md` Bounds] (confidence: high)

## Prior Art

- Baseline Piper/sherpa `play_to_audio_ms = 1551 ms` attested (T051/T052); direct debug-APK run recorded 14059 ms — [source: `measurements.md` Baseline Piper] (confidence: high)
- Host Pocket-TTS 4.0.1 (EN) warm TTFA ~267–414 ms with high one-time costs (load ~2.85 s, voice state ~6.97 s) — [source: `PLAN-80-20.md` Contexto] (confidence: medium — host only)
- Sherpa-onnx 1.13.8 exposes Pocket structs and `generateWithCallback`, but the safe binding used by mdreader is VITS-only — [source: `PLAN-80-20.md` Contexto] (confidence: high)
- Public SD845 benchmark: RTF ~0.94–1.06, first-chunk TTFA unreported; SD680 unverified — [source: `PLAN-80-20.md` Web] (confidence: medium)
- Reference asset prepared: `tts-work/voices/ressoa/reference_30s_24k.wav` (PCM16 mono 24 kHz, 29.88 s, SHA256 `ca471e2f…7e52a0`) — [source: `measurements.md` Contexto] (confidence: high, hash recorded)

## Market & Context

- Cost of doing nothing: Ressoa keeps the single Piper voice; no cloned-voice differentiation. Cost of integrating a slow model: unacceptable TTFA on the reference device, directly violating the project's core UX promise.

## Data & Constraints (measured, `measurements.md`)

- Device: 2201117TY / SM6225 / SD680 / aarch64 / SDK 33; harness C-API v1.13.8, NDK r29.
- 9 configs × cold/warm: `model_load_ms` 2994–3937; `read_ref_ms` 16.6–29.2; cold `first_chunk_ms` 9804–26113; warm 6690–10599; `RTF` 2.035–6.565 (always > 1); `VmHWM` peak ~1.18–1.30 GB (RSS returns to ~34–55 MB after free).
- Host proxy: load ~0.96–1.05 s, first chunk 4.32–4.65 s, RTF 1.14–1.20 — supplementary only.
- Gate: ≤ ~1551 ms (and 3000 ms bound), `RTF < 1`, acceptable RSS → all measured dimensions fail.
- Item 2 (end-to-end `play_to_audio_ms`) not executed; synthesis-only metric excludes app pipeline/player; end-to-end can only be worse [INFERENCE].
- License: Kyutai/HF gated weights, third-party voice and video-extracted reference require explicit clearing.

## Evidence Against the Idea

- Strong direct evidence against integration on this device/runtime/model profile: every config exceeds both TTFA bounds and `RTF < 1`, with ~1.2 GB peak memory. (confidence: high)
- Potential counter-evidence (not obtained): a compatible quantized bundle/newer runtime on stronger hardware might change the result; explicitly [NV]. (confidence: medium)

## Gaps & Open Questions

- Bundle ↔ runtime exact compatibility [NÃO VERIFICADO]; newer bundles may need sherpa ≥2.0.1 + BOS fix.
- License/consent clearing unresolved; no distribution implied.
- Device end-to-end TTFA [NV] and out of scope after NO-GO.

## Sources

- `specs/020-pockettts-voice-cloning-spike/PLAN-80-20.md` (host: local repo, policy: allowlisted path)
- `specs/020-pockettts-voice-cloning-spike/measurements.md` (host: local repo)
- `tts-work/voices/ressoa/voice.json` (host: local repo)
