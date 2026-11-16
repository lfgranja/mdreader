# Research — 018 Sona-Inspired Voice (Option B)

Date: 2026-11-12  
Spec: `specs/018-sona-inspired-voice/spec.md` (GO, 2026-11-10)  
Scope: comparative evaluation of 8 external repos + baseline asset validation for curated alternative voice.

## 1. Comparative evaluation (mdmv-dark-factory protocol, spec-018 fit)

Sources: primary repo metadata/README/LICENSE/docs/HF model cards and sherpa-onnx docs read 2026-11-12. Scoring lens: runtime/stack fit (Rust, `sherpa-onnx = "=1.13.8"`, Piper VITS), pt-BR 22050 Hz mono, offline bundled CC0 catalog, Android feasibility, license/provenance, and spec constraints (no INTERNET, no imitation, closed catalog, fail-closed).

| Repo | License | Stack/runtime fit | pt-BR 22050 Hz CC0 bundled asset | Offline/Android | Directly usable for spec 018? | Usable takeaway |
|---|---|---|---|---|---|---|
| https://github.com/jamiepine/voicebox | MIT | Python/FastAPI + Tauri desktop; ML backends (Qwen/Chatterbox/Kokoro/OmniVoice); HF downloads; no sherpa-onnx/Piper ONNX runtime path found | No (model terms vary; OmniVoice CC-BY-NC) | No Android target shipped; desktop-first, GB-scale models | **No** | UX reference only: curated preset voices, VoiceDesign text→voice cues, pluggable engine registry, local-first posture |
| https://github.com/debpalash/VoiceStudio | AGPL-3.0 app | Electron+Python desktop; native Android/Kotlin mobile pins **sherpa-onnx v1.13.8**; sherpa-onnx loads user-supplied Piper `.onnx`+`tokens.txt`+optional `espeak-ng-data`; output 22.05 kHz default | No bundled CC0 pt-BR Piper catalog (OmniVoice catalog incl. pt-BR, not Piper) | Mobile ships after-download packs w/ checksum/licence gating; not pre-bundled CC0 reader | **No** (AGPL copyleft + mixed model terms + download-centric) | Reference architecture: sherpa-onnx/Piper artifact layout (`tokens.txt`, voice `.onnx`, optional `espeak-ng-data`), mobile on-device lifecycle, `sherpa-onnx v1.13.8` pin matching spec |
| https://github.com/myshell-ai/OpenVoice | MIT | Python/PyTorch base-speaker TTS + tone-color converter (zero-shot cloning); V2 = EN/ES/FR/ZH/JA/KO only; community ONNX exports only | No (not Piper; provenance per-model) | No official ONNX/mobile; heavy PyTorch deps | **No** | None for runtime; cloning paradigm conflicts with non-imitative fixed catalog |
| https://github.com/CorentinJ/Real-Time-Voice-Cloning | unclear (no LICENSE text retrievable in profile) | Python 3.9–3.10 + PyTorch 1.10 SV2TTS (encoder/Tacotron/WaveRNN); English-centric; author-stated stale | No | No; desktop Python | **No** | None |
| https://github.com/k2-fsa/OmniVoice | Apache-2.0 code | Python/PyTorch 2.8 diffusion-LM TTS; 600+ langs incl. pt-BR likely; 24 kHz output; RTF 0.025 claim; voice cloning/design | No (not Piper; 24 kHz ≠ 22050 Hz) | ONNX export not official (issue #151); community `omnivoice-rs` Rust crate exists (future engine option only) | **No** | None for v1; potential future engine diversification (not Piper/sherpa-onnx stock path) |
| https://github.com/RVC-Boss/GPT-SoVITS | MIT code | Python/PyTorch few-shot cloning (GPT+SoVITS); langs zh/ja/en/ko/yue only; ONNX export multi-graph/custom; sherpa-onnx issue #3694 open, unsupported | No | No; heavy RAM Android port exists (GPL-3.0, independent) | **No** | None for runtime; asset-generation inspiration only, legally constrained |
| https://github.com/PlayVoice/whisper-vits-svc | MIT (repo metadata) | **Singing Voice Conversion (SVC), not TTS**; Python/PyTorch; min 6GB VRAM training; multiple pretrained components w/ unclear licenses | No | No | **No** | None |
| https://github.com/abus-aikorea/voice-pro | GPL-3.0 (LICENSE authoritative; README header says LGPL — conflict) | Python/Gradio/Torch desktop dubbing app; Edge-TTS (network), Kokoro 24 kHz, F5/CosyVoice cloning; HF downloads on first use; Windows-primary | No | No Android/Rust/sherpa-onnx/Piper path | **No** (GPL copyleft + online features + format mismatch) | None |

