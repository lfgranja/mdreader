# Provenance — Piper pt-BR `cadu` voice (medium)

Bundled voice asset for the `offline_default` synthesis
engine (`crates/adapter-synthesis`) as the curated alternative
in the spec 018 closed voice catalog (Option B). Loaded at
synthesis time through bundled/packaged voice roots resolved at
runtime (never fetched from network at synthesis time;
constitution Principle IX, offline-first; no `INTERNET`).

## Source

- **ONNX model (sherpa-onnx compatible, embedded metadata):**
  `https://huggingface.co/csukuangfj/vits-piper-pt_BR-cadu-medium`
  (file `pt_BR-cadu-medium.onnx`, downloaded 2026-11-12 via
  `resolve/main/pt_BR-cadu-medium.onnx?download=true`).
- **Config:** `https://huggingface.co/rhasspy/piper-voices`,
  path `pt/pt_BR/cadu/medium/pt_BR-cadu-medium.onnx.json`
  (snapshot 2026-11-12, `resolve/main`).
- **Tokens:** `https://huggingface.co/csukuangfj/vits-piper-pt_BR-cadu-medium`
  (`tokens.txt`, sherpa-onnx/Piper form, downloaded 2026-11-12).
- **Model card:** `https://huggingface.co/rhasspy/piper-voices`,
  path `pt/pt_BR/cadu/medium/MODEL_CARD`.
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

Curated as a calm, clear long-form reading alternative with
thematic "Sona-inspired feel" framing only. No League of Legends,
Riot, performer, or character imitation/branding is claimed or
implied by this voice id, label, or asset.

## Files

| File | Purpose |
|---|---|
| `pt_BR-cadu-medium.onnx` | VITS model (mono, 22,050 Hz, sherpa-onnx compatible). |
| `pt_BR-cadu-medium.onnx.json` | Model config — `audio.sample_rate = 22050`, `espeak.voice = "pt-br"`, `phoneme_type = "espeak"`, `num_symbols = 256`, `num_speakers = 1`, inference defaults (`noise_scale` 0.5, `noise_w` 0.333, `length_scale` 1.0). |
| `tokens.txt` | Token map for sherpa-onnx/Piper synthesis. |
| `MODEL_CARD` | Upstream model card (license + metadata). |

Phonemizer data: resolved via shared bundled
`assets/voices/shared/espeak-ng-data/` (preferred, espeak voice
`pt-br` compatible with faber/cadu/jeff).

## Integrity (SHA-256)

| File | SHA-256 |
|---|---|
| `pt_BR-cadu-medium.onnx` | `b6318f52e2c7ed8a9ac90caec529180aad581958b31e90f99cf3690a830a8ed7` |
| `pt_BR-cadu-medium.onnx.json` | `9fa837b7c1237ea5978bcd7be38c130798e8faa064f754873bb4ce5fc498bd88` |
| `tokens.txt` | `e8a50ae0c75612d18cfcf3f90800dca79f961e3ed97f9574f3535194469187fd` |
| `MODEL_CARD` | `ccd97b4369f50e6fba3a4fb730643ef2675f87b43c5d22298404c58f8204a375` |

- `pt_BR-cadu-medium.onnx`: 62950207 bytes.
- Declared sample rate 22,050 Hz matches handoff contract
  (`engine_core::format::DeclaredFormat::SAMPLE_RATE`); mismatch
  is a named `EngineError::RateMismatch`, never silent resample
  (FR-024b / SC-035).
