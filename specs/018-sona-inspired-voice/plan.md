# Implementation Plan: 018-sona-inspired-voice

- **Feature**: 018-sona-inspired-voice
- **Spec**: `specs/018-sona-inspired-voice/spec.md`
- **Assessment**: `.specify/assessments/sona-inspired-voice/decision.md` (GO)
- **Status**: Draft / Ready for Tasks
- **Created**: 2026-11-10
- **Canonical reference**: `local://sona-inspired-voice-plan.md` (approved; inline plan authoritative if conflict)

## 1. Summary

Implement a curated, bundled, offline, CC0-provenanced pt-BR voice catalog for Ressoa that evokes a non-imitative "Sona-inspired feel" aligned to `DESIGN-SONA/` (requester Option B). Keep `pt_BR-faber-medium` as stable default, add a validated curated alternative (target `pt_BR-cadu-medium`, fallback `pt_BR-jeff-medium`) and per-voice tuning/provenance, generalize `crates/adapter-synthesis` runtime/assets/build/CLI, persist/restore selection in `crates/session`, and wire catalog-aware diagnose/inventory/select/cycle behavior in `src-tauri` with safe restart-at-authoritative-position on voice change. Preserve 22050 Hz mono f32 handoff, fail-closed behavior, `OfflineDefault (None,None)`, Rust authority, and zero `INTERNET`.

## 2. Technical Context

- **Language/Version**: Rust 2024, stable, MSRV ≥ 1.85 (`--locked`)
- **Primary Dependencies**: `sherpa-onnx = "=1.13.8"` (pinned), `blake3`, `redb` (session store), `tauri = "2.12.1"` bridge, `serde/serde_json`, `clap` (CLI), existing normalization/prosody modules; no new network deps
- **Storage/Assets**: bundled voice assets under `crates/adapter-synthesis/assets/voices/` (per-voice dirs + shared `espeak-ng-data`), Android extracted assets, session settings in redb (`last_voice`)
- **Testing**: `cargo nextest --workspace`, unit/integration/contract tests in affected crates, `proptest` where valuable, `scripts/expected-tests.txt` diff, `scripts/verify.sh` gates 1–8 (host)
- **Target Platform**: Android (`minSdk 29`, AAudio/cpal backend context unchanged), host Linux for gates
- **Project Type**: Rust workspace (`crates/*`, `src-tauri`, `android-shell` mirror-only if touched)
- **Performance/Constraints**: additional bundled ONNX(s) increase APK/extraction/RAM/load; voice switching must be deterministic and fail-closed; no mixed-voice PCM in active stream; no silent resample/downmix
- **Scale/Scope**: small closed catalog (2 bundled voices initially, evolution-ready API)

## 3. Constitution / Governance Check

| Gate | Status | Notes |
|------|--------|-------|
| R1 (assess→decision before specify) | PASS | `.specify/assessments/sona-inspired-voice/decision.md` verdict `go` (2026-11-10) |
| R3 (clarifications ≤3) | PASS | 0 blocking clarifications; residuals recorded as implementation validations with contingencies |
| Library-first / one code path | PASS | Same Rust synthesis path for CLI and bridge; `main.rs`/commands remain thin |
| Rust authority / zero Android API in Rust crates | PASS | No playback/synthesis authority moved to Kotlin; Kotlin mirror-only if inventory fields change |
| Closed sets / exhaustive matches | PASS | `VoiceId`/catalog/order/errors modeled as closed types; no `_ =>`, no `Other/Unknown` |
| Audio contract | PASS | Mono `f32` @ 22,050 Hz preserved; no silent resample/downmix; fail-closed named errors |
| Offline / zero `INTERNET` | PASS | Bundled assets only; no manifest/network capability changes |
| Declared capabilities/tier | PASS | `OfflineDefault` only; `(SsmlLevel::None, AlignmentGranularity::None)` unchanged |
| Supply chain / quality gates | PLANNED | `--locked`, `cargo-deny`/`cargo-audit`, fmt/clippy/nextest/doc/closed-set checks; gates 1–8 required |

Post-design re-check required after implementation (especially closed-set scan, expected-tests diff, no-scope/asset assertions, and Android extraction behavior).

## 4. Architecture & Design

