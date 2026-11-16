# Contract: Bundled Voice Catalog (018-sona-inspired-voice)

- **Feature**: `specs/018-sona-inspired-voice`
- **Status**: Draft
- **Scope**: `crates/adapter-synthesis` (source of truth), propagated to CLI/session/bridge/diagnostics

## 1. Closed Catalog

The bundled reading-voice catalog is a **closed set** declared in code with exhaustive handling (no `_ =>`, no `Other`, no `Unknown` variant).

| Index | VoiceId (stable, = dir name) | Role | Provenance/format requirements |
|------:|-------------------------------|------|--------------------------------|
| 0 | `pt_BR-faber-medium` | **Default** (compatibility baseline) | Existing bundled voice; Piper VITS, 22050 Hz mono-compatible, CC0/OHF-Voice, auditable `MODEL_CARD`/`PROVENANCE.md` preserved |
| 1 | `pt_BR-cadu-medium` *(target)* or `pt_BR-jeff-medium` *(validated fallback)* | Curated Sona-inspired alternative (selectable) | Piper VITS, **22050 Hz**, **CC0** (OHF-Voice voice-datasets per upstream card), auditable `MODEL_CARD`/`PROVENANCE.md`, checksums/hashes, recorded inference defaults |

**Exclusions:** `pt_BR-edresson-low` (16 kHz, CC BY 4.0) and any voice not meeting 22050 Hz + CC0-compatible + auditable provenance requirements MUST NOT be bundled.

**Contingency:** if no alternative passes auditory/runtime/packaging validation, ship a catalog containing only `pt_BR-faber-medium` (or tuned faber) but retain the closed catalog/selection API and document evidence/rationale.

## 2. Identity & Ordering

- **Ids:** deterministic, stable, equal to bundled directory names under `assets/voices/<voice-id>/`.
- **Order:** fixed declaration order is authoritative for `CycleVoiceNext/Prev` and listing; default is index 0.
- **Display:** human-readable labels/descriptions may be added as additive metadata but MUST remain non-imitative (no "Sona"/LoL/Riot character or performer claims). Thematic wording (e.g. calm/musical curation) must align with `DESIGN-SONA/` without trademark misuse.
- **Parsing:** external string → known `VoiceId` is total over declared ids only; unknown strings are rejected/fall back at boundaries (never coerced to `Other/Unknown`).

## 3. Asset Layout & Provenance

Per voice directory (minimum, subject to engine needs):
- `<voice-id>.onnx`
- `<voice-id>.onnx.json` (or required config)
- `tokens.txt` (if required by model/runtime)
- `MODEL_CARD`
- `PROVENANCE.md`

Shared phonemizer data (preferred): `assets/voices/_shared/espeak-ng-data/` validated compatible for all bundled voices; **contingency:** per-voice `espeak-ng-data/` with provenance.

`PROVENANCE.md` MUST record: upstream source (e.g. `rhasspy/piper-voices` path/URL), revision/snapshot date, license, included files, checksums (blake3/sha256), sample rate (22050), inference defaults (`noise_scale`, `noise_scale_w`, `length_scale`, speaker count as applicable), and a non-imitative curation note for the "Sona-inspired feel" where relevant.

## 4. Runtime Invariants

- All bundled voices MUST synthesize mono `f32` @ 22,050 Hz through the same normalization/prosody/handoff pipeline; no silent resample/downmix.
- Declared capabilities remain `(SsmlLevel::None, AlignmentGranularity::None)`; tier remains `OfflineDefault`.
- Missing/corrupt/incompatible assets or init failure MUST fail closed with a named error; never silently substitute another voice.
- Voice change applies at a safe synthesis boundary: invalidate production/generation and restart from authoritative resume position in the new voice; never mix voices in an active PCM stream.
- Offline-only: no download, no `INTERNET`.

## 5. Persistence Contract

- Setting key: `last_voice` (string value = `VoiceId`).
- Restore: known id → select; missing/unknown/malformed → `pt_BR-faber-medium` (default), no panic.

## 6. Compatibility

- Existing callers that do not specify a voice use the active/default voice (compatibility-preserving delegation).
- Additive CLI/bridge fields are preferred; any breaking wire change must be coordinated with `contracts/shell-ipc.md`, consumers, and tests in the same change.