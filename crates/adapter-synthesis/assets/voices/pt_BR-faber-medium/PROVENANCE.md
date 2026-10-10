# Provenance — Piper pt-BR `faber` voice (medium)

Embedded voice assets for the `offline_default` synthesis
engine (`crates/adapter-synthesis`), bound by T044
(FR-024). Loaded at synthesis time through paths resolved
at compile time from `CARGO_MANIFEST_DIR`
(`src/synth.rs::{MODEL_PATH, TOKENS_PATH, ESPEAK_DATA_DIR}`)
— **never** fetched from the network at synthesis time
(constitution Principle IX, offline-first; `spec.md`
Assumption 2).

## Source

- **Repository**: `https://huggingface.co/csukuangfj/vits-piper-pt_BR-faber-medium`
  (the canonical sherpa-onnx / Piper voice repository,
  `csukuangfj`).
- **Model family**: VITS, Piper pt-BR, medium quality,
  1 speaker, finetuned from the U.S. English `lessac`
  voice (medium).
- **Downloaded**: 2026-10-09 via `git clone --depth 1`
  (git-lfs) from the repository above.

## License

**CC0** — as stated by the repository's `MODEL_CARD`:

> ## Dataset
> * URL: <https://github.com/NabuCasa/voice-datasets>
> * License: CC0

This matches `spec.md` Assumption 2 (`faber` is CC0;
`edresson` is excluded for CC-BY-4.0). The `MODEL_CARD`
is embedded verbatim alongside the assets.

## Files

| File | Bytes | Purpose |
|---|---|---|
| `pt_BR-faber-medium.onnx` | 63,201,428 | The VITS model (mono, 22,050 Hz). |
| `pt_BR-faber-medium.onnx.json` | 4,855 | Model config — declares `audio.sample_rate = 22050`, `espeak.voice = "pt-br"`, `phoneme_type = "espeak"`, `num_symbols = 256`, `num_speakers = 1`, and the inference parameters (`noise_scale` 0.667, `noise_w` 0.8, `length_scale` 1). |
| `tokens.txt` | 907 | The token map the model was trained with. |
| `espeak-ng-data/` | 19 MiB (355 files, 37 directories) | The espeak-ng data tree the Piper phonemizer resolves pt-BR phonemes against at synthesis time (required at runtime by `sherpa-onnx`'s `piper_phonemize`/`espeak-ng` static libs). |
| `MODEL_CARD` | 278 | The repository's model card (license + metadata), embedded for provenance. |

## Integrity

- `pt_BR-faber-medium.onnx` — exactly **63,201,428** bytes
  (verified after download against the repository).
- **blake3** of `pt_BR-faber-medium.onnx`:
  `feb0b4b613456f0b531c86744af18b2d9a8dbdb7accd34eab2c068a8ece26cbd`
  (blake3-256; the workspace standardizes on blake3).
- The model config's declared sample rate is **22,050 Hz**
  — matching the declared handoff shape
  (`engine_core::format::DeclaredFormat::SAMPLE_RATE`), so
  the delivered PCM conforms to the declared mono-`f32`-at-
  22,050 Hz format with no resampling. A mismatch would be
  a named `EngineError::RateMismatch`, never a silent
  resample (FR-024b / SC-035).

## Engine binding

The native library is **not** built from source: `sherpa-onnx-sys`
1.13.8's build script downloads the prebuilt static archive
`https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.13.8/sherpa-onnx-v1.13.8-linux-x64-static-lib.tar.bz2`,
which bundles ONNX Runtime, espeak-ng and piper_phonemize
as static libraries (see `PROVENANCE` note in
`API_CONTRACT.md` §7 for the `ort` deviation record).