### 4.1 Voice Catalog & Assets

- **Layout**: `crates/adapter-synthesis/assets/voices/<voice-id>/` per bundled voice with `MODEL_CARD`, `PROVENANCE.md`, `<voice-id>.onnx`, `<voice-id>.onnx.json`, `tokens.txt` (and required model files). Share phonemizer data in `assets/voices/_shared/espeak-ng-data/` if validated compatible; otherwise per-voice `espeak-ng-data/` with explicit provenance.
- **Catalog (closed, fixed order):** index 0 `pt_BR-faber-medium` (default), index 1 curated alternative (`pt_BR-cadu-medium` target, `pt_BR-jeff-medium` fallback). Deterministic ids = directory names; no runtime discovery from arbitrary filesystem beyond bundled/packaged roots.
- **Provenance:** record upstream URL, HF revision/commit or snapshot date, license (CC0), file list, checksums (blake3/sha256 as appropriate), inference defaults (`noise_scale`, `noise_scale_w`, `length_scale`, sample rate 22050), and "Sona-inspired feel" curation note (non-imitative).
- **`build.rs`:** emit manifest covering all bundled voice dirs and shared assets deterministically (sorted), failing build/validation only on packaging rules consistent with existing policy.

### 4.2 `crates/adapter-synthesis` Runtime Refactor

- **New module `voice` (or equivalent):** closed `VoiceId` enum + `VoiceSpec` metadata + `CATALOG`/ordering helpers (`default()`, `next/prev`, `parse`, `iter`), all exhaustive. No `Other/Unknown`; unknown external strings map to `Result`/`Option`/fallback at API boundary.
- **`synth.rs`:**
  - Replace single fixed paths with per-voice resolution rooted at selected voices root (`set_assets_root`/`set_voices_dir` generalized from `set_asset_dir`), supporting packaged and dev layouts.
  - Engine lifecycle on pinned `sherpa-onnx = "=1.13.8"`: prefer `OnceLock`-backed per-voice cache map/array keyed by `VoiceId` **if** instance lifecycle is safe/compatible; **contingency** recreate a single active engine on selection/switch (documented, deterministic). Never hold incompatible global mutable state without synchronization.
  - `synthesize_with_voice(voice: VoiceId, segment: &DocumentSegment) -> Result<Vec<f32>, EngineFailure>` (or equivalent) as primary; `synthesize(segment)` delegates to active/default voice for compatibility.
  - Preserve normalization (`normalize_for_speech`), prosody planning, 22050 Hz/sample-shape checks, echoed `resume_point|source_range`, and fail-closed behavior.
- **`android_assets.rs`:** extract/copy the full bundled catalog (and shared espeak dir) with idempotent versioned marker; preserve fail-closed extraction semantics.
- **`cli.rs`:** additive commands/flags: `voices` (list catalog/default/active fields) and `synthesize --voice <id>`; keep existing payload fields stable and add voice metadata additively/compatibly. Stdin/args → stdout JSON, errors → stderr, exit 0/1.
- **`lib.rs`/`reg.rs`/`capabilities.rs`:** keep `REGISTERED_TIER = OfflineDefault`, `declared_capabilities() -> (None,None)`; expose catalog/selection APIs.

### 4.3 Session Persistence (`crates/session`)

- Add `LAST_VOICE_KEY = "last_voice"` and `put_last_voice`/`get_last_voice` on `BookmarkStore` mirroring `last_rate`/`last_normalization_mode` patterns.
- `get_last_voice` returns default `pt_BR-faber-medium` for missing/unknown/invalid stored values (never panic). Parsing maps only known catalog ids.

### 4.4 Tauri Bridge / Shell Surface (`src-tauri`)

- Startup (desktop/Android): resolve packaged/extracted voices root, initialize adapter catalog root, seed normalization and voice from session store (fallback default), diagnose/inventory catalog-aware.
- `commands::diagnose`/voice inventory: report active/default voice and per-voice presence/asset status for catalog (additive fields), replacing hardcoded single-voice assumptions while keeping existing consumers compatible where possible.
- Selection/cycle commands: implement `get_voice`/`set_voice` (and wire `CycleVoiceNext/Prev` live control) to update active selection, persist to session, publish mirror/inventory state, and when playback/production is active perform safe restart (invalidate production generation, restart synthesis from authoritative resume position) — no mixed-voice PCM.
- `synthesize_segment`/production path: resolve active voice per request and pass to `adapter_synthesis`.
- Register any new Tauri commands in `invoke_handler`; keep Kotlin mirror transport-only.

