# Contract: Synthesis Producer (Option A)

**Feature**: `013-ttfa-synthesis-pipeline`  
**Applies to**: `src-tauri/src/lib.rs` production path, `AppState` production control, tests; must preserve `player`/`adapter-synthesis`/`session` semantics.

## Producer contract

1. **Offload**: CPU-bound synthesis (`adapter_synthesis::synth::synthesize`) MUST NOT execute on:
   - cpal audio callback
   - async Tokio task body of `playback_watch_tick` / 500 ms watch loop
   - UI-sensitive blocking command/JNI path in a way that blocks responsive startup/state publication
2. **Bounded scheduling**: producer respects `queue_room` and `MAX_PREROLL_SECS`/`ResidentQueue` backpressure. Demand-driven; no unbounded PCM accumulation.
3. **First useful audio**: for `start_offset_secs >= 0`, producer commits/enqueues the first useful audible piece for the current generation as early as safely possible. `samples_before_entry` remains correct.
4. **Generation safety**: every commit/enqueue/start/publish action checks current generation/token. Stale work is discarded with no backend/session side effects.
5. **Failure safety**: synthesis/backend/start failures produce named refusal (`OpenError`/diag path), abandon playing landing where applicable, and never publish healthy audible playing state without valid start (FR-001/005).
6. **Invariants preserved**:
   - 22.050 Hz mono f32 handoff/queue assumptions unchanged
   - fingerprint/bookmark/session ownership unchanged
   - `SourceRange`/`resume_point`/`segment_index`/`sample_index_of` semantics unchanged
   - timeline/section measurements remain deterministic for committed current-generation units
   - `note_play_request`/`note_first_consumed_sample`/`play_to_audio_ms` remain meaningful

## Watch contract

- `playback_watch_tick` MAY: credit audible progress, publish crossing/state, arm/cancel chapter pre-roll checks, detect drain/end, wake/signal producer.
- `playback_watch_tick` MUST NOT: call `produce_next_unit`, call `adapter_synthesis::synth::synthesize`, or perform other CPU-bound synthesis.

## Lock contract

- Snapshot inputs under short `std::sync::Mutex` locks.
- Perform synthesis with no `AppState`/production/session/backend locks held.
- Reacquire locks briefly to validate generation and commit.
- No guard held across `.await`.
- Avoid nested lock acquisition patterns that can deadlock with cpal callback or Android contention paths.

## Test contract (minimum)

- RED/GREEN tests prove: watch has no synth call path; stale generation cannot enqueue/start; first useful audio path works for offset; FR-001/005 failure does not publish healthy playing; resume/fingerprint/pre-roll/timeline invariants unaffected in targeted cases.
- Verification evidence must include targeted nextest/clippy/fmt results and, for TTFA claim closure, 2201117TY diagnostic/device smoke (`play_to_audio_ms`).
