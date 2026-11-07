# Phase 0 Research: TTFA Synthesis Pipeline Scheduling

**Feature**: `013-ttfa-synthesis-pipeline`  
**Date**: 2026-11-07  
**Decision baseline**: `.specify/assessments/ttfa-synthesis-pipeline/decision.md` = `go` (Option A)

## Decisions

### D1. Production model: segment-level worker/producer (Option A)
- **Decision**: Offload `adapter_synthesis::synth::synthesize` to a blocking worker/dedicated producer; keep segment-level pipelining.
- **Rationale**: sherpa-onnx 1.13.8 VITS progress callback is post-batch/synchronous and API returns full audio; not intra-segment streaming. Segment-level scheduling matches existing `DocumentSegment`/`Production` model and 60 s resident queue.
- **Alternatives considered**: Option B intra-model streaming — rejected (API/assets/equivalence unconfirmed). Tuning `num_threads` only — rejected as primary fix (empirical, does not fix blocking path).

### D2. Runtime offload boundary
- **Decision**: CPU-bound synthesis runs on `tauri::async_runtime::spawn_blocking` bounded units or a dedicated `std::thread` producer loop; never in cpal callback, never in async Tokio watch/tick body, never on UI-sensitive command/JNI path in blocking form.
- **Rationale**: Official Tauri/Tokio guidance: async commands/async runtime for heavy work; `spawn_blocking` for bounded blocking work; blocking Tokio workers is problematic; `spawn_blocking` tasks not abortable mid-call.
- **Alternatives considered**: synthesize in `playback_watch_tick` — rejected. Large parallel synthesis — rejected (CPU contention, memory, queue overproduction).

### D3. Cancellation/invalidation
- **Decision**: Use monotonic generation/token identity for active production/document-open request. Check token before enqueue/start/commit/publish; invalidate on new document, stop, chapter transition, re-anchor, teardown. Discard stale synth results.
- **Rationale**: synchronous ONNX inference cannot be interrupted; stale work must not affect backend/session/fingerprint state.
- **Alternatives considered**: task abort only — insufficient for `spawn_blocking`/synchronous inference once started.

### D4. FR-001/005 state safety
- **Decision**: preserve “no silent false playing”. Startup must not publish healthy audible playback before first useful audio handoff/start succeeds; failures abandon playing landing and publish coherent refusal/state.
- **Rationale**: existing `abandon_playing_landing`, refusal paths, and device UX contracts. Android/wry lock/contention risk requires careful state transitions.
- **Alternatives considered**: keep landing `Playing` before synth and rely only on abandonment — risky once producer is async/background; requires stricter transition design/tests.

### D5. Resume/high-offset TTFA
- **Decision**: prioritize first useful audio for `start_offset_secs` by advancing production until pending useful audio exists, respecting `sample_index_of`/`samples_before_entry`. Optional future optimization: pre-skip whole segments using measured/known durations only when timeline invariants remain exact; initial implementation should prefer correctness (synthesize in order, discard samples) unless tests prove safe skip.
- **Rationale**: device evidence shows 45–120 s TTFA correlated with high offsets; first-useful-audio is mandatory. Exact timeline/section measurement currently tied to unit production.

### D6. Watch responsibilities
- **Decision**: `playback_watch_tick` remains lightweight: feed audible progress, crossing/republish, chapter pre-roll arm, drain/end gated by `production_running`, and optional producer wake/signal; no `produce_next_unit`/`synthesize`.
- **Rationale**: prevents blocking Tokio runtime and keeps 500 ms cadence responsive.

## Evidence URLs (authoritative/high-reliability)

- Tauri v2 calling Rust/commands: https://v2.tauri.app/develop/calling-rust/
- Tauri v2 state management: https://v2.tauri.app/develop/state-management/
- Tauri async runtime: https://docs.rs/tauri/latest/tauri/async_runtime/
- Tauri `spawn_blocking`: https://docs.rs/tauri/latest/tauri/async_runtime/fn.spawn_blocking.html
- Tokio `spawn_blocking`: https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html
- Tokio `Mutex`: https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html
- cpal `DeviceTrait`/callbacks: https://docs.rs/cpal/latest/cpal/traits/trait.DeviceTrait.html
- cpal `StreamTrait`: https://docs.rs/cpal/latest/cpal/traits/trait.StreamTrait.html
- cpal `BufferSize`: https://docs.rs/cpal/latest/cpal/enum.BufferSize.html
- ONNX Runtime threading: https://onnxruntime.ai/docs/performance/tune-performance/threading.html
- ONNX Runtime XNNPACK EP: https://onnxruntime.ai/docs/execution-providers/Xnnpack-ExecutionProvider.html
- sherpa-onnx 1.13.8 C++/Rust source refs (callback synchronous/post-batch, VITS single `model_->Run` per batch): https://raw.githubusercontent.com/k2-fsa/sherpa-onnx/v1.13.8/sherpa-onnx/csrc/offline-tts.h , https://raw.githubusercontent.com/k2-fsa/sherpa-onnx/v1.13.8/sherpa-onnx/csrc/offline-tts-vits-impl.h , https://raw.githubusercontent.com/k2-fsa/sherpa-onnx/v1.13.8/sherpa-onnx/rust/sherpa-onnx/src/tts.rs
- Risk signal (Android mutex/contention): https://github.com/tauri-apps/tauri/issues/14694

## Open Implementation Notes (non-blocking for spec/plan)

- Whether to use bounded `spawn_blocking` per synthesis step signaled by watch/wakeups vs a dedicated `std::thread` producer: choose based on cancellation/serialization simplicity and Android behavior during implementation TDD.
- Exact FR-001/005-safe publication point (current surface contracts) must be verified against `status_view`/publish tests before merge.
- Optional additive phase diagnostics (`prepare/open/synth_first/start`) can help SC-TTFA-001 proof.