**Decision**: none of the 8 repos is adoptable as-is for spec 018. Implementation proceeds per the already-authored spec/plan/tasks using the spec-mandated stack: upstream `rhasspy/piper-voices` CC0 Piper VITS pt-BR assets (`faber` default + `cadu` curated alternative, both 22050 Hz, CC0 via OHF-Voice voice-datasets) consumed by pinned `sherpa-onnx = "=1.13.8"` in `crates/adapter-synthesis`. Reference cues taken only from VoiceStudio (sherpa-onnx/Piper packaging layout, mobile lifecycle pinning, `sherpa-onnx v1.13.8`) and Voicebox (curation/selection UX language, non-imitative framing). No code or assets from the 8 repos are copied.

## 2. Baseline asset facts (verified 2026-11-12)

### Current bundled voice (repo)
- `crates/adapter-synthesis/assets/voices/pt_BR-faber-medium/`: `MODEL_CARD`, `PROVENANCE.md`, `pt_BR-faber-medium.onnx` (63,201,428 B), `pt_BR-faber-medium.onnx.json`, `tokens.txt` (152 lines), `espeak-ng-data/` (355 files).
- Config: 22050 Hz, mono, 1 speaker, espeak `pt-br`, inference defaults `noise_scale 0.667`, `noise_w 0.8`, `length_scale 1.0`. License: CC0 (NabuCasa/voice-datasets). Provenance: `https://huggingface.co/csukuangfj/vits-piper-pt_BR-faber-medium` (sherpa-onnx/Piper canonical form).

### Candidate alternatives (upstream rhasspy/piper-voices)
- **Primary candidate `pt_BR-cadu-medium`**: `rhasspy/piper-voices`, path `pt/pt_BR/cadu/medium/`. Expected files: `MODEL_CARD`, `pt_BR-cadu-medium.onnx.json` (~5.4 kB), `pt_BR-cadu-medium.onnx` LFS (~63,221,984 B), `samples/`. Config per upstream: 22050 Hz, 1 speaker, espeak `pt-br`, defaults `noise_scale 0.5`, `noise_w 0.333`, `length_scale 1.0`, "Requires Piper 1.5 or higher". License: **CC0** (OHF-Voice voice-datasets).
- **Fallback `pt_BR-jeff-medium`**: same path family, same technical shape/defaults, CC0 (OHF-Voice). Use only if cadu fails auditory/runtime/packaging validation.
- **Excluded `pt_BR-edresson-low`**: 16 kHz, CC BY 4.0 — excluded by spec 018.

### Compatibility
- `sherpa-onnx = "=1.13.8"` (workspace pin) ships ONNX Runtime 1.28.2 and hosts converted Piper VITS samples incl. `vits-piper-pt_BR-cadu-medium` and `vits-piper-pt_BR-faber-medium` → Piper-format `.onnx` + `.onnx.json` + `tokens.txt` consumable by pinned runtime.
- Phonemizer: all pt-BR Piper voices use espeak voice `pt-br`; shared `espeak-ng-data` is preferred (contingency: per-voice copy).

## 3. Selection and validation baseline (T001)

