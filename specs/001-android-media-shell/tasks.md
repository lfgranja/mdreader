# Tasks: Android Media Shell

**Input**: Design documents from `/specs/001-android-media-shell/` (spec.md, plan.md, research.md, data-model.md, contracts/, quickstart.md, checklists/)

**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/engine-handoff.md, contracts/shell-ipc.md, contracts/storage-schema.md, contracts/diagnostic-record.schema.json

**Tests**: YES — TDD is MANDATORY per Constitution Principle III and this request. ALL test tasks are BLOCKING: no implementation task for a story may begin until every test task for that story is written AND shown to FAIL. Property-based testing (`proptest`) is used wherever an invariant exists; invariants are quoted verbatim in each test task.

**Organization**: Tasks grouped by user story. Each story phase = Tests (BLOCKING) → Implementation → Checkpoint.

**Deterministic hook**: After finishing editing ANY file, make an atomic conventional commit with a detailed summary of the edits made. (Applies to the implementation run, not to this tasks.md itself except its own creation commit.)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies). [P] tasks are delegated to the maximum viable number of agents/subagents/background agents, executed in waves; tasks sharing context reuse the same subagent.
- **[Story]**: Which user story (US1–US5).
- Every task includes an exact file path. No vague tasks.
- **BLOCKING**: Tasks marked BLOCKING gate all later tasks in their scope until complete.
- **NEVER**: No agent attests success. ONLY the final Phase verification tasks attest completeness.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Greenfield workspace scaffolding. No repo code exists yet (no `Cargo.toml`, `crates/`, `src-tauri/`, `scripts/verify.sh` — verified 2026-10-05). Create it per plan.md §Project Structure.

- [ ] T001 Create Cargo workspace root in `Cargo.toml` (members: `crates/player`, `crates/session`, `crates/diag`, `crates/engine-core`; Edition 2024, MSRV 1.85, `[lints] workspace = true`, shared `[workspace.lints]` with `unsafe_code/missing_docs/missing_debug/unreachable_pub/unused_results` deny, `clippy::all+pedantic` deny, `clippy::nursery+cargo` warn, `unwrap_used/expect_used/panic/todo/dbg/println` deny)
- [ ] T002 [P] Create `crates/player/Cargo.toml` (deps: `cpal = "=0.18.2"` exact, plus latest-stable-at-add-time `thiserror`, `tracing`, `serde`/`serde_json`, `clap` (derive) with exact `=x.y.z` as resolved that day recorded in the atomic commit msg per Principle IV; `[lints] workspace = true`)
- [ ] T003 [P] Create `crates/session/Cargo.toml` (deps: latest-stable-at-add-time `redb`, `blake3`, `thiserror`, `tracing`, `serde`/`serde_json`, `clap` (derive) with exact `=x.y.z` recorded in the atomic commit msg per Principle IV; `[lints] workspace = true`)
- [ ] T004 [P] Create `crates/diag/Cargo.toml` (deps: latest-stable-at-add-time `serde`/`serde_json`, `thiserror`, `tracing`, `clap` (derive) with exact `=x.y.z` recorded in commit msg; `[lints] workspace = true`)
- [ ] T005 [P] Create `crates/engine-core/Cargo.toml` (deps: latest-stable-at-add-time `thiserror`, `tracing`, `serde`/`serde_json`, `clap` (derive) with exact `=x.y.z` recorded in commit msg; `[lints] workspace = true`)
- [ ] T006 [P] Create `deny.toml` in repo root (allow-list: MIT, Apache-2.0+LLVM-exception, BSD-2/3, ISC, Zlib, MPL-2.0, Unicode-3.0, GPL-3.0, LGPL-3.0, AGPL-3.0, CC0-1.0, CC-BY-4.0; deny-list: `tts-rs`, `onnx`, `sherpa`, `candle`, `burn`, `tch`, `llm`, `llama-cpp-2`, `llama-rs`, `rust-bert`, `openai`, `async-openai`, `openai-api`, `langchain`, `langchain-rust`, `edge-tts-rs`; sources = crates.io only) + `.cargo/config.toml` with `--locked` enforcement note
- [ ] T007 [P] Create `clippy.toml` in repo root (document pedantic/nursery baseline; no weakenings) and add dev-dependencies `proptest` (latest stable at add-time, exact `=x.y.z` recorded in commit msg) + `cargo-nextest` config in `.config/nextest.toml`
- [ ] T008 Create `scripts/verify.sh` skeleton in `scripts/verify.sh` (gates 1–9 stubs with `set -euo pipefail`, fail-closed on missing input; full gate bodies filled in Phase 8)
- [ ] T009 BLOCKING GATE — Setup complete: `cargo build --workspace --locked` succeeds on host Linux AND `cargo fmt --all -- --check` passes. No Phase 2 work begins until this is green.

---

## Phase 2: Foundational (BLOCKING Prerequisites)

**Purpose**: Closed-set vocabulary + test harness that EVERY user story depends on. No story work begins until this phase is complete.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete. Test-infrastructure tasks T010–T013 are BLOCKING for all story test phases.

### Foundational tests (write FIRST, show FAIL)

- [ ] T010 [P] Write failing property-test harness for closed-set exhaustiveness in `crates/engine-core/tests/prop_closed_sets.rs` (proptest per FR-044: every closed enum — rate rungs `[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]`, transport `Playing|Paused|Stopped|Completed|Interrupted`, focus outcomes `DuckableTransient|SilenceDemanding|PermanentLoss` + `regained` direction event, `EngineError Unreachable|InitFailed|RateMismatch|ChannelMismatch|DeviceRefused|DeviceLost|StartRefused`, session-end `Paused|Stopped|Completed|Interrupted` — has exactly the specified variants, no `Other/Unknown`, and `serde` round-trips every variant; FAILS before types exist)
- [ ] T011 [P] Write failing JSON-schema conformance test for diagnostic record in `crates/diag/tests/contract_schema.rs` (validates sample events against `specs/001-android-media-shell/contracts/diagnostic-record.schema.json` per FR-039, FR-044, FR-045: required `v/ts_ms/kind/position_secs`, `detail` bound `maxLength 200` + pattern `^[A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+( [A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+)*$`, `error` required on `failure` AND `refusal`; `session_end` requires `transport`∈closed-4 + `session_end_reason`∈closed-4 (never `playing` on `session_end`); superseding entry requires `supersedes_ts_ms`; FAILS before writer exists)
- [ ] T012 [P] Write failing property test for blake3 fingerprint stability in `crates/session/tests/prop_fingerprint.rs` (proptest invariants: same bytes → same 32-byte hash regardless of path; 1-bit content change → different hash; identical bytes at two paths share position; FAILS before `session` crate exists)
- [ ] T013 [P] Write failing shell-purity + manifest-check script skeletons in `scripts/check-shell-purity.sh` and `scripts/check-manifest.sh` (purity: grep `android-shell/` + `src-tauri/` for `transport|PlaybackState|playbackRate|seekTo|position_secs|gain|duck|skip` state mutations outside allowlist → fail if found (allowlist per T081: 5 Kotlin mirrors + 4 src-tauri shape/delegation files, shapes/getters only never computation); manifest: assert `<service android:foregroundServiceType="mediaPlayback">` + `FOREGROUND_SERVICE` + `FOREGROUND_SERVICE_MEDIA_PLAYBACK` present in built manifest; FAILS on empty tree as missing-input-fail-closed)

