# Data Model / Runtime Entities: TTFA Synthesis Pipeline Scheduling

**Feature**: `013-ttfa-synthesis-pipeline`  
**Scope**: in-memory production/scheduling only (no DB/schema/API changes). Existing persisted entities (`session` fingerprint/bookmark store, diagnostics records) retain semantics.

## Existing entities (preserved)

### PlaybackRequest / open path
- Inputs: document bytes/name/path context, `start_offset_secs`, `resume_class` (StartNow/AwaitOffer/AwaitExplicitResume/interrupted resume via JNI).
- Preserved behavior: fingerprint blake3, `lookup_on_open`, `switch_to_document`, `prepare_session_document`, `park_cold_start` for offer/await paths.

### LoadedDocument
- `text`, `name` (and related state in `AppState`).
- Role: synthesis input and surface title source.

### Production (current)
- `segments: Vec<DocumentSegment>`
- `next_unit: usize`
- `samples_before_entry: usize` (trim before resolved position)
- `pending: VecDeque<f32>`
- `total_samples/played_samples`
- section measurement cursor
- `started: bool`

### ResidentQueue / backend pre-roll
- Bound: `MAX_PREROLL_SECS = 60` @ 22.050 Hz mono f32.
- Contract: bounded resident audio, backpressure via `queue_room`/`producer::admits`; callback consumes FIFO; audible progress only on consumed samples.

### Timeline/section measurements
- Unit end and section positions measured during production; session timeline installed as measured.

## Proposed/required runtime entities (Option A)

### ProductionGeneration
- **Type**: monotonic `u64` (e.g. `AtomicU64` in `AppState` or producer control).
- **Lifecycle**: incremented on new authoritative open/start/switch/re-anchor/chapter transition/stop/reset that invalidates prior producer work.
- **Invariants**:
  - A producer result tagged with stale generation MUST NOT enqueue PCM, start backend, install timeline/session state for stale request, or publish as current audible progress.
  - Generation checks occur at: before worker starts producing a unit/result commit, before `enqueue_chunk`, before `start_backend`/first-start transition, before publishing current playback as successfully started for that generation.

### ProducerControl / cancellation
- **State**: current generation + cancellation flag/epoch + optional wake/signal for demand.
- **Operations**:
  - `invalidate_current()` / `begin_generation(new_or_next)` — marks old work stale, clears/purges pending production state as required.
  - `is_current(gen)` — cheap atomic check.
  - cooperative cancel: synchronous `synthesize` cannot be interrupted; worker discards result if stale after call.
- **Integration points**: `open_document_bytes_named`/switch, stop, re-anchor/restart, chapter transitions, app/session teardown paths as needed.

### ProductionCommit (conceptual)
- Produced unit payload ready to commit only if generation current:
  - `generation`
  - `unit_index` / segment identity
  - produced PCM (or trimmed result)
  - measurement deltas (`unit_end_samples`, section position info, samples to hand over after `samples_before_entry`)
- **Commit rule**: take short `production`/`backend`/`session`/diag locks only for commit slices; never hold across synthesis.

### FirstUsefulAudio readiness
- Condition: current generation has pending/committed PCM that represents audible audio at/after resolved `start_offset_secs` and backend can accept/start it.
- Effect: may trigger `enqueue_chunk` and, if not started, `start_backend` + `mark_production_started` + `note_first_consumed_sample` once consumption occurs; publication must remain FR-001/005-safe.

## State transitions (target)

```text
Request open/start
  -> invalidate old generation (if switching)
  -> deterministic prepare (fingerprint/bookmark/switch/park or start intent)
  -> if StartNow/resume play:
       acquire/prepare backend (named refusal on failure)
       create Production for generation G
       schedule producer work (no async tick synth)
       produce/commit until first useful piece ready for G
       enqueue + start backend + publish coherent started state
       continue producer under queue_room until end/cancel
  -> if cancelled/stale/failure before safe start:
       discard results, abandon landing as needed, publish refusal/paused coherent state
```

## Validation rules

- `generation` comparison is required before any stale-visible side effect.
- `samples_before_entry` trim is sample-accurate and never produces negative/overflow state.
- `queue_room` respected; never exceed 60 s resident bound semantics.
- timeline/section measurements remain consistent with produced units actually committed for current generation.
- cpal callback receives no synthesis/producer locks beyond existing queue/clock minimal critical sections.