- **Selected alternative (primary)**: `pt_BR-cadu-medium` (CC0, 22050 Hz, Piper VITS, pt-BR). Rationale: spec-018 contract names cadu as first candidate; CC0/OHF-Voice provenance; sherpa-onnx 1.13.8 compatibility; calm/clear medium-quality profile suitable for curated non-imitative "Sona-inspired feel" (thematic framing only).
- **Default remains**: `pt_BR-faber-medium` (index 0) unless auditory evidence in Step 5 clearly favors alternative; preserves conservative continuity.
- **Fallback**: `pt_BR-jeff-medium` if cadu fails technical/auditory/packaging validation; if neither passes, keep multi-voice-ready catalog/API with `faber` only and document rationale.
- **Auditory fit validation**: manual side-by-side on representative pt-BR passages (recorded qualitatively in Step 5). Non-imitative audit required: no LoL/Riot/performer branding in ids/labels/assets/docs.

## 4. Provenance/checksum evidence

- [x] `pt_BR-faber-medium`: PROVENANCE.md retained/updated for shared `espeak-ng-data` layout (spec 018); asset files unchanged except espeak moved to `_shared/espeak-ng-data`.
- [x] `pt_BR-cadu-medium`: `PROVENANCE.md` + `MODEL_CARD` bundled with checksums recorded. Source `rhasspy/piper-voices` (OHF-Voice, CC0): `pt/pt_BR/cadu/medium/pt_BR-cadu-medium.onnx?download=true`, `.onnx.json`, `MODEL_CARD`; `tokens.txt` from `csukuangfj/vits-piper-pt_BR-cadu-medium` (2026-11-12).
  - onnx (63,221,984 B) sha256 `d5a662e3f606d5a49dd29ba9ce90c304ae45dd9543f8b0c773bcb0de282499de`
  - onnx.json (5,465 B) sha256 `9fa837b7c1237ea5978bcd7be38c130798e8faa064f754873bb4ce5fc498bd88`
  - tokens.txt (161 lines) sha256 `e8a50ae0c75612d18cfcf3f90800dca79f961e3ed97f9574f3535194469187fd`
  - MODEL_CARD (420 B) sha256 `ccd97b4369f50e6fba3a4fb730643ef2675f87b43c5d22298404c58f8204a375`
- [x] Shared `espeak-ng-data` (355 files) copied from validated `pt_BR-faber-medium` bundle; preferred for all catalog voices with per-voice fallback.

**Runtime/host evidence (implementation pass):**
- `cargo fmt -p adapter-synthesis -p session -p src-tauri` / checks: pass.
- `cargo check -p adapter-synthesis`, `cargo check -p session`, `cargo check -p player -p src-tauri`: pass.
- `cargo test -p adapter-synthesis --lib`: 20 passed (voice unit tests included).
- `cargo test -p adapter-synthesis --test voice_catalog`: 5 passed.
- `cargo test -p session --lib last_voice`: 1 passed.
- `cargo test -p src-tauri --test test_noscope`: 8 passed (catalog-aware bundled voice assertion).
- Remaining: `cargo test -p adapter-synthesis --test synth_voice` (real engine, pending/long), full workspace nextest/clippy/doc/deny/audit/gates and final greps — report in Step 6.

## 5. Auditory/default rationale

- `pt_BR-faber-medium` remains default (index 0) for continuity and known baseline behavior.
- `pt_BR-cadu-medium` selected as curated alternative: CC0 OHF-Voice provenance, 22050 Hz Piper VITS/espeak `pt-br`, distinct dataset (`cadu`), compatible with sherpa-onnx 1.13.8, non-imitative descriptive curation.
- **Manual listening (maintainer/device, pending):** side-by-side perceptual assessment on representative pt-BR passages not completed in this host-only implementation pass; default retained pending auditory evidence. `pt_BR-jeff-medium` remains documented fallback if cadu validation fails.

## 6. Non-imitative identity audit

- Voice ids = upstream directory names (`pt_BR-faber-medium`, `pt_BR-cadu-medium`), no "Sona"/LoL/Riot/performer tokens.
- Labels: non-imitative descriptive text only ("Faber (pt-BR, medium)", "Cadu (pt-BR, medium)").
- Catalog/provenance docs carry CC0/OHF-Voice metadata only; no persona/real-person/brand emulation claims added.
- Final shipped-id/label/asset/doc branding and INTERNET-permission greps to be included in gate report.

