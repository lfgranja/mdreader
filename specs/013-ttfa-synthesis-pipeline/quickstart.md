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