### Foundational implementation (only after T010–T013 FAIL)

- [ ] T014 Define closed-set vocabulary crate in `crates/engine-core/src/closed.rs` (enums `PlaybackRate` 7 rungs, `TransportState`, `FocusOutcome`, `EngineError`, `SessionEnd` with `Debug+Clone+Copy+PartialEq+Eq+serde`, `missing_docs` satisfied, NO `_ =>` arms anywhere, NO `Other/Unknown`; engine IDs referenced from constitution registry by name+version only)
- [ ] T015 [P] Implement `DeclaredFormat` constants in `crates/engine-core/src/format.rs` (`SAMPLE_RATE: u32 = 22050`, mono-f32 shape descriptor, `check_handoff(rate, channels, width) -> Result<(), EngineError>` returning `RateMismatch|ChannelMismatch`, never adapting silently; FR-033, FR-035)
- [ ] T016 [P] Implement engine trait + capability declaration in `crates/engine-core/src/trait_.rs` (`declared_capabilities() -> (SsmlLevel, AlignmentGranularity Sentence|Word|None)`, `set_rate()` pitch-preserving contract doc, `stream()` pre-roll source ≤60 s bound, failures as closed `EngineError`; per contracts/engine-handoff.md)
- [ ] T017 [P] Implement `engine-core` CLI in `crates/engine-core/src/cli.rs` + `crates/engine-core/src/main.rs` (`clap` derive, stdin/args→stdout, errors→stderr, `--format json`; commands mirror trait ops for headless driving)
- [ ] T077 [P] Implement stand-in engines in `crates/engine-core/src/standin_a.rs` and `crates/engine-core/src/standin_b.rs` (both honour handoff contract: mono f32 @22050 Hz, declared granularities, injectable mid-document failure for SC-051 double-failure drill; NO fallback/selection logic here — that stays in T075/T076. Built in Foundation so US1 T022 + US4 T055 have fixtures; required for gates T031/T066)
- [ ] T018 BLOCKING GATE — Foundation green: `cargo nextest run --workspace` passes T010–T012 (now implemented), `cargo doc --workspace --no-deps` clean, gate-7 scan (`no _ => in crates/*/src`) passes, stand-ins T077 conform to contracts/engine-handoff.md shape invariant. No story phase begins until this is green.

**Checkpoint**: Foundation ready — closed vocabulary, format gate, trait, schema test, purity/manifest scripts, plus conforming stand-in engines (T077) for US1/US4 fixtures exist. Story test phases can now begin in parallel waves.

---

## Phase 3: User Story 1 — Keep reading with the phone locked (Priority: P1) 🎯 MVP

**Goal**: Playback continues with screen off / locked (FR-001); Rust `player` crate is single authority for position/buffering/transport (FR-002, no second source of truth FR-003); never report playing while silent (FR-005).

**Independent Test**: Start playback, lock device, wait 5 min → audio continuous, media controls reachable (SC-001); zero `transport:"playing"` entries without strictly-greater `position_secs` follow-up per SC-005 record scan.

### Tests for US1 (BLOCKING — write ALL first, FAIL before T024+) ⚠️

- [ ] T019 [P] [US1] Property test: position monotonicity + SC-005 record rule in `crates/player/tests/prop_position_authority.rs` (proptest invariants: `position_secs: f64 ≥ 0` never decreases during `Playing`; every `transport:"playing"` record entry is followed same-session by strictly greater `position_secs` unless intervened by `completed|stopped|paused|device-loss|failure`; pre-roll buffering NEVER advances position; FAIL before implementation)
- [ ] T020 [P] [US1] Property test: session lifecycle transitions in `crates/player/tests/prop_transport.rs` (proptest over closed `TransportState`: allowed `Playing→Paused|Interrupted|Stopped|Completed`, `Paused→Playing`, `Interrupted→Playing` only via FR-023a, FR-032c expiries or explicit action, any→`Stopped`; NEVER `Interrupted→Playing` unattended; NEVER playing-while-silent; FAIL before implementation)
- [ ] T021 [P] [US1] Integration test: kill-recovery bound in `tests/integration/test_session_recovery_us1.rs` (kill mid-playback → relaunch offers resume at last spoken position replaying ≤30 s per FR-004, SC-007; unexpected end → paused + explicit resume per FR-030, never auto-audio per FR-031, SC-011; FAIL before implementation)
- [ ] T022 [P] [US1] Contract test: engine-handoff shape invariance in `tests/contract/test_engine_handoff_us1.rs` (stand-in engines deliver mono f32 @22050 Hz on all paths; rate/channel/width deviation → named `EngineError`, never silent resample per FR-033, FR-035, SC-034; FAIL before implementation)
- [ ] T022b [P] [US1] Integration test: externally-started-service trial in `tests/integration/test_external_start_us1.rs` (start service outside app process → Play from lock surface produces audio without app open per FR-011; platform-refused cold start → stated reason + no playing state per FR-010; FAIL before implementation)
- [ ] T023 BLOCKING GATE [US1] — All US1 tests T019–T022b written and FAILING (`cargo nextest run -p player` + `tests/` show red). Implementation T024–T031 MUST NOT begin until this gate is attested.

### Implementation for US1

