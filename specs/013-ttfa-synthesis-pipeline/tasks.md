# Tasks: 013 — TTFA Synthesis Pipeline Scheduling

**Input**: `specs/013-ttfa-synthesis-pipeline/{spec.md,plan.md,research.md,data-model.md,contracts/synthesis-producer.md,quickstart.md}`  
**Assess decision**: `.specify/assessments/ttfa-synthesis-pipeline/decision.md` = `go` (Option A).  
**Branch**: `013-ttfa-synthesis-pipeline` (repo currently `dev`; feature workspace is the spec folder).  
**Tests**: REQUIRED (TDD). RED tasks MUST exist and FAIL before implementation; do not mark RED complete by making them pass. Verification/attestation tasks are Phase 7 only.

## Execution Rules (mandatory)

- **TDD RED gate**: For each wave, write listed tests first and record failing/missing-contract evidence before implementation in that scope. Use `#[ignore]` only for device-only/manual smoke where host cannot run it; host-checkable behavior must be executable RED.
- **Small blast radius**: modify `src-tauri/src/lib.rs`, targeted `src-tauri/tests/*` (or crate tests named below), and minimal supporting code only. Preserve `adapter-synthesis` API/behavior unless additive diagnostics are necessary and non-breaking.
- **No Option B**: do not add sherpa intra-segment streaming or change synthesis engine/model/asset contracts.
- **Invariants**: 60 s `MAX_PREROLL_SECS`/`ResidentQueue`, fingerprint/blake3/bookmarks, `switch_to_document`, `sample_index_of`/`samples_before_entry`, `SourceRange`/`resume_point`/`segment_index`, timeline/section measurements, FR-001/005, cpal callback real-time safety.
- **Lock discipline**: `std::sync::Mutex` guards short; snapshot outside lock, synth outside lock, commit under short locks; never hold guards across `.await`; avoid nested multi-lock deadlocks and cpal contention.
- **Atomic commits**: when editing a file finishes, create an atomic conventional commit with detailed summary of edits made (one logical edit unit/file-finish event unless repo owner documents narrower batch; never mix unrelated events).

## Path Conventions

- Bridge/production authority: `src-tauri/src/lib.rs`
- Bridge tests: `src-tauri/tests/test_ttfa_synthesis_pipeline.rs` (new, targeted), plus existing `src-tauri/tests/test_play_to_audio_measured.rs` compatibility checks as needed
- Backend invariants: `crates/player/src/backend.rs` (preserve semantics; tests only if behavior changes)
- Synthesis: `crates/adapter-synthesis/src/synth.rs` (prefer unchanged)
- Session/fingerprint: `crates/session/src/*` (preserve semantics)

---

## Phase 1: Setup / Test Harness (Shared)

**Purpose**: establish targeted TTFA/production test scaffolding and baseline evidence hooks.

- [~] T001 (ttfa_013_tests module used as targeted host harness; dedicated integration file optional) Create targeted test module `src-tauri/tests/test_ttfa_synthesis_pipeline.rs` with shared helpers for `AppState`/production setup, fake or controllable synthesis boundaries where feasible, and generation assertions; prefer host-checkable seams over real sherpa inference in unit-level RED where possible. Record `N/A:<reason>` if a seam is unavailable and use integration-style harness.
- [x] T002 [P] Capture baseline evidence references in `specs/013-ttfa-synthesis-pipeline/quickstart.md` (append evidence log section): device 2201117TY `play_to_audio_ms` range from handoff, code refs (`play_document`, `stream_synthesis`, `produce_next_unit`, `advance_production`, `playback_watch_tick`), and current watch→synth coupling observation.

---

## Phase 2: Foundational — Generation & Producer Control (Blocking)

**Purpose**: add safe invalidation primitives before moving synthesis. Blocks all user stories.

### RED (write first)

