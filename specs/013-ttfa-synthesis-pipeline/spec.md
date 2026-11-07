# Feature Specification: TTFA Synthesis Pipeline Scheduling

**Feature Branch**: `013-ttfa-synthesis-pipeline`  
**Created**: 2026-11-07  
**Status**: Draft (ready for plan after assess `go`)  
**Input**: Mapa objetivo + assess `ttfa-synthesis-pipeline` (`.specify/assessments/ttfa-synthesis-pipeline/`) + device diagnostics (2201117TY, Android 13/SDK 33) + sherpa-onnx 1.13.8 API evidence.

## Problem / Evidence Summary

Current playback start runs CPU-bound sherpa-onnx/Piper VITS synthesis synchronously in `src-tauri/src/lib.rs` (`play_document` → `stream_synthesis` → `produce_next_unit` → `adapter_synthesis::synth::synthesize`). Continuous production also calls `commands::advance_production` from `playback_watch_tick` (Tokio async task, 500 ms interval), which can execute `synthesize` synchronously in that context. No `spawn_blocking`/dedicated synthesis thread was found in `src-tauri`/`player`.

Device diagnostic evidence (`app.mdreader` diagnostic record, 2026-11-07) shows 24 `play_to_audio_ms` samples: 1659–17309 ms common; 45878, 61802, 85949? (not present), 110466, 120422 ms extreme cases correlated with high `position_secs` at measurement. This is consistent with resume/offset paths synthesizing and discarding segments before first useful audio (`samples_before_entry`/`sample_index_of`), plus blocking synthesis on caller/tick paths.

sherpa-onnx 1.13.8 evidence: Rust `OfflineTts::generate_with_config` calls C API synchronously on the calling thread; VITS progress callback delivers post-synthesis batch audio, not intra-segment streaming. True Option B intra-model streaming is out of scope.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Fast cold start (Priority: P1)

As a reader, when I open a document and playback starts, I hear audio quickly without the UI/command path stalling.

**Independent Test**: open/play a representative document from offset 0 on target device; measure `note_play_request` → first consumed sample (`play_to_audio_ms`/diagnostic) and verify no ANR/crash.

**Acceptance Scenarios**:
1. **Given** a valid document and `StartNow` open path, **When** `play_document(..., 0.0)` is invoked, **Then** the system produces/enqueues the first useful PCM piece and starts backend playback with materially lower TTFA than baseline while preserving FR-001/005.
2. **Given** synthesis is in progress, **When** the first useful piece is not yet ready, **Then** no silent `Playing` audio state is reported as healthy playback without a valid start path; failures abandon/restore state per existing semantics.

---

### User Story 2 - Fast resume at stored/high offset (Priority: P1)

As a reader resuming at a stored position, I hear the correct resumed audio quickly instead of waiting for synthesis of discarded pre-offset segments.

**Independent Test**: resume at a high offset (document with long measured timeline) and compare TTFA/baseline; verify resumed position/fingerprint/bookmark semantics unchanged.

**Acceptance Scenarios**:
1. **Given** `start_offset_secs > 0` and `samples_before_entry` skips leading samples, **When** production begins, **Then** synthesis scheduling prioritizes reaching first useful pending audio for that offset without synthesizing unnecessary discarded work beyond what segment boundaries require.
2. **Given** re-anchor requires re-synthesis from stored position, **When** queued audio diverges, **Then** re-anchor uses the same safe production path and preserves resume/fingerprint/timeline invariants.

---

### User Story 3 - Continuous playback without underrun/blocking (Priority: P1)

As a reader, after first audio starts, playback continues smoothly while synthesis fills the 60 s resident window in background.

**Independent Test**: play 5+ minutes continuous reading; observe underruns/gaps, `buffer_state` cap 60 s, dropped samples, and no synthesis work inside `playback_watch_tick`.

**Acceptance Scenarios**:
1. **Given** backend buffer has room, **When** producer runs, **Then** it synthesizes/enqueues next segments up to `queue_room` and never exceeds `MAX_PREROLL_SECS`/ResidentQueue behavior.
2. **Given** buffer is full, **When** producer runs, **Then** it waits/yields without producing droppable audio; backend head remains contiguous.
3. **Given** playback watch ticks, **When** `playback_watch_tick` executes, **Then** it performs progress/crossing/drain/end/pre-roll-arm duties without calling CPU-bound `synthesize`.

---

### User Story 4 - Safe cancellation and state transitions (Priority: P1)

As a reader, if I open another document, stop, skip, or an error occurs during startup/production, the app does not play stale audio and does not get stuck.

**Independent Test**: cancel during first-segment synthesis and during background production; switch documents rapidly; induce backend/synthesis refusal.

**Acceptance Scenarios**:
1. **Given** a worker/producer owns a generation for document A, **When** document B opens or stop/cancel occurs, **Then** stale A audio is not enqueued/started and production is cancelled/invalidated safely.
2. **Given** synthesis or backend start fails, **When** error is handled, **Then** state abandons playing landing as needed, publishes coherent state, and never leaves silent false playback.

