# Converge/Clarify: TTFA synthesis pipeline

- **Slug**: ttfa-synthesis-pipeline
- **Date**: 2026-11-07
- **Scope**: resolve `needs-clarification` blockers from `decision.md` with code + device + API evidence; no source-code changes.

## Device evidence (target 2201117TY, Android 13 / SDK 33, app.mdreader)

Source: `run-as app.mdreader` diagnostic record `/data/data/app.mdreader/diagnostic-record.jsonl` (24 `play_to_audio_ms` entries observed).

| `play_to_audio_ms` (ms) | `position_secs` at progress | Interpretation (evidence-bounded) |
|---:|---:|---|
| 1659–4770 | 0.08–3.55 | Lower range; includes non-zero offsets in some cases. |
| 6310–17309 | 0.03–11.26 | Common startup/resume range in sampled entries. |
| 45878 | 5.44 | High TTFA despite moderate position — needs phase breakdown; consistent with CPU-bound synthesis/start path contention risk. |
| 61802–120422 | 85.98–147.13 | **Extreme TTFA** correlated with high `position_secs` at measured progress. Consistent with resume/offset path synthesizing+discarding segments until `samples_before_entry`/useful audio (`produce_next_unit` trims via `sample_index_of`; `stream_synthesis` loops while `pending_samples==0`). Causality by phase still requires instrumentation. |

Context sample (doc `4c16...`): `session_start/play` at `position_secs=0.0` → progress with `play_to_audio_ms=16815` and later `buffer preroll_secs=11.597 preroll_cap_secs=60.000 dropped_samples=0`.

**Conclusion:** TTFA is materially above the 3 s target in observed cases and can become catastrophic on high-offset starts. A fix must address first-useful-segment production for resume offsets, not only cold-start first segment.

## Code/architecture clarifications

1. **Synthesis location:** `adapter_synthesis::synth::synthesize(&segment)` in `commands::produce_next_unit` (`src-tauri/src/lib.rs:5286+`), called by `stream_synthesis` (`5041-5100`) and `advance_production` (`5120-5153`). `play_document` (`5767-5899`) calls `stream_synthesis` synchronously after landing `Playing`.
2. **Runtime/blocking:** Tauri `open_document` is a sync command (`3332`); `run_mobile_plugin` dispatches play/open paths; JNI `shell_jni.rs` calls ingest/open/play paths directly. No `spawn_blocking`/dedicated synthesis thread found in `src-tauri`/`player`. `playback_watch_tick` is spawned via `tauri::async_runtime::spawn` with 500 ms Tokio interval and calls `commands::advance_production` (`2337-2354,2455-2515`), so CPU-bound synthesis can run in that async task context — **architectural blocker** (Tokio/Tauri guidance: move CPU-bound work off async executor/UI/JNI-sensitive paths).
3. **FR-001/005 safe state design (spec constraint):** never publish/report audible `Playing` when no audio can start. Current design lands `Playing` before synthesis and abandons on error (`abandon_playing_landing`). Minimal safe change: keep fast prepare/open/switch deterministic; move synthesis/start handoff to worker; publish `Playing`/start backend only after first valid piece is enqueued and stream start succeeds, or publish an explicit preparing/loading state if already supported by UI contract — spec must choose and verify against status_view/surface contracts. All failures must abandon/restore non-playing state and surface named refusal/error.
4. **Sherpa-onnx bindings (confirmed, local + upstream source):** `sherpa-onnx = "=1.13.8"`, vendored `sherpa-onnx-sys` 1.13.8. Rust `OfflineTts::generate_with_config` installs a progress callback trampoline and calls `SherpaOnnxOfflineTtsGenerateWithConfig` **synchronously** on the calling thread (`~/.cargo/registry/.../sherpa-onnx-1.13.8/src/tts.rs:520-700+`). Callback receives samples/progress and can stop early; samples valid only during callback. C++ VITS impl (`v1.13.8` `offline-tts-vits-impl.h`): for one batch runs `Process` (one `model_->Run`) producing whole batch audio, then invokes callback; batched long text invokes callback after each synthesized batch and still returns full accumulated audio. **Implication:** progress callback is **not** intra-segment/model-streaming for VITS/Piper; only sentence/batch granularity post-synthesis. Option B (true split streaming) remains unconfirmed/unscoped. Option A segment-level pipelining is the correct lever.
5. **Chapter pre-roll impact (T217/T232/T233):** `chapter_preroll` state exists (`NextChapterPreRoll`, `begin/should_trigger/cancel_and_purge` in `crates/player/src/skip.rs`; armed in `maybe_start_chapter_preroll` `lib.rs:2799+`). No actual next-chapter synthesis/enqueue path observed in scanned code; `advance_chapter_queue` opens next chapter via `open_document_bytes_named` (`lib.rs:2714+`). Tasks mark T232/T233 open. **Constraint:** implementation must preserve current behavior and cancellation semantics; any new producer must respect chapter transitions/cancel tokens.

## Decision update

Evidence now adequate to **go for a narrow spec/plan** (Option A, segment-level scheduling worker + producer decoupled from playback watch + offset-aware first-useful-unit fast path), with TDD and device validation mandatory before release. Exclude intra-model VITS streaming (Option B) until sherpa-onnx API/assets prove equivalent incremental synthesis.