### 4.5 Contracts & Interfaces

- `contracts/` (feature-local) documents catalog schema, CLI additions, diagnose/inventory fields, and cycle/set/get semantics/deltas.
- `contracts/shell-ipc.md` updated for `CycleVoiceNext/Prev` behavior and any voice fields if they are part of the normative shell contract.
- Player CLI/shell types remain source-compatible; additive wire fields only unless explicitly versioned.

## 5. Milestones / Slices

- **Slice 1 — Assessment/Spec artifacts (done/planned):** `.specify/assessments/sona-inspired-voice/*` (GO), `specs/018-sona-inspired-voice/{spec,plan,tasks,checklists,contracts}`.
- **Slice 2 — Assets & provenance:** acquire/validate bundled curated voice (target cadu), add `MODEL_CARD`/`PROVENANCE.md`/checksums, restructure to per-voice + shared espeak layout, update `build.rs` manifest.
- **Slice 3 — Adapter core:** `voice` catalog/types, refactor `synth.rs`/paths/engine lifecycle, update `android_assets.rs`, extend `cli.rs`/`lib.rs` with compatible APIs.
- **Slice 4 — Persistence & bridge:** `session` `last_voice`, `src-tauri` seed/diagnose/inventory/get/set/cycle + safe restart wiring, publish/contract updates.
- **Slice 5 — Tests & script alignment:** unit/integration/contract tests across crates, update `src-tauri/tests/test_noscope.rs` and `scripts/expected-tests.txt` if needed, sweep hardcoded voice assumptions.
- **Slice 6 — Verification & evidence:** narrow checks, host gates 1–8, record auditory validation/default rationale and provenance; report gate 9/device status.

## 6. Verification Plan

- Narrow, change-exercising checks first: `cargo test -p adapter-synthesis` / targeted nextest filters, `cargo test -p session`, targeted `src-tauri` and `player` tests as affected.
- Host gates 1–8 (authoritative): `cargo fmt --all -- --check`; `cargo nextest run --workspace` and compare `cargo nextest list --workspace` (sorted) against `scripts/expected-tests.txt`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo doc --workspace --no-deps`; `cargo deny check advisories licenses bans sources`; `cargo audit`; `./scripts/verify.sh`.
- Contract/script hygiene: if nextest target set changes, update `scripts/expected-tests.txt` in the same change; update `test_noscope` closed-catalog assertion; ensure no forbidden lints/patterns.
- Evidence (manual/recorded): auditory side-by-side on representative pt-BR passages (faber vs curated), default-choice rationale, runtime switch/load behavior notes, and (if run) Android extraction/device observations. Gate 9 not claimed unless executed.

## 7. Risks & Contingencies

| Risk | Mitigation / Contingency |
|------|---------------------------|
| Curated alternative not subjectively "Sona-inspired" enough | Record evidence; keep `faber` default, ship validated alternative as selectable or fall back to tuned-faber-only catalog with multi-voice-ready API |
| `sherpa-onnx 1.13.8` cannot cache multiple instances safely | Recreate single active engine on selection/switch (restart-at-boundary) |
| Shared `espeak-ng-data` incompatible | Per-voice espeak dirs with provenance; accept size increase |
| APK/extraction size & load time regression | Measure/record; keep catalog minimal (2 voices), deterministic packaging |
| Broad hardcoded-voice blast radius | Systematic sweep + tests (`test_noscope`, CLI/bridge/session/contract expectations) and gate diff hygiene |
| IP/branding drift toward LoL imitation | Explicit non-imitative naming/provenance checks and out-of-scope enforcement |

## 8. Out of Scope (implementation)

- User import/download/cloud/marketplace voices, `INTERNET`, paid packs.
- LoL Sona/performer imitation, Riot branding/likeness.
- SSML/alignment or engine-tier expansion.
- Unrelated UI redesign, gate-9 device execution unless explicitly run and evidenced.