## Functional Requirements

- **FR-TTFA-001 (worker offload)**: CPU-bound synthesis (`adapter_synthesis::synth::synthesize`) MUST run off Tauri/UI/JNI/Tokio-async tick paths via a dedicated blocking worker/producer mechanism compatible with Tauri/Tokio and Android.
- **FR-TTFA-002 (first useful audio)**: Startup production MUST prioritize synthesizing until first useful pending audio for the requested `start_offset_secs` (accounting for `samples_before_entry`), enqueue a valid piece, and start backend playback as early as safely possible.
- **FR-TTFA-003 (watch decoupling)**: `playback_watch_tick` MUST NOT execute CPU-bound synthesis. It MAY signal producer demand but synthesis/handoff logic lives in producer/worker.
- **FR-TTFA-004 (60s bound/backpressure)**: Production MUST respect `MAX_PREROLL_SECS`/`ResidentQueue` semantics and `queue_room`/`producer::admits`; never depend on dropping head-contiguous consumed/resident audio to hide overproduction.
- **FR-TTFA-005 (resume/fingerprint invariants)**: Preserve blake3 fingerprint flow, `switch_to_document`/bookmark semantics, `sample_index_of`/`samples_before_entry` trimming, `SourceRange`/`resume_point`/`segment_index`, timeline/section measurements, and re-anchor behavior.
- **FR-TTFA-006 (FR-001/005 safety)**: Never report successful audible playback when start/first audio path fails. State transitions and publish events MUST be coherent across worker/caller boundaries.
- **FR-TTFA-007 (cancellation)**: Worker/producer MUST be cancellable/invalidation-safe on new document, stop, chapter transition, skip/seek behaviors that invalidate queued work, and app teardown paths. Chapter pre-roll hooks (T217/T232/T233) MUST NOT regress.
- **FR-TTFA-008 (measurements)**: Preserve `note_play_request` / `note_first_consumed_sample` / `play_to_audio_ms` semantics; add phase-level instrumentation only as additive, bounded diagnostic detail if needed.

## Key Entities / State

- **Production**: existing `segments`, `sections`, `next_unit`, `samples_before_entry`, `pending`, counters, `started`; extended only if needed for generation identity/cancellation.
- **Generation identity/token**: monotonic or unique token binding producer output to current document/session/open request.
- **Queue room**: samples backend can accept now under 60 s cap.
- **Resume start offset**: absolute/stream anchor and sample discard count before useful audio.

## Edge Cases

- High resume offset where many initial segments become fully discarded.
- First segment empty after trim/resume; loop must continue until useful audio or proper failure.
- Backend open succeeds but `play()`/enqueue fails after worker starts.
- Poisoned mutex/lock ordering during worker + watch + command interaction.
- Rapid open A→B→C; only C may affect audible output.
- Chapter advance/re-anchor while production active.
- Very short segments / headings-only content producing small PCM.

## Success Criteria

- **SC-TTFA-001**: On 2201117TY Android 13, representative `play_to_audio_ms` median improves materially from sampled baseline (common 1.6–17.3 s range) and extreme offset cases (>45 s) are eliminated or reduced to bounded/acceptable values by first-useful-segment scheduling.
- **SC-TTFA-002**: No underrun regression in 5+ minute continuous playback smoke; `preroll_cap_secs` remains 60 s, no unbounded queue growth.
- **SC-TTFA-003**: `playback_watch_tick` contains no synthesis CPU-bound path (code/trace/test evidence).
- **SC-TTFA-004**: Existing tests relevant to playback/session/resume/surface/diagnostics pass; new tests cover worker cancellation, offset-first-useful-audio, and FR-001/005 failure safety.
- **SC-TTFA-005**: Device smoke confirms resume/fingerprint/lock-screen/media controls remain functional for open, play/pause, resume, and chapter/document switch cases exercised.

## Assumptions

- `sherpa-onnx` 1.13.8 `OfflineTts` remains synchronous; callback is not intra-segment VITS streaming (evidence). `OfflineTts` is treated as `Send+Sync` per crate docs; actual contention/reuse is validated during implementation.
- cpal callback remains real-time and must never wait for synthesis/locks held by synthesis.
- Target validation device is available unless explicitly unavailable; missing device blocks SC-TTFA-001/005 smoke claims.

## Out of Scope

- Intra-model VITS/Piper streaming (Option B).
- Voice/model/asset changes, `num_threads` tuning beyond optional measured follow-up.
- Chunking/prosody/symbol-fatigue/translation/UI redesign.
- Completing T232/T233 behavior beyond preserving current hooks/non-regression.

## Dependencies

- Tauri 2.12.1 runtime (`tauri::async_runtime`/Tokio) for blocking/background work.
- `adapter-synthesis`/`player`/`session`/`diag` contracts.
- Android device `adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp` / model 2201117TY for smoke.