- [~] T003 (host tests: stale active target, stale bounded no-op/no-start, stale commit side-effect discard, stale commit mutates no current production; hook-driven first-useful commit covered by T007 host test; `enqueue_chunk_for_generation`/`start_backend_for_generation` added; full publish/enqueue-start-after-return matrix still pending) [US1][US3] In `src-tauri/tests/test_ttfa_synthesis_pipeline.rs`, add failing tests that a stale production generation cannot commit PCM, cannot start backend/playback for stale request, and cannot publish healthy started state after invalidation/switch/stop. Cover race-shaped case: result arrives after generation changed.
- [~] T004 (runtime watch synthesis-free refactor+smoke; stronger structural assertion pending) [P] [US1][US2] In `src-tauri/tests/test_ttfa_synthesis_pipeline.rs`, add failing test/contract assertion that `playback_watch_tick`/watch path does not perform synthesis (`produce_next_unit`/`adapter_synthesis::synth::synthesize`) and remains lightweight for production advancement (observation/wake only).

### Implementation (blocked until T003–T004 RED)

- [~] T005 (generation/wake/driver fields and invalidation wired; full transition coverage pending) [US3] Add generation/cancellation control to `AppState` and production flow in `src-tauri/src/lib.rs` (e.g. monotonic `AtomicU64` generation + current production generation; invalidation helpers). Ensure open/switch/stop/re-anchor/chapter transitions invalidate old generation; keep lock scope short.
- [~] T006 (commit/start paths generation-checked in refactored flow; `handover_pending_for_generation` uses generation-checked backend enqueue/start; exhaustive guard audit across publish/session/timeline/after_execute still pending) [US1][US3] Refactor production commit path in `src-tauri/src/lib.rs` so `enqueue_chunk`/`start_backend`/timeline/session commit/publish helpers check current generation and discard stale work with no side effects.

**Checkpoint**: stale work cannot affect active playback; watch-synth coupling is identified and ready to remove.

---

## Phase 3: User Story 1 — Fast start and continued production (P1) 🎯 MVP

**Goal**: start useful audio quickly and continue synthesis without blocking watch/startup critical path, preserving 60 s backpressure.

**Independent Test**: from StartNow play request, first useful PCM is enqueued/started without waiting for full-document synthesis; watch tick remains non-synthesizing; backpressure honors 60 s.

### RED

- [~] T007 [P] [US1] Host hook-driven test `ttfa_013_tests::t007_produce_bounded_commits_first_useful_audio_via_synth_hook` proves `produce_bounded` commits first useful synthesized unit (next_unit/total advanced, pending or started). Full multi-segment no-full-loop/end-to-end startup proof still pending.
- [ ] T008 [P] [US1] Add failing test: producer continues under `queue_room`/`MAX_PREROLL_SECS` backpressure without dropping committed audio and without blocking cpal callback contract assumptions (host-level queue/admission assertions acceptable where direct callback test is impractical).

### Implementation

- [~] T009 (spawn_blocking demand driver + bounded produce_bounded; startup still performs bounded inline first-useful work; dedicated robustness pending) [US1] Move CPU-bound synthesis in `src-tauri/src/lib.rs` off async watch/critical caller path: bounded `tauri::async_runtime::spawn_blocking` units or dedicated producer thread. Snapshot segment/production inputs under short locks, call `adapter_synthesis::synth::synthesize` outside locks, validate generation, commit under short locks.
- [x] T010 [US1] Refactor `playback_watch_tick` in `src-tauri/src/lib.rs` to be synthesis-free: keep audible progress crediting, crossing/republish, chapter pre-roll arm, drain/end gated by `production_running`/buffer state; replace direct `advance_production` synthesis with wake/signal or safe non-blocking coordination to producer.
- [~] T011 (startup reordered to first-useful before Playing publish; abandon_landing seam tested; full FR matrix pending) [US1] Preserve FR-001/005-safe startup in `play_document`/open path (`src-tauri/src/lib.rs`): no healthy audible playing publish before first valid useful-audio start/handoff for current generation; failures use `abandon_playing_landing`/named refusal paths.
- [ ] T012 [US1] Re-run RED tests T004/T007/T008/T003 for MVP scope; record GREEN evidence in task notes/commit body (non-attesting until Phase 7).

