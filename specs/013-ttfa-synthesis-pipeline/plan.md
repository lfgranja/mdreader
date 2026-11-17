# Implementation Plan: TTFA Synthesis Pipeline Scheduling

**Branch**: `013-ttfa-synthesis-pipeline` | **Date**: 2026-11-07 | **Spec**: [`spec.md`](./spec.md)

**Input**: Feature specification from `specs/013-ttfa-synthesis-pipeline/spec.md` + assess `ttfa-synthesis-pipeline` (`.specify/assessments/ttfa-synthesis-pipeline/`, decision `go`) + code trace/device diagnostics/sherpa-onnx 1.13.8 + Tauri/Tokio/cpal authoritative guidance.

## Summary

Reduce TTFA and remove blocking synthesis by refactoring playback production to **Option A (segment-level scheduling)**: keep deterministic prepare/open/switch work fast; move CPU-bound `adapter_synthesis::synth::synthesize` to a dedicated blocking producer path; enqueue/start the **first useful PCM piece** as early as safely possible (including resume offsets via `samples_before_entry`); continue production under `MAX_PREROLL_SECS`/`queue_room` backpressure; make `playback_watch_tick` non-synthesizing. Preserve FR-001/005 (no silent false playing), pre-roll 60 s, fingerprint/bookmark, resume/re-anchor, `SourceRange`/`segment_index`/`resume_point`, timeline/section measurements, and chapter hooks/T217/T232/T233 non-regression.

## Technical Context

**Language/Version**: Rust 2024, stable, workspace lints deny `unwrap/expect/panic/todo/dbg/println`; Tauri 2.12.1; Android minSdk 29 (`aarch64-linux-android` gate 9).  
**Primary Dependencies**: `tauri` async runtime/Tokio-backed, `std::sync::{Mutex, atomic}`, `sherpa-onnx = "=1.13.8"` (`OfflineTts::generate_with_config` synchronous, calling-thread, VITS batch/post-synthesis callback only), `cpal` 0.18.2 (`CpalBackend` + `ResidentQueue` 60 s), `adapter-synthesis` chunk/synth, `player` transport/position/queue/producer, `session` fingerprint/bookmark, `diag`.  
**Storage**: N/A for schema changes; bookmarks/diagnostics existing stores unchanged semantically.  
**Testing**: `cargo nextest --workspace` (host), targeted unit/integration in `src-tauri`, `crates/player`, `crates/adapter-synthesis`/session as needed; TDD RED→GREEN; Android smoke on 2201117TY (SDK 33) via ADB/diagnostics for TTFA.  
**Target Platform**: Android device (AAudio via cpal), host CI for gates 1–8.  
**Project Type**: Rust workspace + Tauri mobile + Kotlin shell mirror.  
**Performance Goals**: materially lower `play_to_audio_ms`; eliminate extreme high-offset cases (45–120 s observed) by first-useful-segment scheduling; no underrun regression in 5+ min smoke.  
**Constraints**: CPU-bound synthesis never on Tokio async tick, Tauri UI/command-sensitive caller path, JNI callback-sensitive path, or cpal callback. cpal callback real-time safe only (queue/clock/buffer fill, silence fallback). `std::sync::Mutex` guards short, no nested multi-lock deadlocks, never held across `.await`; snapshot outside lock, synth outside lock, commit under short locks. `ResidentQueue`/60 s bound preserved. Sherpa callback is not intra-segment VITS streaming.  
**Scale/Scope**: playback start/production path in `src-tauri/src/lib.rs` + minimal supporting changes; no format/chunking/fingerprint/prosody/model changes.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design and before implementation merge.*