- [ ] T024 [P] [US1] Implement transport state machine in `crates/player/src/transport.rs` (closed `TransportState`, transition function per data-model.md PlaybackSession, `#[expect]`-only suppressions, `thiserror`, `tracing`, full rustdocs)
- [ ] T025 [P] [US1] Implement authoritative position clock in `crates/player/src/position.rs` (`position_secs: f64 ≥ 0`, advances only on audible playback, pre-roll never advances it, kill-replay ≤30 s basis per FR-004)
- [ ] T026 [US1] Implement cpal playback backend in `crates/player/src/backend.rs` (cpal 0.18.2 AAudio, exact-request 22050 Hz / PCM-float / mono, device-format conversion in Rust, refusal → named `EngineError::DeviceRefused` per FR-034, FR-035, R4; depends on T024, T025)
- [ ] T027 [P] [US1] Implement `player` CLI in `crates/player/src/cli.rs` + `crates/player/src/main.rs` (`play|pause|stop|skip-forward|skip-backward|rate-up|rate-down|voice-next|voice-prev|resume-at-stored|start-over|choose-engine <id>` verbs, plus read-only queries `position|state`, `--format json` (default for scripting), each verb calls the identical backend function as the same-named IPC verb per contracts/shell-ipc.md (`position|state` are queries mapping to no IPC verb and excluded from closed-verb exhaustiveness; no `rate-set` — rate moves only one-step saturating per FR-019); `skip-*` = exactly ±5.0 s clamped per FR-018, `rate-*` = one-step saturating ladder moves per FR-019, `voice-*` = one-voice step with FR-038 re-anchor per FR-018b; same code path as IPC per Principle II)
- [ ] T028 [US1] Pin + implement Tauri bridge: `src-tauri/Cargo.toml` pins `tauri = "=2.12.1"` (stable 2.x per Principle IV; 3.0.0-alpha rejected — version+reason recorded in commit msg) and `src-tauri/src/lib.rs` implements plugin registration (`register_android_plugin` + `run_mobile_plugin`, Activity-constructor `(Landroid/app/Activity;)V`, `tauri::ipc::Channel` streaming; NO playback logic, pure delegation per FR-042; owns externally-started-service path per FR-011 — plugin registration usable without prior app-process start, asserted by T022b external-start trial)
- [ ] T029 [US1] Implement Kotlin resident `Service` skeleton in `android-shell/src/main/kotlin/app/mdreader/PlaybackService.kt` (foreground `mediaPlayback` service, self-bootstrap from Application/Activity, API-34 `ForegroundServiceStartNotAllowedException` → stated-reason path per FR-010; mirror ONLY, zero transport/rate/position logic)
- [ ] T030 [US1] Declare foreground service in `src-tauri/gen/android/app/src/main/AndroidManifest.xml` per FR-006, FR-007 (`<service android:foregroundServiceType="mediaPlayback">` + `FOREGROUND_SERVICE`, `FOREGROUND_SERVICE_MEDIA_PLAYBACK`, `POST_NOTIFICATIONS` API-33+; wired to `scripts/check-manifest.sh` gate SC-006)
- [ ] T030b [P] [US1] Implement permission-state gate in `src-tauri/src/permissions.rs` (startup + pre-play check: FGS permissions present per manifest else stated-reason + no playing state per FR-008, FR-010; API-33+ notification-denied routes to T065 in-app statement per FR-021; pure delegation check, no playback logic; wired to `scripts/check-manifest.sh`)
- [ ] T031 [US1] US1 GREEN gate: T019–T022b now pass (`cargo nextest run -p player`, `tests/integration/test_session_recovery_us1.rs`, `tests/integration/test_external_start_us1.rs`, `tests/contract/test_engine_handoff_us1.rs` green) + T030b permission-state gate implemented with no `_ =>` added and no lint weakening.

**Checkpoint**: US1 independently functional — screen-off playback with single Rust authority; the MVP slice.

---

## Phase 4: User Story 2 — Hand the phone back to the system (Priority: P1)

**Goal**: Classify focus loss by interrupter request: `MAY_DUCK` → duck to 15–25% gain and keep reading (FR-013); demand-silence call/alarm → pause + permanent-loss reason (FR-013a); permanent loss → pause, never auto-resume, latest-event-wins supersede (FR-014); deferred/refused focus answers handled explicitly (FR-013b); transient resume rewinds `min(5%×paused, 30 s)` + sentence snap hard-capped at 30 s (FR-015).

**Independent Test**: Trigger transient + permanent interruptions separately → volume/transport responses differ correctly (SC-003 gain 15–25% + SC-041 silence-pause + SC-055 deferred/refused).

### Tests for US2 (BLOCKING — write ALL first, FAIL before T037+) ⚠️

- [ ] T032 [P] [US2] Property test: rewind/snap/cap math in `crates/player/tests/prop_rewind.rs` (proptest invariants verbatim from FR-015: cursor = `min(0.05 × paused_duration_secs, 30.0)`; audio resumes at containing-sentence start; TOTAL backward movement ≤30.0 s even when sentence longer than cap (land at exactly cap point); cap-point mid-word snaps forward to next inter-word gap unless gap beyond cap (land exactly at cap); no-sentence-granularity degrades to raw cursor, never document start; resume lands in gap never energy peak on synthetic audio; superseded-transient cumulative clock from first yield; FAIL before implementation)
- [ ] T033 [P] [US2] Property test: focus classification + duck band in `crates/player/tests/prop_focus.rs` (proptest: `DuckableTransient` → gain factor ∈ [0.15, 0.25] AND still `Playing`; `SilenceDemanding|PermanentLoss` → `Paused/Interrupted`, gain restored, never auto-resume; supersede transient→permanent clears resume/duck-restore flags, late transient gain authorizes nothing; deferred grant → zero audio before grant, no rewind, no FR-041 reason; refusal → named failure, no playing state; FAIL before implementation)
- [ ] T034 [P] [US2] Integration test: duck-vs-pause trials in `tests/integration/test_focus_duck_pause_us2.rs` (navigation prompt → ~20% then restore + sentence-snap resume; call/alarm → pause + `"Leitura pausada — outro app usando o áudio"`; permanent → pause visible locked; FR-041 3 s window restarts at permanent loss, expires everywhere after 3 s; FAIL before implementation)
- [ ] T035 [P] [US2] Contract test: focus-intent boundary in `tests/contract/test_focus_intents_us2.rs` (Kotlin→Rust intents `DuckableTransient|SilenceDemanding|PermanentLoss|FocusRegained|BecomingNoisy|DeviceAppeared|DeviceDisappeared` per contracts/shell-ipc.md each map to exactly one FR-013, FR-013a, FR-014, FR-015, FR-016, FR-017a outcome; supersede recorded as two ordered entries with supersede link, never merged rewrite; FAIL before implementation)
- [ ] T036 BLOCKING GATE [US2] — All US2 tests T032–T035 written and FAILING. Implementation T037–T041 MUST NOT begin until attested.

### Implementation for US2