**Checkpoint**: MVP — fast first useful audio path and non-synthesizing watch, with stale-generation safety.

---

## Phase 4: User Story 2 — Fast resume / high offset (P1)

**Goal**: resume/high `start_offset_secs` reaches first useful audio without unnecessary discarded leading work beyond required boundaries.

**Independent Test**: for high offset, production reaches first useful pending audio quickly; `samples_before_entry` trim remains sample-accurate; timeline/section/resume invariants preserved.

### RED

- [ ] T013 [P] [US2] Add failing test in `src-tauri/tests/test_ttfa_synthesis_pipeline.rs`: resume offset near later segment starts backend/enqueues useful audio without synthesizing every leading segment in full when a correctness-preserving fast path is implemented; if initial correctness-first path still synthesizes in order, test must instead assert minimal discard + bounded early useful handoff behavior and be named accordingly.
- [ ] T014 [P] [US2] Add failing invariant tests: `sample_index_of`/`samples_before_entry` trimming, `segment_index`/`resume_point`/`SourceRange` semantics, and timeline/section measurements remain correct for committed current-generation units after offset start.

### Implementation

- [ ] T015 [US2] Implement first-useful-audio resume scheduling in `src-tauri/src/lib.rs`: prioritize reaching non-empty pending useful audio for `start_offset_secs`. Correctness-first default: synthesize in order and discard only required samples; optional measured-duration segment skipping allowed only with tests proving timeline/section/resume exactness.
- [ ] T016 [US2] Ensure `install_session_timeline`/measured unit/section updates and resume/re-anchor state (`stream_anchor_secs`, bookmark/position behavior via existing session path) remain consistent under new producer commits.
- [ ] T017 [US2] Re-run RED tests T013/T014 and regression subset for resume/fingerprint paths; record evidence.

**Checkpoint**: high-offset startup no longer pays avoidable discarded-work TTFA while preserving resume correctness.

---

## Phase 5: User Story 3 — Cancellation and failure safety (P2)

**Goal**: switching/cancelling during startup/production leaves no stale audible work and no false playing state.

**Independent Test**: invalidated generation cannot enqueue/start/publish; synth/backend failure abandons playing landing and publishes refusal.

### RED

- [ ] T018 [P] [US3] Add failing tests in `src-tauri/tests/test_ttfa_synthesis_pipeline.rs` for cancellation during/after synth: stale generation result discarded; backend not started for stale generation; no late `Playing`/audible publish.
- [ ] T019 [P] [US3] Add failing FR-001/005 tests: synthesis error or backend start failure after play request does not leave healthy audible `Playing`; `abandon_playing_landing`/refusal observable as appropriate.

### Implementation

- [ ] T020 [US3] Wire invalidation points in `src-tauri/src/lib.rs`: `open_document_bytes_named`/`switch_to_document` interactions, stop, re-anchor/restart backend stream, chapter advance hooks, and teardown paths as required. Avoid deadlocks with `BookmarkStore` lock-last order.
- [ ] T021 [US3] Add explicit stale-result guards before every side-effecting producer commit/start/publish; log bounded diagnostics for discarded stale/cancelled work when useful (no `println`).
- [ ] T022 [US3] Re-run RED tests T003/T018/T019 and startup failure cases; record evidence.

**Checkpoint**: no stale audio/state leaks across cancellation or failures.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: diagnostics, docs, and non-regression cleanup across stories.

