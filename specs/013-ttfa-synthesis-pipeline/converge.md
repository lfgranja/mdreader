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

### Device build/install (no TTFA proof yet)
- `cargo tauri android build --debug --target aarch64` completed: `src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk`.
- Installed on `adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp` / 2201117TY (2026-11-07 09:38:23, `app.mdreader` debug, v0.1.0/1000). `run-as` diagnostic tail was empty in sampled read.
- **Still blocked for proof**: controlled TTFA smoke not executed (needs reproducible StartNow/open flow, optional high-offset resume/switch/cancel/5+ min checks) to capture fresh `play_to_audio_ms` and compare to baseline. No automated UI path executed here; claiming improvement would be invalid.

## 2026-11-07 autonomous continuation (host hardening + research integration)

### Deep research (high-reliability, 2024-2026, integrated)
Subagent `Tauri013Research` (authoritative sources): Tauri v2 Calling Rust/State/async runtime docs, `tauri::async_runtime::spawn_blocking` docs.rs (tauri 2.12.x line), Tokio `spawn_blocking`/`JoinHandle::abort` (non-abortable once started), Tokio `Mutex` guidance, cpal 0.18.1 UPGRADING/`DeviceTrait` callbacks, Android AAudio guide, Tauri #14694 contention/deadlock risk signal, sherpa-onnx 1.13.8 refs from baseline.
- **Supports 013 Option A**: bounded CPU synthesis off UI/watch/cpal callback; generation/token stale-result discard at every side-effect boundary; short `std::sync::Mutex` critical sections, no guard across `.await`; FR-001/005 no healthy `Playing` before valid first-useful handoff/start; correctness-first resume (`samples_before_entry`) unless exact timeline invariants proven.
- **Risks to track**: `spawn_blocking` work cannot be aborted mid-inference (discard after return only); Android lock contention/deadlock class risk (Tauri #14694 shape) — keep locks short, avoid nested/cross-thread lock+backend/cpal interactions; cpal callback must remain allocation/block/wait free.

### Implementation hardening (targeted)
- `9a77909f` `fix(ttfa): harden 013 enqueue/start with generation checks` — replaced unguarded legacy `enqueue_chunk`/`start_backend` path with generation-checked `enqueue_chunk_for_generation`/`start_backend_for_generation`; removed unused legacy wrappers; `handover_pending_for_generation` uses checked backend side effects.

### Host evidence (fresh)
- `cargo fmt --all` clean.
- `cargo clippy -p src-tauri --lib -- -D warnings` — **clean**.
- `cargo nextest run -p src-tauri --lib ttfa_013_tests` — **7 passed** (includes hook-driven `t007_produce_bounded_commits_first_useful_audio_via_synth_hook`, stale `commit_produced_unit` discard, resume trim, abandon landing, watch/demand smoke).
- `cargo nextest run -p src-tauri --lib playback_watch_tests ttfa_013_tests --test test_resume_restarts_audio` — **12 passed** (playback watch + 013 + resume restart).
- `cargo nextest run -p src-tauri --test test_play_to_audio_measured --test test_resume_restarts_audio` — **9 passed** (previous wave, still relevant).

### SC mapping (honest, current)
- `SC-TTFA-001` **blocked/unproven**: 2201117TY debug APK installed (013 build), but no controlled fresh `play_to_audio_ms` capture/comparison yet. Baseline remains 1659–17309 ms common; 45878–120422 ms high-offset extremes.
- `SC-TTFA-002` **partial host**: hook-driven produce_bounded commits first useful unit; end-to-end startup multi-segment/full timeline proof pending.
- `SC-RESUME-001` **partial host**: sample-trim invariant proven at commit boundary; first-useful high-offset/resume end-to-end proof pending.
- `SC-CANCEL-001` **partial host**: stale generation target/commit discard and bounded no-op tests pass; full enqueue/start/backend handover/publish side-effect matrix and async completion-after-cancel proofs pending.
- `SC-SAFE-001` **partial host**: `abandon_playing_landing` seam tested; startup reordered to first-useful before `Playing` publish; full FR-001/005 cancel/failure matrix pending.
- `SC-REG-001` **partial**: targeted TTFA/watch/resume/play-to-audio regressions pass on host; broader gates/device smoke/60s pre-roll/underrun/5+ min playback not executed as full 013 proof. **T232/T233 not complete.**

### MDMV flow status
`intake/research/problem/concept/clarify/decision/spec/plan/tasks` artifacts exist for 013 (`decision=go`, Option A). Implementation/converge advanced autonomously with TDD evidence; **validate/release not performed** (R4/R7-style gates/evidence incomplete; device proof missing). No `specify.bug.*` flow opened (no separately reported bug artifact beyond 013 task scope).

## 2026-11-07 autonomous continuation 2 (host side-effect audit/tests)

### Code/test
- `abf55fc4` `test(ttfa): cover stale 013 pump demand discard` — host test proves `pump_production_demand` does not hand over stale pending audio when current generation has no production.
- `9a77909f` (amended/clean) remains the 013 generation-checked backend handover hardening.

### Fresh host evidence
- `cargo fmt --all` clean.
- `cargo clippy -p src-tauri --lib -- -D warnings` — clean.
- `cargo nextest run -p src-tauri --lib ttfa_013_tests` — **8 passed** (adds `t018_pump_production_demand_ignores_stale_generation`).
- `cargo nextest run -p src-tauri --lib playback_watch_tests ttfa_013_tests --test test_resume_restarts_audio --test test_play_to_audio_measured` — **13 passed**.

### Honest status delta
- `SC-CANCEL-001` strengthened on host: stale commit discard + stale bounded producer no-op + stale demand/handover no-op + generation-checked enqueue/start. Still missing: async synth completion returning after invalidation with real hook-driven `produce_bounded`/startup path, and exhaustive publish/`after_execute`/timeline side-effect audit.
- `SC-TTFA-001` still **blocked/unproven** (no controlled 2201117TY `play_to_audio_ms` capture). APK installed; smoke not executed.
- All other SCs remain partial/blocked as previously recorded; T232/T233 unchanged.

## 2026-11-07 autonomous continuation 3 (host TDD expansion)

### Code/test
- `31e79210` `test(ttfa): cover 013 incremental first-useful and synth errors`
  - `t007_produce_bounded_commits_first_unit_without_draining_all_segments`: hook-driven multi-segment production advances only first useful unit (`next_unit=1`, `total_samples=220`, 3 segments remain) — host evidence against unnecessary full-document loop in bounded first-useful step.
  - `t019_synthesis_hook_error_propagates_without_corrupting_production`: synth hook failure propagates; production not advanced/corrupted (`next_unit=0`, `total=0`, `pending=0`).

### Fresh host evidence
- `cargo fmt --all` clean.
- `cargo clippy -p src-tauri --lib -- -D warnings` — clean.
- `cargo nextest run -p src-tauri --lib ttfa_013_tests` — **10 passed**.
- `cargo nextest run -p src-tauri --lib playback_watch_tests ttfa_013_tests --test test_resume_restarts_audio --test test_play_to_audio_measured` — **15 passed**.

### Honest status delta
- `SC-TTFA-002` strengthened (host): bounded producer commits first useful unit without draining all segments. Still missing: end-to-end startup `play_document`/`stream_synthesis` proof across real segment list + backend/start/publish invariants.
- `SC-SAFE-001`/FR failure path strengthened (host): synthesis error propagates without corrupting production state. Still missing: full `play_document` failure matrix proving no healthy `Playing` publish and coherent abandon/refusal in all failure points.
- `SC-TTFA-001` remains **blocked/unproven** (device controlled `play_to_audio_ms` smoke required). T232/T233 unchanged.
- MDMV `validate` (R4) / `release` (R7): **not satisfied** — device proof and full SC/regression evidence incomplete.

## 2026-11-07 autonomous continuation 4 (device smoke attempt, blocked)

### Attempted
- Verified 2201117TY connected (`adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp`) and 013 debug APK installed (2026-11-07 09:38:23).
- Created `/data/local/tmp/013-ttfa-smoke.md` and triggered `app.mdreader/.DebugIngestActivity --es mode document --es path /data/local/tmp/013-ttfa-smoke.md` → `ok:document:/data/local/tmp/013-ttfa-smoke.md` (log: `debugDeliverDocument`, snapshot written). `debug_ingest_result.txt` read OK.

### Blocker (honest)
- `DebugIngestActivity`/`debugDeliverDocument` **delivered the document snapshot but did not start controlled playback**; `diagnostic-record.jsonl` tail showed no new `play_to_audio_measured`/`play_to_audio_ms` records (only prior progress records; no file growth attributable to TTFA measurement). During sampling, ADB reported device transiently unavailable (`adb ... not found` on one command; later connectivity should be rechecked before manual smoke).
- **No TTFA proof obtained.** `SC-TTFA-001` remains blocked/unproven. Controlled cold open + high-offset resume + switch/cancel + 5+ min smoke requires a playback-trigger path (manual UI or debug StartNow command not found in Kotlin shell) and fresh diagnostic capture.

### Manual unblock steps (2201117TY, 013 debug installed)
1. Reconnect/verify device: `adb devices -l` (expect 2201117TY).
2. Mark baseline: note current `diagnostic-record.jsonl` size/time.
3. Open `app.mdreader`, load a known document (or `/data/local/tmp/013-ttfa-smoke.md` if UI can open it), perform **StartNow**; wait for audible/first consumed sample.
4. Capture: `adb -s <serial> shell run-as app.mdreader cat /data/data/app.mdreader/diagnostic-record.jsonl > /tmp/mdreader-013-diag.jsonl && rg 'play_to_audio_measured|play_to_audio_ms' /tmp/mdreader-013-diag.jsonl | tail -n 20`.
5. Repeat for **high-offset resume** (document with stored high `position_secs`), **switch/cancel during startup**, and **5+ min playback**.
6. Append raw `play_to_audio_ms` values + `position_secs`/scenario labels to 013 `converge.md`/`quickstart.md`; update SC mapping and T232/T233 honestly. Do not claim improvement without this evidence.

## 2026-11-07 autonomous continuation 5 (stale synth-result discard proof)

### Code/test
- `83fa3f1e` `test(ttfa): discard stale 013 synth result after invalidation`
  - `t018_stale_synth_result_after_invalidation_is_discarded`: hook-driven test simulates cancellation/invalidation completing while sync synth returns; `produce_bounded` discards stale result (no active production for stale generation, current generation advanced, no production running).

### Fresh host evidence
- `cargo fmt --all` clean.
- `cargo clippy -p src-tauri --lib -- -D warnings` — clean.
- `cargo nextest run -p src-tauri --lib ttfa_013_tests` — **11 passed**.
- `cargo nextest run -p src-tauri --lib playback_watch_tests ttfa_013_tests --test test_resume_restarts_audio --test test_play_to_audio_measured` — **16 passed**.

### Honest status delta
- `SC-CANCEL-001` further strengthened (host): stale commit discard, stale bounded no-op, stale demand/handover discard, generation-checked enqueue/start, and stale synth-result discard after invalidation during hook-simulated sync synth. Still missing: real async return-path integration across all publish/`after_execute`/session/timeline side effects and full switch/stop/re-anchor matrix.
- Device `SC-TTFA-001` remains `blocked:device_ttfa_smoke` (manual controlled playback capture required). R4/R7 not satisfied. T232/T233 unchanged.

## 2026-11-07 autonomous continuation 7 (uncontrolled device observation, not 013 proof)

### Observation (read-only, uncontrolled, attribution uncertain)
After foregrounding `app.mdreader` and sending `input keyevent 126` (media play) on 2201117TY, diagnostics recorded a fresh session for doc `4326c1eb5d6435ab...`:
- `session_start` `ts_ms=1794090344394`, `position_secs=0.0`, `replay_secs=0.000`
- `transport_change action=play` `ts_ms=1794090344401`
- `buffer preroll_secs=43.125 preroll_cap_secs=60.000 dropped_samples=0` `ts_ms=1794090344402`
- `transport_change action=progress play_to_audio_ms=3536 position_secs=5.985759637188209` `ts_ms=1794090344777`

### Why this is NOT `SC-TTFA-001` proof
- **Attribution uncertain**: 013 debug APK was installed at 2026-11-07 09:38:23; the running `app.mdreader` process may or may not have reloaded the new native library after `adb install -r` (process restart/reload not verified). This measurement cannot be credited to 013 changes.
- **Uncontrolled scenario**: triggered by foreground+media key, not a documented controlled cold StartNow/high-offset resume/switch-cancel protocol; different document than 013 smoke fixture; preroll was 43.125s (cap 60s) — mechanism details not isolated.
- **Single sample, no baseline pairing** for this exact scenario/build.

### Status
`SC-TTFA-001` remains **blocked/unproven**. To prove 013: force-stop/relaunch `app.mdreader` after install (or confirm new process/lib), run controlled scenarios, capture labeled raw `play_to_audio_ms` rows, compare to baseline/high-offset extremes.

## 2026-11-07 user-controlled smoke evidence (provided by engineer; uncontrolled/attribution-cautious)

### Provided user scenario notes (verbatim-ish)
- Cenário A (`saf-real-play-test.md`, 473 B): ~10s TTFA; app indisponível ~15–20s (tela preta), bloqueia uso do app/celular; botões desabilitados após terminar.
- Cenário B (`relatorio_...md`, 8,60 KB): travou, bloqueou app/celular >3 min para TTFA; iniciou às 19:00; "Continuar de onde parei": ~10s TTFA; demora nas transições; botões Avançar/Retroceder/Recomeçar/2x inoperantes. Segunda execução: TTFA ~60s; "Continuar": ~60s; demora nas transições.
- Cenário C (`aula 02 modulo 01.md`, 44,64 KB): não abriu, contorno ciano ao redor da Caixa de Título; sem erro. Segunda execução: mesmo. Terceira (`relatorio...`): TTFA ~15s; não permite trocar arquivo durante pré-síntese (app indisponível ~15–20s); após liberar, trocou para `saf-real-play-test.md` (~10s); botões 100% desabilitados após reprodução.
- Cenário D (`relatorio...`, 8,60 KB): TTFA ~15s (app indisponível ~15–20s); transições ~30s, 10s, 100s, 108s; pausou no meio da 5ª seção; performance degradando; play novamente: TTA >5 min; waveform parou; degradação app/celular; force-stop.

### Diagnostic tail captured (mixed/uncontrolled, 2201117TY)
`/tmp/mdreader-013-diag.jsonl` (1794027651495..1794094943602) and `/tmp/mdreader-013-diag-CenarioA.jsonl` (1794081875586..1794091736421) show:
- repeated `start_refused cause=no_start` failures
- `device_refused cause=backend` (ts 1794082839781)
- several session starts with high `position_secs` and very high TTFA: `play_to_audio_ms=110466` (position 115.6), `120422` (position 147.1), `16815`, `17309`, `17050`, `10766`, `10717`, `10592`, `10071`, `8659`, `7468`, `7314`, `6310`, `4725`, `4589`, `4565`, `4371`, `4341`, `3862`, `3536`, `5271`
- short docs: `play_to_audio_ms=4589/1659/4565/4725` for doc `33ea974b...`
- buffer `preroll_secs` observed 3.4–43.2, `preroll_cap_secs=60.000`, `dropped_samples=0` in sampled records

### Interpretation (honest)
- This is **real user-observed degradation** (UI freeze/black screen, disabled controls, huge TTFA for high offsets, failed/cancelled starts). It corroborates baseline severity and current 013 **host-proven** generation/stale/cancellation invariants not yet reaching device playback path cleanly.
- **Still not a controlled 013 proof**: scenarios were interleaved, some before/after force-stop; 013 debug APK install time 2026-11-07 09:38:23 vs diagnostic timestamps ~18:44–19:40; process/native-lib reload not verified; no clean before/after pairing; no isolated cold StartNow/high-offset/switch-cancel/5+min protocol captured.
- **`SC-TTFA-001` remains blocked/unproven**. `SC-TTFA-002`, `SC-RESUME-001`, `SC-CANCEL-001`, `SC-SAFE-001`, `SC-REG-001` remain partial; **T232/T233 unchanged/not complete**; **R4/R7 not satisfied**.
- Additional observed defects (controls disabled, speed button no effect, switch blocked during synthesis, black screen) are likely outside pure 013 TTFA scheduling or indicate broader regressions; should be tracked separately if needed (not silently claimed as 013).

## 2026-11-07 continuation 9 (automated smoke attempt on 2201117TY)

### Automated attempt (read-only, no new 013 code changes)
- Force-stopped and relaunched `app.mdreader`; diagnostic size before ingest: **227730 bytes**.
- Triggered `app.mdreader/.DebugIngestActivity --es mode document --es path /data/local/tmp/013-ttfa-smoke.md` → `ok:document:/data/local/tmp/013-ttfa-smoke.md` (`debug_ingest_result.txt` read OK).
- Waited 12s + 20s; diagnostic size remained **227730 bytes**; tail showed only prior session (`doc=4326c1eb...`, paused at 211.02s) and **no new `session_start`/`play_to_audio_ms`/`start_refused`**.
- `monkey -p app.mdreader` injected one event but did not produce a new controlled playback session.

### Conclusion (honest)
- Automated trigger delivered the document snapshot but **did not start playback/produce TTFA measurement** on this device. `SC-TTFA-001` remains **blocked/unproven**.
- 013 debug APK install time is 2026-11-07 09:38:23; process relaunched in this attempt, but no playback occurred to attribute.
- Manual UI interaction (StartNow) or a debug playback intent is still required; no evidence fabricated.