- [ ] T037 [P] [US2] Implement rewind/snap engine in `crates/player/src/rewind.rs` (FR-015 formula + snap + hard cap + cumulative supersede clock reading `first_yield_ms` (never reset at supersede; record carries `supersedes_ts_ms` link); depends on NOTHING except closed vocabulary — parallel-safe after gate T036)
- [ ] T038 [P] [US2] Implement focus classifier in `crates/player/src/focus.rs` (yield per FR-012; closed outcomes, 15–25% gain factor per FR-013, `AUDIOFOCUS_GAIN` + `USAGE_MEDIA` + `CONTENT_TYPE_SPEECH` + `setWillPauseWhenDucked(true)` + `setAcceptsDelayedFocusGain(true)` assumptions, FR-013b deferred/refused arms, FR-014 latest-event-wins)
- [ ] T039 [P] [US2] Implement pre-roll ring buffer in `crates/player/src/buffer.rs` (≤60 s engine-delivered audio, never advances position, empty while idle, refill on Play, abandon at end per FR-015a, FR-015b, FR-015c; rate change re-prepares without shifting position)
- [ ] T040 [US2] Implement Kotlin focus mirror in `android-shell/src/main/kotlin/app/mdreader/FocusMirror.kt` (`AudioFocusRequest.Builder(AUDIOFOCUS_GAIN)`, `OnAudioFocusChangeListener`, forwards classified intents to Rust, applies NOTHING locally; SC-035 allowlist entry only; depends on T038 contract)
- [ ] T041 [US2] US2 GREEN gate: T032–T035 now pass with `cargo nextest run -p player` + integration/contract suites green.

**Checkpoint**: US1 + US2 both independently functional; duck-vs-pause differs observably.

---

## Phase 5: User Story 3 — Unplug headphones, do not disturb the cabin (Priority: P1)

**Goal**: Removal pauses ≤1 s via EITHER signal alone (FR-016, FR-016a); reconnect + explicit resume plays via that device (FR-017); new-device-appears follows switch at unchanged position, never private→speaker (FR-017a); disappearance+appearance coincidence pauses first (privacy precedence).

**Independent Test**: Headphones disconnect → pause ≤1 s, never on speaker; Bluetooth-only-signal loss also ≤1 s (SC-002); new device → continue unchanged (SC-033); coincide → pause-first (SC-033b).

### Tests for US3 (BLOCKING — write ALL first, FAIL before T046+) ⚠️

- [ ] T042 [P] [US3] Property test: routing direction invariants in `crates/player/tests/prop_routing.rs` (proptest: `DeviceDisappeared` → `Paused` with position preserved, ≤1 s bound recorded either signal alone sufficient; `DeviceAppeared` while playing → same `position_secs`, no pause/restart/repeat; private→public move ALWAYS pauses first even with simultaneous appearance; each switch emits `output_device` record event; FAIL before implementation)
- [ ] T043 [P] [US3] Integration test: noisy + device-callback trials in `tests/integration/test_routing_us3.rs` (wired pull via `BECOMING_NOISY` ≤1 s; BT loss via `onAudioDevicesRemoved`-only ≤1 s; reconnect + explicit resume via reconnected device; coincide unplug-wired/connect-BT pauses first; FAIL before implementation)
- [ ] T044 [P] [US3] Contract test: removal-signal independence in `tests/contract/test_removal_signals_us3.rs` (each of the two FR-016a mechanisms reaches pause ≤1 s independently; neither waits for the other; no removal passes unhandled when only one fires; FAIL before implementation)
- [ ] T045 BLOCKING GATE [US3] — All US3 tests T042–T044 written and FAILING. Implementation T046–T048 MUST NOT begin until attested.

### Implementation for US3

- [ ] T046 [P] [US3] Implement routing state machine in `crates/player/src/routing.rs` (disappear→pause+preserve, appear→follow-unchanged, private→public pause-first precedence, `DeviceAppeared|DeviceDisappeared|BecomingNoisy` intents per contracts/shell-ipc.md; FR-016, FR-016a, FR-017, FR-017a)
- [ ] T047 [US3] Implement Kotlin routing mirror in `android-shell/src/main/kotlin/app/mdreader/RoutingMirror.kt` (`ACTION_AUDIO_BECOMING_NOISY` receiver + `AudioDeviceCallback.onAudioDevicesRemoved`, each independently forwarded to Rust, no local pause decision; depends on T046 intent shapes)
- [ ] T048 [US3] US3 GREEN gate: T042–T044 now pass.

**Checkpoint**: US1–US3 functional — the full P1 driving-safety core.

---

## Phase 6: User Story 4 — Stay in control from the lock screen (Priority: P2)

**Goal**: Lock-screen transport: ±5 s skips clamped (FR-018), one-step voice cycling with re-anchor (FR-018b, FR-038), 7-rung ladder one-step saturating + pitch-preserving engine stretch + app-wide rate memory (FR-019, FR-019a, FR-019b, FR-019c, FR-043), Pause≠Stop storage (FR-032, FR-032a, FR-032b, FR-032c, FR-032d, FR-032e, FR-032f), blake3 resume offers (FR-036, FR-036a, FR-038), notification title+section+progress only (FR-028, FR-028a, FR-028b, FR-028c, FR-028d, FR-028e), non-dismissable-while-playing (FR-028e), resident-service lifecycle (FR-009, FR-009a, FR-010).

**Independent Test**: Locked, exercise each control → correct effect (SC-004, SC-008, SC-016, SC-026, SC-036, SC-020, SC-021, SC-030, SC-042, SC-047, SC-053, SC-043, SC-049, SC-045).

### Tests for US4 (BLOCKING — write ALL first, FAIL before T057+) ⚠️

