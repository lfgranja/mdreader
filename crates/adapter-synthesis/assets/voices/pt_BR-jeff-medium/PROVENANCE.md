# Provenance — Piper pt-BR `jeff` voice (medium)

Bundled voice asset for the `offline_default` synthesis
engine (`crates/adapter-synthesis`) as the third voice in the
spec 018 closed voice catalog (custom-voices assessment,
Option A). Loaded at synthesis time through bundled/packaged
voice roots resolved at runtime (never fetched from network at
synthesis time; constitution Principle IX, offline-first; no
`INTERNET`).

## Source

- **ONNX model (sherpa-onnx compatible, embedded metadata):**
  `https://huggingface.co/csukuangfj/vits-piper-pt_BR-jeff-medium`
  (file `pt_BR-jeff-medium.onnx`, downloaded 2026-11-14 via
  `resolve/main/pt_BR-jeff-medium.onnx`).
- **Config:** `https://huggingface.co/csukuangfj/vits-piper-pt_BR-jeff-medium`
  (file `pt_BR-jeff-medium.onnx.json`, snapshot 2026-11-14).
- **Tokens:** `https://huggingface.co/csukuangfj/vits-piper-pt_BR-jeff-medium`
  (`tokens.txt`, sherpa-onnx/Piper form, downloaded 2026-11-14).
- **Model card:** `https://huggingface.co/csukuangfj/vits-piper-pt_BR-jeff-medium`
  (file `MODEL_CARD`).
- **Model family:** VITS, Piper pt-BR, medium quality, 1 speaker.
- **Runtime compatibility:** Piper VITS consumable by pinned
  `sherpa-onnx = "=1.13.8"` (ONNX Runtime 1.28.2).

## License

**CC0** — as stated by the upstream `MODEL_CARD`:

> ## Dataset
> * URL: <https://github.com/OHF-Voice/voice-datasets>
> * License: CC0

`MODEL_CARD` is embedded verbatim alongside the assets.

## Curation note (non-imitative)

Curated as an additional pt-BR reading voice with thematic
"Sona-inspired feel" framing only. No League of Legends, Riot,
performer, or character imitation/branding is claimed or
implied by this voice id, label, or asset.

## Files

| File | Purpose |
|---|---|
| `pt_BR-jeff-medium.onnx` | VITS model (mono, 22,050 Hz, sherpa-onnx compatible). |
| `pt_BR-jeff-medium.onnx.json` | Model config — `audio.sample_rate = 22050`, `espeak.voice = "pt-br"`, `phoneme_type = "espeak"`, `num_symbols = 256`, `num_speakers = 1`, inference defaults (`noise_scale` 0.667, `noise_w` 0.8, `length_scale` 1.0). |
| `tokens.txt` | Token map for sherpa-onnx/Piper synthesis. |
| `MODEL_CARD` | Upstream model card (license + metadata). |

Phonemizer data: resolved via shared bundled
`assets/voices/shared/espeak-ng-data/` (preferred, espeak voice
`pt-br` compatible with faber/cadu/jeff).

## Integrity (SHA-256)

| File | SHA-256 |
|---|---|
| `pt_BR-jeff-medium.onnx` | `e5a8c799ee3782114eb788999a97f58fac5994f5444777d807d8930cb4c1cad7` |
| `pt_BR-jeff-medium.onnx.json` | `7bf8145b572b36806f5ce0f1d3322b6711975bc7d0473e8d36fced4a9ec0030d` |
| `tokens.txt` | `e8a50ae0c75612d18cfcf3f90800dca79f961e3ed97f9574f3535194469187fd` |
| `MODEL_CARD` | `acdea1b233a478a8ce3bac1f4886a14c8bb6a6287cefb41a26a657a3a7418b08` |

- `pt_BR-jeff-medium.onnx`: 62914560 bytes.
- Declared sample rate 22,050 Hz matches handoff contract
  (`engine_core::format::DeclaredFormat::SAMPLE_RATE`); mismatch
  is a named `EngineError::RateMismatch`, never silent resample
  (FR-024b / SC-035).
