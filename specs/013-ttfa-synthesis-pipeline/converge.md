# Converge: 013 TTFA Synthesis Pipeline Scheduling

**Date**: 2026-11-07  
**Status**: partial / in progress (autonomous implementation wave 1)  
**Decision**: `.specify/assessments/ttfa-synthesis-pipeline/decision.md` = `go` (Option A)

## Delivered artifacts
- Assess: `.specify/assessments/ttfa-synthesis-pipeline/{intake,research,problem,concept,converge-clarify,decision}.md` (go).
- Spec/plan/tasks: `specs/013-ttfa-synthesis-pipeline/{spec.md,plan.md,research.md,data-model.md,quickstart.md,tasks.md,contracts/synthesis-producer.md,checklists/requirements.md}`.

## Code changes (committed, targeted)
- `src-tauri/src/lib.rs`: `AppState`/`Production` generation+wake+producer guard; invalidation on teardown/abandon; FR-001/005-safer `play_document` startup (prepare/open backend, produce/enqueue first useful generation-safe audio, then land/publish Playing); bounded generation-safe producer helpers; `playback_watch_tick` synthesis-free (watch-safe signal/pump/background request only); `request_background_production` via `tauri::async_runtime::spawn_blocking` serialized by `producer_driving`; test-only synthesis hook seam for host TDD expansion.
- Existing dirty chapter-queue changes in `src-tauri/src/lib.rs` were preserved (not reverted).

## Host evidence (2026-11-07)
- `cargo nextest run -p src-tauri --lib playback_watch_tests ttfa_013_tests` — 8 passed.
- `cargo nextest run -p src-tauri --test test_play_to_audio_measured` — 4 passed.
- `cargo nextest run -p src-tauri --lib playback_watch_tests ttfa_013_tests --test test_play_to_audio_measured` — 8 passed (after fmt).
- `cargo fmt --all -- --check` applied/fixed for touched files.

## Device evidence (2201117TY, SDK 33, connected)
- ADB connected: `adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp` / 2201117TY.
- `diagnostic-record.jsonl` path lookup attempted; not found at checked app files path in this probe. Baseline `play_to_audio_ms` evidence remains from prior capture in handoff (1659–17309ms common; 45878–120422ms high-offset extremes). **No new TTFA improvement claim** without rebuilt install + controlled smoke + fresh `play_to_audio_ms` capture.

## Not yet complete / blocked-for-proof
- Full 013 task closure: stronger RED/GREEN for stale-commit side effects, resume/high-offset first-useful proof, cancellation during/after sync synth result, FR-001/005 failure matrix (T003 full, T007-T022).
- Producer is partially host-driver/inlined for correctness; needs dedicated robust non-watch driver/wakeup strategy validated under Android behavior and full integration.
- `cargo clippy -p mdreader` / workspace gates broader than targeted tests not fully run in this wave.
- Device TTFA claim **blocked for proof** until debug build/install + controlled smoke on 2201117TY with fresh diagnostics.
- T232/T233 chapter pre-roll/resume unchanged; not claimed complete.

## SC status (honest)
| Outcome | Status | Evidence |
|---|---|---|
| SC-TTFA-001 lower TTFA / eliminate extremes | partial, unproven on device | code path refactor; no fresh device `play_to_audio_ms` |
| SC-TTFA-002 watch not blocked by synth | partial host evidence | watch tick refactored to signal/pump/background request; structural/code evidence + existing tests pass |
| SC-RESUME-001 first useful audio for offset | partial | generation-safe bounded startup keeps trim semantics; no targeted offset benchmark proof yet |
| SC-CANCEL-001 stale work discarded | partial | generation invalidation/guards added; stronger tests pending |
| SC-SAFE-001 no false healthy Playing | partial host evidence | startup reordered, abandon invalidates/publishes; smoke test passes; matrix pending |
| SC-REG-001 regressions absent | partial | targeted tests pass; broader gates/device smoke pending |

## Next wave
1. Add/finish 013 RED tests (T007/T008/T013/T014/T018/T019) using synthesis hook + generation seams; make failures prove contracts.
2. Harden producer: no synth in `produce_bounded` when called from disallowed paths? ensure startup first-useful uses blocking-safe path explicitly; complete commit guards before every enqueue/start/publish.
3. Run `cargo clippy -p src-tauri`/relevant gates; fix 013 warnings where appropriate.
4. Build/install debug on 2201117TY, capture fresh `play_to_audio_ms` for cold open/high-offset resume/switch/startup cancel; update converge with proof or `blocked:<reason>`.


## 2026-11-07 host verification wave (autonomous)

### Implementation/test commits (targeted)
- `776c2956` test(ttfa): strengthen 013 host generation/watch contracts
- `a50303a3` fix(ttfa): clean finish_production doc comment
- `ecf73f36` test(ttfa): add 013 host stale-generation and resume-trim tests
- `7e3b63d5` test(ttfa): fix 013 host tests for stale generation and resume trim
- `7cd9e576` style(ttfa): fmt 013 test assertion
- `13651c81` test(ttfa): cover stale 013 commit side-effect discard

### Host evidence
- `cargo fmt --all` — clean.
- `cargo clippy -p src-tauri --lib -- -D warnings` — clean.
- `cargo nextest run -p src-tauri --lib ttfa_013_tests` — **6 passed**:
  - stale generation cannot target active production
  - stale generation does not commit/start production (bounded producer no-op)
  - stale `commit_produced_unit` ignores stale generation and does not mutate current production
  - watch tick/demand helper is synthesis-free contract smoke
  - abandon landing never leaves healthy `Playing` (FR-001/005 seam)
  - resume offset trims `samples_before_entry` sample-accurately (220 -> 120 useful samples)
- `cargo nextest run -p src-tauri --test test_play_to_audio_measured --test test_resume_restarts_audio` — **9 passed**.

### Honest status
- Host contract/invariant progress is stronger, but **013 is not complete**: full hook-driven first-useful/resume end-to-end proofs, complete stale side-effect audit (enqueue/start/backend handover/publish), and full FR-001/005 cancel/failure matrix remain pending.
- **TTFA proof blocked**: 2201117TY connected (`adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp`, `app.mdreader` installed) but rebuilt debug 013 binary + controlled `play_to_audio_ms` smoke/capture not executed in this wave. Do not claim `SC-TTFA-001..004` pass or T232/T233 complete.