- [ ] T049 [P] [US4] Property test: skip + ladder invariants in `crates/player/tests/prop_transport_math.rs` (proptest: skip = exactly ±5.0 s vs press-time position, clamp at `[0.0, duration]`, forward-to-end → paused end-state; ladder closed 7 rungs, one step per press, saturate both ends never wrap/fail; rate change effective ≤2 s; position unchanged by rate change; pre-roll re-prepared at new rate; FAIL before implementation)
- [ ] T050 [P] [US4] Property test: fingerprint + re-anchor invariants in `crates/session/tests/prop_resume.rs` (proptest: move/rename → same offer; edited-bytes while idle → NO offer + start `0.0` + old entry discarded; edit detected while a session is active → paused state + explicit resume from `0.0` of the changed content, no approximate match (FR-036a); Stop|Completed → entry deleted, reopen starts `0.0` no offer; Pause|close|kill|power|battery|network → entry preserved; voice/engine change → nearest sentence/node boundary else `0.0`, boundary reported; `last_rate` ∈ ladder default `1.0`, survives close, crosses sessions+documents; FAIL before implementation)
- [ ] T051 [P] [US4] Property test: notification-surface + determinacy invariants in `crates/player/tests/prop_surface.rs` (proptest: surface = title + section + progress + controls ONLY — never body/elapsed/remaining/rate/voice; section = most-recent-passed else absent (never forward-looking, never body); determinacy fixed at session start, never switches mid-session, never estimated; title from metadata else filename-stable-placeholder, never first sentence; non-dismissable while playing, transient 3 s auto-dismiss vs resident becomes-dismissable; headings supplied via T061b stub; FAIL before implementation)
- [ ] T052 [P] [US4] Property test: resume-offer state machine in `crates/session/tests/prop_offer.rs` (proptest: deliberate-pause → 3 s offer continue-default + start-over, expiry auto-starts ONLY if foreground else converts to explicit-resume; unexpected-end/power/battery/network → paused + explicit resume never auto-audio; start-over → `0.0` + discard; completion → teardown + no offer + no residency; offer window never outlives session; FAIL before implementation)
- [ ] T053 [P] [US4] Integration test: lock-screen + residency trials in `tests/integration/test_lockscreen_us4.rs` (blind-test protocol per SC-008 — tester cannot see surface, trial fails on any unlock or any effect left unconfirmed aloud: play, pause, resume, skip ±, rate change, voice-cycle next/prev (one-voice step, re-anchored per FR-038), each confirmed audibly not visually per FR-018a, FR-018b; SC-004 functional correctness on same run; ±5 s clamp; ladder incl. 2.0x→4.0x; Stop-discards vs Pause-preserves; completion tears down residency SC-045, SC-049; one resident + one notification while resumable SC-043; cold-start refusal stated + no playing state FR-010, SC-014; denied permission stated in-app FR-021, SC-028; regression check only: lock-screen controls remain functional on resident service (FR-011 proof owned by T022b, refs T028); FGS-absent trial per FR-008 — permission missing → stated reason + no playing state (refs T030b); airplane-mode trial per FR-026 — start + 60 s continue with no network on parsed document; status-view assertions: rate shown in-app equals lock-screen rate source (SC-027); denied-permission statement visible (SC-028); FAIL before implementation)
- [ ] T054 [P] [US4] Contract test: shell-IPC verb/state mirror in `tests/contract/test_shell_ipc_us4.rs` (closed verbs `Play|Pause|Stop|SkipForward5s|SkipBack5s|RateUp|RateDown|CycleVoiceNext|CycleVoicePrev|ResumeAtStored|StartOver|ChooseEngine` each honoured-or-rejected never phantom-playing; `CycleVoice*` asserts one-voice step + FR-038 re-anchor + audible confirmation per FR-018a, FR-018b; Rust→shell mirror carries `transport|position_secs|section_title|progress|transient_reason?`; rapid presses each resolved; per contracts/shell-ipc.md; CLI parity: every IPC verb has a CLI equivalent resolving through the same function (FR-042, Principle II), asserted by invoking both drivers; mirror never shows playing while backend is paused per FR-020 (assert on every verb response); FAIL before implementation)
- [ ] T055 [P] [US4] Property test: pitch invariance in `crates/engine-core/tests/prop_pitch.rs` (F0 contour + spectral peaks of identical PCM across all 7 rungs match within ±50 cents per SC-015 incl. 2.0x→4.0x; delivered rate invariant 22050 Hz across rungs per SC-034; FAIL before implementation)
- [ ] T061b [P] [US4] Implement document-structure stub in `tests/fixtures/doc_structure.rs` (test-wave fixture, parallel with T049–T055: ordered headings `{position_secs: f64, title: String}` + `headings_before(pos)` helper; consumed by T051 prop_surface and T061 resolver; asserts FR-028d re-resolve triggers boundary-cross/resume/skip/re-anchor, fingerprint re-check on resume, and no polling/watcher between events; real parser replaces the stub without changing T051/T061 assertions)
- [ ] T056 BLOCKING GATE [US4] — All US4 tests T049–T055 + fixture T061b written and FAILING. Implementation T057–T066 MUST NOT begin until attested (T061b already done in test wave).

### Implementation for US4