- [ ] T023 [P] Add additive bounded phase diagnostics for TTFA proof in `src-tauri/src/lib.rs` / diag path if low-risk: `note_play_request`→prepare/open→first synth start/complete (producer-observed)→first enqueue→`start_backend`→first consumed sample. Keep JSONL schema stable or version/additive only.
- [ ] T024 [P] Update `specs/013-ttfa-synthesis-pipeline/quickstart.md` and assess/converge notes with implementation evidence pointers; keep chapter pre-roll T232/T233 status unchanged.
- [ ] T025 Review `src-tauri/src/lib.rs` lock ordering/comments after refactor; remove dead code paths made obsolete by producer model; keep workspace clippy/deny compliance.

---

## Phase 7: Verification & Attestation (only phase that may claim completion)

**Purpose**: host gates and device TTFA evidence. Do not claim TTFA fixed without device data.

- [ ] T026 Run targeted host checks and record results: `cargo fmt --all -- --check`; `cargo nextest run -p mdreader --test test_ttfa_synthesis_pipeline` (and related impacted tests); `cargo clippy -p mdreader --all-targets -- -D warnings` (expand to workspace as needed by repo rules).
- [ ] T027 Device smoke on 2201117TY (SDK 33, `adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp`): install/debug flow per repo, exercise cold open, high-offset resume, switch during startup, 5+ min playback; capture `diagnostic-record.jsonl` `play_to_audio_ms`, buffer/underrun/responsiveness observations. Mark as blocked with explicit reason if device unavailable.
- [ ] T028 Attest spec outcomes in `specs/013-ttfa-synthesis-pipeline/checklists/requirements.md` or a convergence note: map evidence to `SC-TTFA-001..004`, `SC-RESUME-001`, `SC-CANCEL-001`, `SC-SAFE-001`; state whether device TTFA claim is proven, partial, or blocked. Do not mark T232/T233 complete.

---

## Dependencies & Execution Order

- **Phase 1**: starts immediately.
- **Phase 2 (T003–T006)**: blocks US1/US2/US3 implementation; T003–T004 RED first.
- **Phase 3 (US1)**: MVP after Phase 2; T007–T008 RED before T009–T011.
- **Phase 4 (US2)**: depends on Phase 2; can proceed after US1 core producer exists (shared `src-tauri/src/lib.rs` context — same owner/subagent wave).
- **Phase 5 (US3)**: depends on Phase 2; integrates with US1/US2 producer and state transitions.
- **Phase 6**: after desired stories complete.
- **Phase 7**: after implementation; device evidence may block TTFA closure claim only.

## Parallel Opportunities

- T002, T004, T007, T008, T013, T014, T018, T019, T023, T024 are parallelizable where file/context is disjoint; tasks touching `src-tauri/src/lib.rs` (T005,T006,T009-T011,T015,T016,T020,T021,T025) should reuse the same subagent/wave owner to avoid merge/conflict churn.
- US2/US3 test design can be prepared in parallel with US1 implementation planning, but RED/implementation order must respect TDD and shared-file ownership.

## MVP First

1. Phase 1 + Phase 2 RED/impl (generation + watch-no-synth contract)
2. Phase 3 US1 (first useful audio + producer offload + watch decoupled)
3. Validate host tests + targeted device TTFA smoke
4. Phase 4 US2, Phase 5 US3, Phase 6–7

## Traceability

| Spec/Contract | Tasks |
|---|---|
| FR-001/FR-005/SC-SAFE-001 | T003,T006,T011,T019,T021,T022,T028 |
| FR-002/FR-003/FR-007/FR-010/SC-TTFA-* | T004,T007,T009,T010,T012,T023,T026,T027,T028 |
| FR-004/FR-008/FR-009/FR-011/FR-012/SC-RESUME-001 | T013,T014,T015,T016,T017,T028 |
| FR-006/FR-013/SC-CANCEL-001 | T003,T005,T006,T018,T020,T021,T022,T028 |
| Contracts: producer/watch/lock/test | T003-T022,T026-T028 |
