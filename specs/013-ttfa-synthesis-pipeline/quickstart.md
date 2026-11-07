# Quickstart: 013 TTFA Synthesis Pipeline

## Goal
Implement Option A (segment-level producer) to reduce TTFA and remove blocking synthesis from `playback_watch_tick`/startup critical path while preserving FR-001/005, 60 s pre-roll, fingerprint/resume/timeline invariants.

## Read first
- `specs/013-ttfa-synthesis-pipeline/spec.md`
- `specs/013-ttfa-synthesis-pipeline/plan.md`
- `.specify/assessments/ttfa-synthesis-pipeline/{decision,research,problem,concept,converge-clarify}.md`
- `src-tauri/src/lib.rs`: `play_document`, `prepare_session_document`, `stream_synthesis`, `produce_next_unit`, `advance_production`, `playback_watch_tick`, `open_backend/start_backend/enqueue_chunk`, `Production`

## Implementation order (TDD)
1. Add/adjust tests (RED): watch-no-synthesize, generation/stale-result discard, first-useful-audio with offset, FR-001/005 failure safety, resume/timeline/pre-roll invariants.
2. Introduce generation/cancellation control in `AppState`/production flow.
3. Move synthesis to blocking producer path (bounded `spawn_blocking` step or dedicated thread) with short-lock snapshot/commit.
4. Make `playback_watch_tick` observation-only for synthesis.
5. Ensure first useful piece for current generation enqueues/starts before unnecessary discarded work where safe; keep sample-accurate trim.
6. Run targeted tests, then gates; collect 2201117TY `play_to_audio_ms` smoke.

## Verification commands (targeted, adjust to existing test names)
```sh
cargo fmt --all -- --check
cargo nextest run -p mdreader --test <ttfa/production test names>
cargo clippy --workspace --all-targets -- -D warnings
```

## Device smoke (2201117TY)
- Install/debug build: `cargo tauri android build --target aarch64` / project-standard deploy flow.
- Exercise cold open, resume/high offset, switch during startup, 5+ min playback.
- Capture `diagnostic-record.jsonl` `play_to_audio_ms`, buffer/underrun state, and app responsiveness observations.

## Non-goals / guardrails
- No Option B sherpa intra-segment streaming.
- No chunking/model/fingerprint/prosody redesign.
- No chapter pre-roll T232/T233 completion claim.
- No silent false `Playing`; no stale generation enqueue/start.

## Evidence log (013 implementation, 2026-11-07)
- `3dd8f7b1` docs(spec): 013 assess/spec/plan/tasks/contracts artifacts.
- `401e7280` fix(ttfa): AppState/Production generation+wake fields, bounded producer helpers foundation.
- `f5c189c7` fix(ttfa): `playback_watch_tick` synthesis-free (signal/pump only), `play_document` reordered to FR-001/005-safe generation-first-useful startup, generation invalidation in teardown/abandon.
- `3f6fe0af` (+amend) fix(ttfa): `request_background_production` via `tauri::async_runtime::spawn_blocking`, serialized by `producer_driving`, generation-checked bounded batches, stale-safe failure handling.
- `14dcc4c1`/`d85b358b` test(ttfa): initial 013 lib smoke tests T003/T004/T011 passing (`cargo nextest run -p src-tauri --lib playback_watch_tests ttfa_013_tests` => 8 passed).
- Baseline refs: `play_document`/`prepare_session_document`/`stream_synthesis`/`produce_next_unit`/`advance_production`/`playback_watch_tick` in `src-tauri/src/lib.rs`; device 2201117TY `play_to_audio_ms` 1659–17309ms common, 45878–120422ms high-offset extremes.
- Remaining: full TDD coverage for resume/high-offset first-useful (T013-T017), stronger stale-commit/FR-001/005 tests (T018-T019), host gate sweep, 2201117TY smoke for TTFA claim.

## Final autonomous host snapshot (2026-11-07, continuation 6)

### Verified
- `cargo fmt --all` — clean
- `cargo clippy -p src-tauri --lib -- -D warnings` — clean
- `cargo nextest run -p src-tauri --lib ttfa_013_tests` — **11 passed**
- `cargo nextest run -p src-tauri --lib playback_watch_tests ttfa_013_tests --test test_resume_restarts_audio --test test_play_to_audio_measured` — **16 passed**
- 013 debug APK: `src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk` installed on 2201117TY (`adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp`, 2026-11-07 09:38:23, `app.mdreader` debug v0.1.0/1000)

### Blocked (not faked)
- **Device TTFA proof `blocked:device_ttfa_smoke`**: `DebugIngestActivity` document ingest succeeded (`ok:document:/data/local/tmp/013-ttfa-smoke.md`, snapshot logged) but no controlled playback started; `/data/data/app.mdreader/diagnostic-record.jsonl` showed no fresh `play_to_audio_ms`. Manual controlled StartNow, high-offset resume, switch/cancel during startup, and 5+ min playback/underrun capture required.
- **MDMV R4/R7**: not satisfied. **T232/T233**: unchanged.

### Capture command (after manual controlled playback)
```sh
adb -s adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp shell \
  run-as app.mdreader cat /data/data/app.mdreader/diagnostic-record.jsonl > /tmp/mdreader-013-diag.jsonl
rg 'play_to_audio_measured|play_to_audio_ms|position_secs' /tmp/mdreader-013-diag.jsonl | tail -n 40
```