- [ ] T057 [P] [US4] Implement skip + rate ladder in `crates/player/src/rate.rs` (closed ladder const, one-step saturating moves, ±5 s clamped skips, ≤2 s effect path invoking engine stretch op never local stretch per FR-043; pitch invariant; audible confirmation as a research-derived refinement of FR-018a per R6 (short earcon on every press + spoken state on resume/rate-change) through the same Rust path the mirror reports; acceptance is earcon-or-speech per T053 blind trials, not earcon-mandatory)
- [ ] T058 [P] [US4] Implement bookmark store in `crates/session/src/store.rs` (redb `bookmarks` blake3→`{offset_secs: f64, first_yield_ms: u64, sentence_idx: u32, end_kind: SessionEnd}` + `settings.last_rate`; `first_yield_ms` = continuous non-advancing interval start per FR-015 supersede clock, never reset at supersede; `sentence_idx` = snap hint only, node re-anchor resolves via document structure at resume per contracts/storage-schema.md; persist `end_kind` (`paused|interrupted` retained) on every write, delete ONLY on Stop|Completed, lookup→offer-flow resolved SOLELY from stored `end_kind` per contracts/storage-schema.md)
- [ ] T059 [P] [US4] Implement fingerprint + re-anchor in `crates/session/src/fingerprint.rs` (blake3-256 content hash, move/rename-stable, edit→different-doc no-offer discard, cross-voice nearest-boundary else `0.0` + report per FR-036, FR-036a, FR-038)
- [ ] T060 [P] [US4] Implement resume-offer machine in `crates/session/src/offer.rs` (offer form resolved SOLELY from stored `end_kind` — deliberate 3 s auto-start-if-foreground on `paused` vs unexpected explicit-resume on `interrupted`, start-over→`0.0`+discard, completion mutual-exclusion vs open offers per FR-032c, FR-032d, FR-032b, FR-032e, FR-032f)
- [ ] T061 [P] [US4] Implement notification content resolver in `crates/player/src/surface.rs` (title+section+progress only, FR-028, FR-028a, FR-028b, FR-028c, FR-028d re-resolve triggers boundary-cross/resume/skip/re-anchor with fingerprint check, FR-029 metadata-else-filename, FR-028e ongoing/dismissal split; headings supplied via T061b stub)
> Note: the doc-structure fixture is built in the test wave as T061b above; T061 consumes it. A real parser replaces the stub without changing T051/T061 assertions.
- [ ] T062 [US4] Implement Kotlin `MediaSession` mirror + notification in `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` (`setPlaybackState` reflects Rust state never computes position per FR-003, `MediaStyle` title+section+progress, `setOngoing(true)` while playing, transient 3 s dismiss vs resident dismissable; depends on T061 shapes)
- [ ] T063 [P] [US4] Implement `session` CLI in `crates/session/src/cli.rs` + `crates/session/src/main.rs` (`get|put|clear <file>`, `rate get|set <v>`, `--format json` per contracts/storage-schema.md)
- [ ] T064 [US4] Implement resident lifecycle + teardown in `android-shell/src/main/kotlin/app/mdreader/Residency.kt` (exactly-one resident + one notification while resumable, resident-stop control preserving bookmark vs transport-Stop discarding, teardown at document end per FR-009, FR-009a, FR-032f, swipe-away-while-playing pauses first; SC-043 asserts liveness (Play press produces audio), SC-049 asserts count/teardown — T083 records separate verdicts; depends on T058–T060)
- [ ] T065 [US4] Implement reason-string shapes + in-app reason/permission surfaces in `src-tauri/src/surface.rs` (defines shared shapes `ShortReason`/`FullReason` + `ReasonExpiry::{Focus3s, FailurePersistent}` + getters `short_reason()`/`full_reason()`; focus-loss reason 3 s expiry both surfaces FR-041, FR-037a vs failure persistence FR-037; denied-permission statement FR-021; lock-screen short + full in-app strings, document-title-only; T065 OWNS the shapes — T065b renders through them, T078 conforms to them)
- [ ] T065b [US4] Implement minimal in-app status view in `src-tauri/src/status_view.rs` + frontend binding (read-only: shows `last_rate`, transport/position mirror, current failure/focus reason with FR-037 indefinite vs FR-041 3 s expiry; zero playback logic; same Rust getters as CLI/IPC per Principle II; rendered strings come ONLY from T065 getters; asserted by T053 status-view assertions; depends on T065 ONLY — explicitly NOT on T078. Frontend binding pins `svelte` 5.x exact in package lock, version recorded in that task's atomic commit msg)
- [ ] T066 [US4] US4 GREEN gate: T049–T055 now pass.

**Checkpoint**: US1–US4 functional — full lock-screen control with fingerprinted resume.

---

## Phase 7: User Story 5 — Be told when the system cannot play audio at all (Priority: P3)

**Goal**: Named failures everywhere (FR-022, FR-023, FR-035), 3 s engine offer + consent-bounded declared fallback order + auto-takeover from last spoken position (FR-023a, FR-023c), two-stand-in failover mechanism SC-051 (SC-044 hardware re-verified later), diagnostic record 30-day/10 MB oldest-first + gap/trim markers + private storage + never transmitted + no surface + no document text (FR-039, FR-039a, FR-039b, FR-040, FR-044, FR-045).

**Independent Test**: Force device failure → clear message, never phantom playing (SC-023); kill stand-in A → B takes over from last position unattended after 3 s; force B → paused+reason (SC-051); record trim visible, zero egress (SC-025); scanner zero prose (SC-056); zero surface (SC-037).

### Tests for US5 (BLOCKING — write ALL first, FAIL before T073+) ⚠️

- [ ] T067 [P] [US5] Property test: diagnostic bounds + trim/gap in `crates/diag/tests/prop_record.rs` (proptest invariants: 30-day window + 10 MB cap oldest-first; cap-reached → trim event `{dropped_events ≥ 1, cutoff_ts_ms}` never silent absence; storage-full → gap marker via reserved capacity, playback uninterrupted; trim/gap markers flushed before any trim of retained entries; every transport change carries `position_secs`; session-end carries `transport`+`session_end_reason` from shared closed-4; superseding permanent-loss entry carries `supersedes_ts_ms`; FAIL before implementation)
- [ ] T067b [P] [US5] Property test: idle-drain + zero-billing in `crates/player/tests/prop_idle_drain.rs` (proptest invariants verbatim from FR-015a, FR-015c, SC-050, SC-050b: after entering Paused-idle, engine-spy records zero synthesised characters/bytes for simulated idle ≥1 h AND pre-roll buffer is empty throughout; Play press after idle begins pre-roll and produces audible audio within the FR-027 bound; discarded pre-roll bills ≤ smallest documented per-request byte budget on a per-character engine-spy; FAIL before implementation)
- [ ] T068 [P] [US5] Property test: no-prose scanner in `crates/diag/tests/prop_no_prose.rs` (proptest + scanner per FR-045, SC-056: zero entries contain document text; `detail` absent-or-`≤200 chars` matching non-prose pattern where retained; EVERY `failure` AND `refusal` carries `error`; SC-005 playing-advance rule scan; zero `transport:"playing"` without greater-`position_secs` follow-up; FAIL before implementation)
- [ ] T069 [P] [US5] Property test: fallback-order + consent in `crates/engine-core/tests/prop_fallback.rs` (proptest: order = remaining same-tier → higher tiers registry order → Edge last never default; descent only behind fresh 3 s offer naming lower tier; unconsented engine NEVER selected; offer names exactly selectable engines; expired fallback continues from last spoken position never restart; no-alternative expires to paused-with-reason; repeated failure re-offers remaining + each recorded per FR-023, FR-023c; FAIL before implementation)
- [ ] T070 [P] [US5] Integration test: failover + recovery paths in `tests/integration/test_failover_us5.rs` (stand-in A fail mid-doc → named failure + offer → B takes over locked without unlock, stated failure+takeover; B forced down → paused+reason; single-engine-equivalent → paused+reason never silence; service-kill → paused-at-position no auto-audio; deliberate vs unexpected reopen paths SC-020 (assert offer-to-audio ≤4 s foreground per SC-020 timing, silent background) vs SC-021; power-loss replay ≤30 s; FAIL before implementation)
- [ ] T071 [P] [US5] Contract test: stand-in conformance in `tests/contract/test_standin_conformance_us5.rs` (stand-ins A offline-like + B cloud-like pass handoff-conformance: shape invariant all 7 rungs, granularity honestly declared `Sentence|Word|None`, failure taxonomy exact per contracts/engine-handoff.md; FAIL before implementation)
- [ ] T072 BLOCKING GATE [US5] — All US5 tests T067, T067b–T071 written and FAILING. Implementation T073–T079 MUST NOT begin until attested.

### Implementation for US5

- [ ] T073 [P] [US5] Implement diagnostic writer + trim in `crates/diag/src/writer.rs` (append-only JSONL app-private storage, dual 30-day/10 MB oldest-first bounds, trim/gap events with reserved marker capacity, best-effort writes never interrupting playback per FR-039, FR-039b)
- [ ] T074 [P] [US5] Implement diagnostic reader CLI + scanner in `crates/diag/src/cli.rs` + `crates/diag/src/main.rs` + `crates/diag/src/scan.rs` (tooling-readable read, SC-005, SC-056 scanners, NO in-app surface per FR-039a, never transmit per FR-040)
- [ ] T075 [P] [US5] Implement fallback selector in `crates/engine-core/src/fallback.rs` (closed declared order, consent read per attempt from app-private storage, Edge-last-never-default, fresh-offer descent rule, supersede-aware per FR-023, FR-023c)
- [ ] T076 [P] [US5] Implement engine offer machine in `crates/player/src/recovery.rs` (named failure + alternatives, 3 s window, choice-honoured else auto-fallback from last spoken position, no-alternative → paused-with-reason, re-offer on repeated failure, locked-resolution without unlock per FR-023, FR-023a, SC-039, SC-040)
> Note: stand-in engines are built in Foundation as T077; US5 uses them in T070/T071 and builds nothing here.
- [ ] T078 [US5] Implement failure reason plumbing in `src-tauri/src/failure.rs` (short lock-screen + full in-app strings conforming to T065 `ShortReason`/`FullReason` shapes, document-title-only no body, failure persists indefinitely vs focus-reason 3 s expiry per FR-037, FR-037a; depends on T065 shapes + T076)
- [ ] T079 [US5] US5 GREEN gate: T067, T067b–T071 now pass.

**Checkpoint**: All five stories functional. SC-044 (two REAL engines) explicitly NOT discharged — carried forward as hardware obligation per FR-023b.

---

## Phase 8: Polish, Gates & Verification (ONLY this phase attests success)

**Purpose**: Cross-cutting hardening + the deterministic gates. NO agent statement attests completeness — ONLY these verification tasks do.

- [ ] T080 [P] Harden `scripts/verify.sh` in `scripts/verify.sh` (gates 1–8-shell: fmt, nextest, doc, lint-drift `[lints] workspace=true` + root deny-tables intact, supply-chain `cargo deny check advisories licenses bans sources` + `cargo audit`, deny-list scan, closed-set `no _ => in crates/*/src`, pipeline gate 8-shell (feature-scoped) = engine-handoff conformance only (shape invariant mono-f32@22050 Hz across all 7 rungs + declared granularity per adapter + 22050 Hz stream-request invariant). Constitution gate 8-full (byte-budget <5000 bytes/request + SSML capability + sentence-alignment) is DEFERRED to the synthesis feature per FR-024 (same status as SC-044); verify.sh MUST report `gate-8-shell: PASS` + `gate-8-full: DEFERRED (owner=synthesis)` and MUST NOT exit 0 under the name `gate-8` while those inputs are absent; fail-closed on missing input)
- [ ] T081 [P] Harden automated checks in `scripts/check-manifest.sh` and `scripts/check-shell-purity.sh` (SC-006 manifest incl. minSdk 29 per SC-032 + SC-035 purity + SC-057 closed-set/registry-reference check: every closed set has no catch-all/`other`/`internal`/`unknown`, every engine ID resolves to constitution registry name+version; traceability check: every FR-### / SC-### token in spec.md resolves to a literal token in tasks.md, fail on shorthand-only coverage; check-manifest.sh asserts `<service android:foregroundServiceType="mediaPlayback">` + `FOREGROUND_SERVICE` + `FOREGROUND_SERVICE_MEDIA_PLAYBACK` + `minSdk(Version) 29`; check-shell-purity.sh allowlist = `PlaybackService.kt`, `FocusMirror.kt` (T040), `RoutingMirror.kt` (T047), `MediaMirror.kt` (T062), `Residency.kt` (T064) plus `src-tauri/src/surface.rs` (T065 shapes+getters only), `src-tauri/src/status_view.rs` (T065b read-only render), `src-tauri/src/failure.rs` (T078 conforming strings), `src-tauri/src/permissions.rs` (T030b delegation check) — forward-only intent/state mapping, zero position/rate/transport computation; allowlisted files may carry `transport|position_secs` ONLY in type/getter/delegation positions, never `mut` state, arithmetic, gain application, or skip math; any other match fails)
- [ ] T082 [P] Docs + examples in `docs/` (`docs/architecture.md`, `docs/ipc.md`, `docs/fallback-order.md`, `docs/diagnostic-record.md`; `cargo doc` coverage for every public item per `missing_docs` deny)
- [ ] T083 [P] On-device trial harness in `tests/device/README.md` + `tests/device/trials.md` (10/10 procedures for SC-001, SC-002, SC-003, SC-041, SC-055, SC-004, SC-007, SC-008, SC-009, SC-011, SC-013, SC-014, SC-016, SC-017, SC-018, SC-020, SC-021, SC-022, SC-023, SC-024, SC-025, SC-026, SC-027, SC-028, SC-029, SC-030, SC-031, SC-032-floor, SC-033, SC-033b, SC-034, SC-036, SC-038, SC-039, SC-040, SC-042, SC-043, SC-045, SC-046, SC-047, SC-048, SC-049, SC-050, SC-050b, SC-051, SC-052, SC-053, SC-054; SC-037 via static-scan + manual-absence procedure (grep app surfaces for record-read/export strings, refs T074/T081); SC-010 via surface procedure (T051 prop_surface green + grep android-shell/ + src-tauri/ for body-text render paths returns zero hits + locked visual check 10/10 title+section+progress only); plus FR-026 offline-start procedure — airplane-mode start + 60 s continue with no network; SC-044 marked CARRIED-FORWARD not runnable here; zero-occurrence SC-005, SC-012, SC-019 routed to T087 record scans with procedure refs, NOT to trials; host-decided SC-006, SC-015, SC-035, SC-056, SC-057 noted as N/A on-device with gate refs T081(SC-006, SC-035)/T055(SC-015)/T068(SC-056)/T081(SC-057))
- [ ] T084 VERIFICATION — Host gates: run `./scripts/verify.sh` green on Linux CI where green := gates 1–7 PASS + `gate-8-shell: PASS` + `gate-8-full: DEFERRED (owner=synthesis)` explicitly reported (constitution gate 8 remains OPEN, never claimed as PASS) + `cargo tauri android build --target aarch64` (gate 9) produces device artifact
- [ ] T085 VERIFICATION — quickstart.md validation: execute every command in `specs/001-android-media-shell/quickstart.md` §§1–2 + privacy glance §4 host-checkable parts; record per-step pass/fail
- [ ] T086 VERIFICATION — On-device trials: execute `tests/device/trials.md` each 10/10 on API-29+ hardware (SC-051 two-stand-in failover included; SC-044 explicitly excluded and logged as carried-forward hardware debt)
- [ ] T087 VERIFICATION — Attestation report in `specs/001-android-media-shell/VERIFICATION.md` (per-criterion verdict table for every SC: pass/fail + evidence log line; zero-occurrence criteria SC-005, SC-012, SC-019 backed by record scans; SC-010 backed by notification-surface scan (T051 prop_surface + body-text grep + locked visual check); SC-044 verdict = DEFERRED with owner = synthesis feature; gate-8-full verdict = DEFERRED with owner = synthesis feature)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies. Gate T009 BLOCKS Phase 2.
- **Foundational (Phase 2)**: Depends on T009. Gate T018 BLOCKS all stories. Test harness T010–T013 BLOCKS every story test phase.
- **Stories**: Each story depends on T018. Within a story: ALL its test tasks → story BLOCKING GATE → implementation → GREEN gate. Stories MAY proceed in parallel waves once T018 is green IF staffed (P1 stories US1→US2→US3 recommended order; US4 (P2) after P1 core; US5 (P3) last but its tests may be written in parallel).
- **Polish/Verification (Phase 8)**: Depends on all story GREEN gates (T031, T041, T048, T066, T079). ONLY T084–T087 attest success.

### Within-story TDD order (NON-NEGOTIABLE)

1. Write ALL `Tests` tasks for the story → 2. Run them, confirm FAIL → 3. Pass story BLOCKING GATE → 4. Implement → 5. Tests turn GREEN → 6. GREEN gate → next story.

### Test-before-implementation blocking map

- T023 blocks T024–T031 incl. T030b (US1) • T036 blocks T037–T041 (US2) • T045 blocks T046–T048 (US3) • T056 blocks T057–T066 (US4; T061b already done in test wave) • T072 blocks T073–T076 + T078–T079 (US5; T077 already done in Foundation)

### Parallel Opportunities ([P] waves via subagents)

- Wave 0 (setup): T002–T007 in parallel (separate files), then T008, then gate T009.
- Wave 1 (foundation tests): T010–T013 in parallel (4 agents, separate files, shared context: closed-set list — may reuse one subagent).
- Wave 2 (foundation impl): T014→T015/T016 parallel after T014 (vocabulary first), T017 + T077 parallel-safe (stand-ins depend only on T014/T015/T016 contract shapes).
- Wave 3+ (per story): ALL test tasks of a story in parallel (US1: T019–T022b = 5 agents; US2: T032–T035 = 4 agents; US3: T042–T044 = 3 agents; US4: T049–T055 + T061b fixture = 8 agents; US5: T067, T067b–T071 = 6 agents). Implementation [P] tasks likewise parallel per story.
- Cross-story: after T018, US1/US2/US3 test waves may run concurrently (different files); shared-context tasks (e.g., T077 stand-ins A+B) reuse one subagent.
- Final: T080–T083 in parallel, then strictly sequential T084→T085→T086→T087.

---

## Parallel Example: User Story 2 (focus)

```bash
# Wave 1 — ALL US2 tests together (4 subagents, must ALL fail first):
Task T032: "Property test rewind/snap/cap in crates/player/tests/prop_rewind.rs"
Task T033: "Property test focus classification in crates/player/tests/prop_focus.rs"
Task T034: "Integration test duck-vs-pause in tests/integration/test_focus_duck_pause_us2.rs"
Task T035: "Contract test focus intents in tests/contract/test_focus_intents_us2.rs"
# Gate T036: confirm all FAIL → only then:
# Wave 2 — implementation (T037+T038+T039 parallel, T040 after T038, T041 gate)
```

## Parallel Example: User Story 4 (lock-screen, 8 test agents incl. fixture)

```bash
Task T049: "prop_transport_math in crates/player/tests/prop_transport_math.rs"
Task T050: "prop_resume in crates/session/tests/prop_resume.rs"
Task T051: "prop_surface in crates/player/tests/prop_surface.rs"
Task T052: "prop_offer in crates/session/tests/prop_offer.rs"
Task T053: "lockscreen integration in tests/integration/test_lockscreen_us4.rs"
Task T054: "shell-IPC contract in tests/contract/test_shell_ipc_us4.rs"
Task T055: "pitch invariance in crates/engine-core/tests/prop_pitch.rs"
Task T061b: "doc-structure stub in tests/fixtures/doc_structure.rs (test-wave fixture for T051/T061)"
```

---

## Implementation Strategy

### MVP First (US1 only)

1. Phase 1 Setup (T001–T009) → 2. Phase 2 Foundational (T010–T018) → 3. US1 tests→gate→impl→green (T019–T031) → **STOP, VALIDATE**: screen-off 5-min playback + SC-005 scan → demo.

### Incremental Delivery

Foundation → +US1 (locked playback, MVP) → +US2 (focus, P1 core) → +US3 (routing, P1 core) → +US4 (lock-screen, P2) → +US5 (failures/record, P3) → Phase 8 verification. Each increment independently testable per story Checkpoint.

### Subagent-Driven Development

- Every [P] task is delegated, never executed inline in bulk. Maximum viable agents per wave (counts above).
- Same-subagent reuse where context is shared (closed vocabulary T010/T014; stand-ins T071/T077; surface T051/T061b/T061/T062).
- Background agents for long builds (`cargo tauri android build`, on-device 10/10 trials).
- After EVERY file edit during implementation: atomic conventional commit (`feat|fix|test|chore(scope): summary` + detailed body). This tasks.md creation is itself committed atomically.

### Scope Guardrails (from plan/constitution — not negotiable in tasks)

- Closed sets everywhere: no `Other/Unknown`, no `_ =>` in `crates/*/src` (gate 7 / SC-057, FR-044).
- Rust owns ALL logic; Kotlin is mirror-only (SC-035, FR-042). Tauri 2.12.1 pinned (not 3.0.0-alpha).
- Mono f32 @22050 Hz handoff; mismatch = named `EngineError`, never silent resample (SC-034, FR-033, FR-034, FR-035).
- Record: 30-day + 10 MB oldest-first, app-private, never transmitted, no in-app surface, no document text, `detail` ≤200 chars identifier-pattern, `error` on `failure`+`refusal` (SC-025, SC-056, SC-037; FR-039, FR-039a, FR-039b, FR-040, FR-045).
- FR-025 (Android Auto): no build task by design — exclusion verified by absence, not by trial.
- SC-044 (real-engine hardware) is CARRIED FORWARD, discharged by SC-051 (stand-ins) here.

---

## Notes

- Total: **92 tasks** (T001–T087 + T022b + T030b + T061b + T065b + T067b). Tests: 29 (T010–T013 foundation + 25 story tests). BLOCKING gates: 7 red (T009, T018, T023, T036, T045, T056, T072) + 5 GREEN (T031/T041/T048/T066/T079). Cross-reference notes carry no checkbox ID and are excluded from the count.
- Per story: US1 14 (5 tests), US2 10 (4 tests), US3 7 (3 tests), US4 20 (7 tests + T061b fixture in test wave), US5 14 (6 tests). Setup 9, Foundation 9 (4 tests), Polish/Verification 8 (4 verification).
- [P] tasks: 59 — the parallel delegation surface. Non-[P] tasks are gates or same-file sequences.
- MVP scope: Phases 1+2+3 (T001–T031) = workspace + locked playback.
- Verification tasks T084–T087 are the SOLE attestation of success. Agent claims attest nothing.
