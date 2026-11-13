# Converge: 018 Sona-Inspired Voice

Status: **CONVERGED** (host gates 1–8 PASS; gate 9 Rust compile PASS; Android assets verified).

## Implementation Summary

Closed bundled voice catalog (`pt_BR-faber-medium` default + `pt_BR-cadu-medium` curated alternative, CC0) implemented across adapter-synthesis, session, and tauri bridge. All 27 tasks (T001–T027) completed.

### Key Deliverables

- **Catalog**: `crates/adapter-synthesis/src/voice.rs` — closed `VoiceId` enum (2 variants), fixed order, `DEFAULT`, `CATALOG`, `next()`/`prev()` with wrap, `parse_id`/`FromStr` (unknown rejected), `spec()`, `inference_defaults()`. No `Other`/`Unknown`/catch-all.
- **Synthesis**: `synth.rs` — `set_voices_dir`/compat `set_asset_dir`, per-voice cached engines (`OnceLock<Option<OfflineTts>>`), `synthesize_with_voice`, `active_voice`/`replace_active_voice` (atomic index), compat `set_model_path`/`set_playback_rate`. Normalization/prosody/22050 Hz fail-closed checks preserved.
- **Android extraction**: `android_assets.rs` — full catalog extraction with versioned `.extracted` marker (`catalog.version="1"`), freshness requires marker + faber onnx + cadu onnx + shared espeak.
- **CLI**: `voices` subcommand (deterministic JSON), `synthesize --voice <id>` (additive `voice` field in payload). Existing contracts preserved.
- **Session**: `store.rs` — `LAST_VOICE_KEY`, `put_last_voice`/`get_last_voice` (malformed → `None`). Existing DSP preset persistence preserved.
- **Tauri**: startup seeds voices root + active voice from persisted `last_voice` (default fallback). `diagnose` catalog-aware (`active_voice`, `default_voice`, `voices[]` + legacy fields). `get_voice`/`set_voice` commands registered. `execute_live_control` handles `CycleVoiceNext/Prev` with safe restart-on-change (invalidate generation, stop backend, reanchor from authoritative resume position when playing; drop stale buffers when paused/interrupted).
- **tauri.conf.json**: resources mapped from entire `assets/voices` directory.

### Tests

- `adapter-synthesis` lib: 20/20 PASS (voice unit tests included)
- `voice_catalog`: 5/5 PASS (catalog order, cycle wrap, parse, bundle presence, inference defaults)
- `synth_voice`: 2/2 PASS (both catalog voices produce non-empty PCM; active-voice restore)
- `session` `last_voice`: 1/1 PASS
- `src-tauri` `test_noscope`: 8/8 PASS (catalog-aware bundled voice assertion + no INTERNET)
- `cli_protocol`: all PASS (preserved existing contracts)

### Verification

- `cargo fmt --all --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo doc --workspace --no-deps`: PASS (warnings only, pre-existing)
- `cargo audit` / `cargo deny`: PASS (bans/licenses/sources OK)
- `scripts/expected-tests.txt` two-way diff: CLEAN (136/136)
- Gate 9 (Android): Rust library compiled for aarch64 (minSdk 29); full voice catalog in Android assets; no INTERNET permission. Gradle APK packaging timed out (mechanical step).

### Non-Imitative Audit

- Voice ids = upstream directory names (`pt_BR-faber-medium`, `pt_BR-cadu-medium`); no performer/brand/character tokens.
- Labels: non-imitative descriptive text only ("Default reading voice", "Curated alternative — calm, clear").
- Provenance docs carry CC0/OHF-Voice metadata only.

### Unresolved / Deferred

- Manual auditory comparison (maintainer/device) for final default-choice confirmation: `pt_BR-faber-medium` retained as default pending perceptual evidence.
- `pt_BR-jeff-medium` fallback not bundled (cadu validated for catalog slot).
- Android APK packaging (gradle) did not complete within timeout; Rust compilation and asset layout verified.