| Gate / Principle | Status | Notes |
|---|---|---|
| Rust sole playback/synthesis authority | PASS | Producer/worker remains Rust; Kotlin shell unchanged behaviorally except receiving coherent published state. |
| Pure offline / no network | PASS | No new deps/network paths. |
| Driving-first / coherent state | PASS | TTFA improves responsiveness; FR-001/005 forbids silent false playing. |
| Closed sets / explicit failures | PASS | Use existing `OpenError`/diag/refusal paths; no silent fallback. |
| Pre-roll bound 60 s / no overrun drop holes | PASS | Keep `MAX_PREROLL_SECS`, `ResidentQueue`, `queue_room`/`producer::admits`. |
| Fingerprint/resume/SourceRange invariants | PASS | Preserve `switch_to_document`, bookmarks, `sample_index_of`/`samples_before_entry`, timeline/section measurements. |
| TDD / host-checkable gates 1–8 | PASS (required) | RED tests for cancel/generation, offset-first-useful, watch-no-synth, FR-001/005 failure, resume invariants; run targeted nextest/clippy/fmt. Gate 9 device smoke required for TTFA claim. |
| Lint/safety (no unwrap/panic/dbg/println) | PASS | Follow workspace denies. |

**Post-design re-check required** after `data-model.md`/`contracts/` confirm generation-token/cancellation and lock discipline do not introduce nested locks or cpal contention.

## Project Structure

### Documentation (this feature)

```text
specs/013-ttfa-synthesis-pipeline/
├── plan.md
├── spec.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── synthesis-producer.md
├── checklists/
│   └── requirements.md
└── tasks.md             # created next
```

### Source Code / Artifacts (repository root)

```text
src-tauri/src/lib.rs                 # play_document/stream_synthesis/advance_production/watch/tick/state
src-tauri/tests/                     # targeted TTFA/production/cancellation/FR-001/005 tests
crates/player/src/backend.rs         # unchanged semantics; cpal callback real-time contract preserved
crates/player/src/position.rs        # unchanged audible-clock authority
crates/adapter-synthesis/src/synth.rs # unchanged sync API unless additive diagnostics; no Option B
crates/session/src/*                 # fingerprint/bookmark unchanged
.specify/assessments/ttfa-synthesis-pipeline/*  # assess evidence/decision (already written)
```

**Structure Decision**: Keep changes localized to Tauri playback production/scheduling in `src-tauri/src/lib.rs` (and tests), reusing existing `player`/`adapter-synthesis`/`session`/`diag` contracts. Avoid new crates/deps unless implementation proves necessary.

## Phase 0 Research Focus (complete enough for plan)

1. **Blocking/runtime**: Tauri async commands preferred for heavy work; `spawn_blocking` for bounded blocking units; dedicated `std::thread` for long-lived producer if needed. `tokio::task::spawn_blocking` tasks are not abortable mid-blocking call; shutdown/cancel must be cooperative/invalidation-based. (Tauri v2 calling-rust/state/runtime, Tokio spawn_blocking/mutex docs.)
2. **State transitions**: current path lands `Playing` before synthesis and abandons on error. Refactor must preserve FR-001/005 — prefer preparing/opening deterministically, starting backend/enqueueing first useful audio from producer, and publishing coherent playing/abandoned state only on valid transitions; avoid publishing audible healthy playback before start succeeds.
3. **Resume/high offset**: `samples_before_entry` trims produced samples; extreme TTFA evidence indicates need to reach first useful segment quickly. Design may scan/skip segments whose full duration cannot contribute, but only if timeline/section measurement/resume-point invariants remain exact; otherwise synthesize in order and discard minimal samples. This requires explicit tests.
4. **Cancellation**: generation token (`AtomicU64`/monotonic id) checked before enqueue/start/commit; `Production` invalidated on new doc/stop/chapter/re-anchor. Sherpa synchronous call cannot be interrupted; stale result must be discarded.
5. **Watch decoupling**: `playback_watch_tick` becomes observation-only for production (drain/end gate via `production_running`/buffer state) and may wake/signal producer via cheap flag/channel; no `synthesize`.

## Complexity Tracking

| Violation / Risk | Why Needed | Simpler Alternative Rejected Because |
|---|---|---|
| Background producer + generation token | Remove CPU-bound work from command/JNI/watch and prevent stale audio | Keeping synthesis in `stream_synthesis`/`playback_watch_tick` preserves measured TTFA/blocking pathology |
| Careful lock choreography | `AppState` has many `std::sync::Mutex` fields and cpal callback | Coarse global lock risks callback underrun/deadlock (Tauri #14694 risk class) |
