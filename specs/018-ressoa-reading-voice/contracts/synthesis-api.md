# Contract: Adapter Synthesis API & CLI (018-sona-inspired-voice)

- **Feature**: `specs/018-sona-inspired-voice`
- **Status**: Draft
- **Primary crates**: `crates/adapter-synthesis`, consumers in `src-tauri`, `crates/player` (as applicable)

## 1. Rust Library Surface (`adapter-synthesis`)

### Catalog / selection (additive, closed)
- Expose closed `VoiceId`, default voice, fixed-order catalog iteration, and helpers: parse known id, next/prev cycle, metadata/spec lookup (paths/default inference config/provenance refs as appropriate).
- Active voice accessors: get/set active or default-selected voice for synthesis entrypoints (thread-safe, deterministic). Unknown external ids are not represented as `VoiceId`.

### Voice-aware synthesis
- **Primary:** `synthesize_with_voice(voice: VoiceId, segment: &DocumentSegment) -> Result<Vec<f32>, EngineFailure>` (or equivalent signature consistent with crate style).
- **Compatibility:** existing `synthesize(segment)` delegates to active/default voice.
- Behavior preserved: normalize (`normalize_for_speech`) → prosody plan/rate handling → engine generate → 22,050 Hz/shape validation → echoed `resume_point`/`source_range` metadata semantics in CLI payloads; fail-closed named errors on init/load/missing assets/format mismatch.

### Engine lifecycle
- Pinned runtime: `sherpa-onnx = "=1.13.8"`.
- Implementation MAY cache per-voice engines if safe; MUST fall back to recreate-on-switch (single active engine) if caching is infeasible/unsafe. Switching/creation failures are named errors; no panic on expected runtime/input conditions.

### Assets root
- Generalize `set_asset_dir`/root initialization to a voices/catalog root supporting bundled dev layout and Android extracted layout (with shared `espeak-ng-data` or validated per-voice layout). `build.rs` manifest and `android_assets.rs` extraction MUST align.

## 2. CLI Surface (`adapter-synthesis` CLI)

- **List voices (additive):** `voices` (or equivalent) prints deterministic JSON on stdout describing catalog entries (id, default/active markers, availability/provenance metadata as implemented), exit 0 on success.
- **Synthesize by voice (additive):** `synthesize --voice <voice-id>` selects the specified bundled voice; omitted `--voice` uses default/active compatible behavior.
- **Output:** stdout JSON payload remains compatible with existing consumers; voice id/metadata MAY be added as additive fields. Errors to stderr, exit 1 on failure. No interactive prompts.

## 3. Capabilities / Registry (unchanged)

- `OfflineDefault` remains the sole registered/selectable synthesis tier for this feature.
- `declared_capabilities()` remains `(SsmlLevel::None, AlignmentGranularity::None)`.
- No SSML, no alignment, no cloud/provider fallback, no network.

## 4. Failure Modes (named, fail-closed)

- Unknown requested voice id at boundary: explicit CLI/bridge/session validation error or safe fallback per surface contract (persistence restores default; explicit invalid request is rejected, not silently remapped except where contract states fallback).
- Missing/corrupt/incompatible voice assets or engine init failure: `EngineError::InitFailed`/equivalent named failure surfaced to caller; no silent substitution, no resample/downmix.
- NUL/invalid synthesis input handling remains unchanged.