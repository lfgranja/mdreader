# Tasks: Ressoa Dogfood Build (004-ressoa-dogfood-build)

**Input**: `specs/004-ressoa-dogfood-build/{spec.md, plan.md, research.md, data-model.md, contracts/, quickstart.md}`
**Prerequisites**: plan.md (required) · spec.md (user stories P1/P2) · research.md (R1–R7) · data-model.md · contracts/{ingest,surface,earcon,diagnostic-trace}.md
**Constitution**: `.specify/memory/constitution.md` v1.5.0 (Rust rigor, closed sets gate 7, supply-chain gates 5–6, single playback authority, manual manifest)

**Execution rules (binding for this file)**:
1. **TDD gate**: every `### Tests` subsection is BLOCKING — all its tasks MUST be written, shown to FAIL (`cargo nextest`), then implementation begins. No implementation task in the same phase starts until its phase test gate is red.
2. **Property-based first**: invariants use `proptest` wherever applicable (`prop_*`); unit edge cases complement, never replace, the property.
3. **[P] = subagent wave**: every `[P]` task is delegated to the maximum viable number of agents/subagents/background agents; tasks sharing context may reuse one subagent; waves run per the Parallel Examples below.
4. **Verification attests**: ONLY the Final Phase verification tasks attest success/completeness — NEVER the implementing agent's claim.
5. **Deterministic hook**: whenever editing a file finishes, make an atomic conventional commit with a detailed edit-summary (`feat(004): …`, `test(004): …`, `docs(004): …`).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: parallelizable (different files, no dependencies on incomplete tasks) → subagent-driven, in waves
- **[Story]**: US1/US2/US3/US4 maps to spec.md user stories
- Every task names its exact file path; tasks are specific enough to execute without extra context

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Fixture, scaffolds, and shared type skeletons that everything else builds on. No behavior yet.

- [ ] T001 Create Portuguese acceptance fixture `tests/device/fixtures/fixture-pt-2h.md` (Markdown with headings + numbers/currency, ~2 h at 1.0x; SC-001 input for every speed incl. 4.0x)
- [ ] T002a [P] Add snapshot `.gitignore` exception in `.gitignore` (ignore `snapshots/` content; no logic yet — parallel-safe with T002b, different file)
- [ ] T002b [P] Add snapshot directory convention const in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` (header const `SNAPSHOT_DIR="filesDir/snapshots"`, `<hex>.{md,txt}` naming; no logic yet — parallel-safe with T002a, different file)
- [ ] T003 [P] Scaffold `crates/player/src/earcon.rs` module declaration + closed `EarconKind` enum (`Play|Pause|SkipFwd|SkipBack|Warning|Failure|RateStep`, no `Other`, wired in `crates/player/src/lib.rs`)
- [ ] T004 [P] Scaffold `crates/player/src/interruption.rs` closed 8-reason enum (`FocusDenied|FocusDelayed|PermanentLoss|TransientPaused|DeviceRemoved|VoiceUnavailable|FormatUndeliverable|DecodeFailed`, no `Other`, exhaustive `reason_line()` stub ≤8 words) wired in `crates/player/src/lib.rs`
- [ ] T005 [P] Change `Progress` payload type in `crates/player/src/surface.rs` to content-offset shape (`Determinate{consumed_units,total_units,k,n} | Indeterminate`, fixed-at-session-start flag; keep `None`-section + dismissal rules)

**Note**: T002 split into T002a/T002b (different files, both `[P]`); all later IDs T003–T052 unchanged.

**Checkpoint**: `cargo check --workspace` green; skeletons compile; zero behavior changed.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Test harness + shared wiring points that MUST complete before ANY user story. **⚠️ CRITICAL: no US-phase implementation starts until T006–T012 are all FAIL-red.**

### Tests (BLOCKING — write first, show FAIL)

- [ ] T006 [P] Property `prop_progress_fraction` in `crates/player/tests/prop_progress.rs` (invariants: `0 ≤ consumed ≤ total`; determinate fixed at session start, never flips mid-session; zero sections ⇒ indeterminate; re-parse never moves backward — freeze-then-drop)
- [ ] T007 [P] Property `prop_earcon_band_duration` in `crates/player/tests/prop_earcon.rs` (invariants: every kind ≤500 ms, band 300–2500 Hz, ≥2 of {timbre,rhythm,contour} distinct pairwise; PCM mono f32 @ 22050 Hz)
- [ ] T008 [P] Property `prop_persist_cadence` in `crates/player/tests/prop_persist.rs` (invariants: durable writes every 20 s ± tolerance (fixed 20s within 15-30s envelope) while Playing + on every listed event; cursor = last-spoken never buffered-ahead; worst-case loss ≤ one interval; replay ≤30 s interruption / ≤60 s kill)
- [ ] T009 [P] Property `prop_ingest_identity` in `crates/session/tests/prop_ingest.rs` (invariants: identical bytes at any path ⇒ identical blake3; edited byte ⇒ different hash ⇒ no resume; snapshot bytes are the synthesis input, never the URI)
- [ ] T010 [P] Contract test `contracts/diagnostic-trace` kinds in `crates/diag/tests/contract_trace.rs` (required kinds present: session/transport/per-section/per-rate/buffer/`FocusTransition`+measured `play_to_audio_ms`/`removal_pause_ms`/`replay_secs`/failure+`error`; `detail` ≤200 ch; failure/refusal without `error` FAILS; prose predicate FAILS)

### Wiring (starts only after T006–T010 are red)

- [ ] T011 20 s durable heartbeat scheduler skeleton in `src-tauri/src/lib.rs` (timer calls `BookmarkStore::put` while Playing; event-durable `put` call sites listed for US3; atomic single-row tx shape)
- [ ] T012 Diagnostic measured-latency `detail` keys (`play_to_audio_ms`, `removal_pause_ms`, `replay_secs`) in `crates/diag/src/writer.rs` (typed fields, ≤200 ch, hash refs only, never prose)

**Checkpoint (TDD gate)**: `cargo nextest run --workspace` shows T006–T010 FAIL (red) before any Phase 3+ implementation. Record the red output in the commit message.

---

## Phase 3: User Story 1 — Listen through a whole commute with the screen off (Priority: P1) 🎯 MVP

**Goal**: Picker → snapshot → screen-off continuous reading with lock-screen transport; first audio ≤3 s.
**Independent Test**: Sideload on reference phone, start `fixture-pt-2h.md`, lock 30 min, continuous audio with lock-screen controls only, zero unlocks (quickstart §3.1–3.2); host: T013–T016 green.

### Tests for US1 (BLOCKING — write first, FAIL before T017+) ⚠️

- [ ] T013 [P] [US1] Property `prop_skip_section_fallback` in `crates/player/tests/prop_skip.rs` (invariants: skip lands on next/previous section boundary; zero sections ⇒ exactly ±10 s; never mid-word anchor lost; clamp at doc ends)
- [ ] T014 [P] [US1] Integration `ingest → snapshot → fingerprint → parse` in `crates/session/tests/integration_ingest.rs` (pick `.md`/`.txt` ⇒ snapshot written, `fingerprint()` matches, `store.get` decides resume; moved/renamed re-pick ⇒ hash-match restore; revoked grant ⇒ re-prompt, never crash; non-`.md`/`.txt` pick ⇒ refused with `FormatUndeliverable` before synthesis)
- [ ] T015 [P] [US1] Contract test lock-screen payload (no Stop CONTROL; `stopped` STATE permitted) in `src-tauri/tests/test_surface_us1.rs` (payload matches `contracts/surface.md`: title+section+progress only, transport ∈ `{playing,paused,stopped}`, no `stop` control, no prose/times/rate-VALUE/voice keys — rate-step action permitted, no value display)
- [ ] T016 [P] [US1] Play-to-audio bound harness in `crates/player/tests/test_play_latency.rs` (cold-start Play ⇒ first audible ≤3 s declared; measures and logs `play_to_audio_ms` into diag `detail`)

### Implementation for US1

- [ ] T017 [P] [US1] Kotlin picker + snapshot write in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` (per `contracts/ingest.md`: `ACTION_OPEN_DOCUMENT`, `text/markdown`+`text/plain` filter, persistable grant best-effort `try/catch(SecurityException)`, `ContentResolver` single read, `filesDir/snapshots/<hex>.{md,txt}` write/trim; forward-only, zero position logic)
- [ ] T018 [P] [US1] `session ingest` CLI verb in `crates/session/src/cli.rs` (hash + `store.get` → `{fingerprint, resume: bool, offset}` JSON; Tauri IPC calls same fn; `--format json` default)
- [ ] T019 [US1] `open_document(bytes)` command reusing `play()` path in `src-tauri/src/lib.rs` (depends on T017, T018; snapshot bytes are the sole synthesis input; totals `total_n`+`total_units` fixed at session start; pure-offline, zero network)
- [ ] T020 [P] [US1] Section-boundary skip in `crates/player/src/skip.rs` (next/previous boundary via `doc-structure`, ±10 s fallback when zero sections, boundary clamp)
- [ ] T021 [US1] Single minimal in-app screen transport + Stop in `src-tauri/src/status_view.rs` (+ Svelte view `src-tauri/src/status_view.html`) (depends on T019; title/section/progress/speed/transport + dedicated Stop clearing stored position; never prose; read-only mirror owning no state)
- [ ] T022 [US1] Foreground residency + screen-off continuity assertion in `android-shell/src/main/kotlin/app/mdreader/PlaybackService.kt` (reuse existing holder; assert audio continues locked; dismissal refused while playing)

**Checkpoint**: US1 independently functional — host `cargo nextest` green for T013–T016; on-device 30-min locked session + 10/10 ≤3 s trials per quickstart §3.1–3.2.

---

## Phase 4: User Story 2 — Survive interruptions without looking (Priority: P1)

**Goal**: Classified focus/routing outcomes + blind earcon/haptic confirmation; replay ≤30 s at sentence-start.
**Independent Test**: 10/10 per class (nav-duck vs call/alarm pause vs headphone ≤1 s vs permanent loss, no unlock, no spontaneous resume) + 10/10 blind per confirmation type sounds-on and -off (quickstart §3.3–3.4); host: T023–T026 green.

### Tests for US2 (BLOCKING — write first, FAIL before T027+) ⚠️

- [ ] T023 [P] [US2] Property `prop_focus_latest_wins` in `crates/player/tests/prop_focus8.rs` (invariants: 8 reasons exhaustive, no `_ =>`; transient-immediately-followed-by-permanent ⇒ final paused no-auto-resume with latest reason; `Deferred` ⇒ zero-audio hold with no reason; duck keeps `Playing` with `DUCK_GAIN 0.20 ±0.02` + restore without place loss, never owes rewind; call/alarm pause ≤1s)
- [ ] T024 [P] [US2] Property `prop_routing_pause_bound` in `crates/player/tests/prop_routing.rs` (invariants: `BecomingNoisy`/`DeviceDisappeared` ⇒ pause decision; declared 250 ms ≤ 1000 ms bound; never routes to speaker)
- [ ] T025 [P] [US2] Property `prop_resume_snap_cap` in `crates/player/tests/prop_resume.rs` (invariants: resume at containing-sentence-start + 5–10 s cue, never mid-word; total backward ≤30 s interruption / ≤60 s kill; persisted cursor is last-spoken)
- [ ] T026 [P] [US2] Earcon mirror 1:1 test in `crates/player/tests/test_earcon_mirror.rs` (every trigger ⇒ same-call lock-screen state change + `reason_ttl_ms=3000` for pause-demanding (distinct from `dismissal`); undeliverables (`VoiceUnavailable`/`FormatUndeliverable`/`DecodeFailed`) ⇒ never start + persistent line + `error` logged; earcon PCM never advances bookmark; ducked gain never quiets earcon)

### Implementation for US2

- [ ] T027 [P] [US2] 8-reason mapping + ≤8-word lines in `crates/player/src/interruption.rs` (`TransientPaused` split: `MAY_DUCK`→duck ~20% keep playing / non-duckable→pause; call/alarm pause ≤1s; `FocusDenied`→`Refused`, `FocusDelayed`→`Deferred` hold; `PermanentLoss`/`DeviceRemoved`→pause latest-wins; undeliverables (`VoiceUnavailable`/`FormatUndeliverable`/`DecodeFailed`)→never start + persistent line + `error` logged; no `Other`)
- [ ] T028 [P] [US2] Procedural earcon synthesis in `crates/player/src/earcon.rs` (grammar per `contracts/earcon.md` incl. failure E4 ~330 Hz transposition; mono f32 22050; constructors CLI-drivable)
- [ ] T029 [US2] Duck-exempt short-mix bypass in `crates/player/src/backend.rs` (depends on T028; earcon skips `SampleQueue::gain`, same stream/clock, files no focus request, re-fires post-`GAIN` if missed, never moves bookmark)
- [ ] T030 [P] [US2] `player earcon` CLI verb in `crates/player/src/cli.rs` (`earcon <play|pause|skip-fwd|skip-back|warning|failure|rate> --format json` ⇒ PCM descriptor + duration/band asserts)
- [ ] T031 [US2] Haptic patterns in `MediaMirror.publish()` in `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` (+ `VIBRATE` permission; depends on T027; distinct `VibrationEffect.createOneShot` per action; outside `STREAM_MUSIC`; visual glyph+progress banner covers fully-disabled-haptic)
- [ ] T032 [US2] Sentence-snap resume path in `crates/player/src/rewind.rs` + `crates/player/src/resume.rs` (containing-sentence-start snap, mid-word forward-snap, cap-precedence, cue 5–10 s; degrade to raw cursor never doc start)

**Checkpoint**: US2 independently functional — host green; on-device 10/10 per interruption class + 10/10 blind per confirmation type.

---

## Phase 5: User Story 3 — Never lose your place (Priority: P2)

**Goal**: Content-keyed durable position with 3 s deliberate offer vs explicit lock-screen resume; edit/stop ⇒ no resume.
**Independent Test**: 10/10 move+rename ⇒ offer; 10/10 edit ⇒ no resume; Stop ⇒ start at beginning; kill ⇒ ≤60 s replay (quickstart §3.5); host: T033–T034 green.

### Tests for US3 (BLOCKING — write first, FAIL before T035+) ⚠️

- [ ] T033 [P] [US3] Property `prop_bookmark_lifecycle` in `crates/session/tests/prop_bookmark.rs` (invariants: `put` atomic single-row; `Stopped|Completed` ⇒ deleted; `Paused`⇒deliberate 3 s offer silence-continues; `Interrupted`⇒explicit resume never auto-starts; content-change hash ⇒ miss ⇒ `0.0` no offer)
- [ ] T034 [P] [US3] Offer-window test in `crates/session/tests/test_offer3s.rs` (deliberate reopen ⇒ visual dialog 3 s continue-vs-start-over; kill/power-loss ⇒ paused + lock-screen resume control; `Stop` clears before next open)

### Implementation for US3

- [ ] T035 [US3] Event-durable `put` call sites in `crates/player/src/position.rs` + `src-tauri/src/lib.rs` (depends on T011; durable on pause, focus-loss, `BecomingNoisy`/`DeviceDisappeared`, section-change, Stop-then-`delete`, app-background, `DocumentEnd`; in-RAM ~1 s tick; non-durable per sentence/5 s snap-back source only)
- [ ] T036 [P] [US3] Re-hash-on-open lookup in `crates/session/src/store.rs` + `crates/session/src/offer.rs` (bytes → `fingerprint()` → `get` → end-kind-routed offer; identical-bytes-any-path shares position)
- [ ] T037 [P] [US3] Visual-dialog vs lock-screen-resume routing in `src-tauri/src/lib.rs` (deliberate ⇒ in-app 3 s dialog; unexpected ⇒ wait-paused + explicit lock-screen control; silence rule per FR-009)

**Checkpoint**: US3 independently functional — host green; on-device 10/10 resume matrix + replay caps.

---

## Phase 6: User Story 4 — A lock screen that respects attention (Priority: P2)

**Goal**: Purity (title+section+progress only), 7-rung persistent pitch-preserving rate, honest determinacy, bounded private full-trace record.
**Independent Test**: 10/10 purity inspections; rate persists + unpitched; diag within 30-day/10 MB with zero prose (quickstart §3.6–3.8); host: T038–T040 green.

### Tests for US4 (BLOCKING — write first, FAIL before T041+) ⚠️

- [ ] T038 [P] [US4] Purity property in `src-tauri/tests/test_purity.rs` (lock-screen + in-app snapshots across playing/paused/section-change/rate-change/interruption ⇒ keys ⊆ `{title, section_title, progress}`; zero prose/time/rate-VALUE/voice leaks — rate-step action permitted, no value display; dismissal-refused-while-playing assert; zero sections ⇒ `section_title == null` + indeterminate)
- [ ] T039 [P] [US4] Rate-ladder property in `crates/player/tests/prop_ladder.rs` (7 rungs `[0.75,1.0,1.25,1.5,1.75,2.0,4.0]` exact, one step saturating, `settings.last_rate` persists across sessions/documents, pitch within `PITCH_TOLERANCE_CENTS=50` every rung incl. 2.0x→4.0x)
- [ ] T040 [P] [US4] Diag no-prose + bounds property in `crates/diag/tests/prop_trace.rs` (30-day + 10 MB oldest-first with `Trim` before retained, `Gap` on full with `Ok`; `ProseInDetail`/`DetailTooLong`/`FailureWithoutError`/`RefusalWithoutError` scan violations trip; zero egress — no network call sites)

### Implementation for US4

- [ ] T041 [US4] Content-offset progress rendering in `crates/player/src/surface.rs` (depends on T005; bar = `consumed_units/total_units`, label frozen template `"Section {k} of {N} — {p}% (content-offset)"`; determinate iff both totals known at session start; freeze-then-drop; audio-duration never feeds it; no seek without random access)
- [ ] T042 [P] [US4] Rate stepper + persistence in `crates/player/src/rate.rs` + `crates/session/src/store.rs` (in-app stepper + lock-screen rate-step action without value display — no rate leak, earcon+haptic confirm; engine-contract stretch, `last_rate` read-at-start/write-on-change; acceptance: locked rate-step works, no numeric leak)
- [ ] T043 [P] [US4] Reason-expiry + dismissal in `src-tauri/src/surface.rs` + `src-tauri/src/failure.rs` (pause-demanding 3 s `reason_ttl_ms=3000` distinct from `dismissal` ∈ `{non_dismissable_while_playing,dismissable}`; undeliverables (`VoiceUnavailable`/`FormatUndeliverable`/`DecodeFailed`) ⇒ never start + persistent line + `error` logged; non-dismissable while playing; ≤8-word lines; failures never quote document text)
- [ ] T044 [P] [US4] DESIGN-SONA token styling of MediaStyle + in-app grid in `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` + `src-tauri/src/status_view.html` (title `#F2F0E6`, section `#A9B8CE`, progress white/`#55688F`; yellow `#FFD400` warning-only, red `#FF4D63` failure-only, green `#5FD0A0` prohibited; no marquee/body/rate-display/voice — no rate value display)

**Checkpoint**: US4 independently functional — host green; on-device purity/rate/record trials.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Docs, hygiene, and scope guards. No behavior changes except T047 wiring already listed.

- [ ] T045 [P] `docs/` + `tests/device/README.md` trial procedures update (004 commute/blind/resume/purity/rate/record per quickstart §3; reference-phone-only scope; out-of-scope absence checklist: zero network voice call sites, exactly one voice dir, no library view, lock-screen no Stop CONTROL, no Auto/head-unit service, no in-app diag surface, no distribution artifacts)
- [ ] T046 [P] `scripts/expected-tests.txt` target-list sync (new `prop_*`/contract suites recorded so gate 2 fails closed on missing *or* unexpected targets)
- [ ] T047 Out-of-scope absence guard test in `src-tauri/tests/test_noscope.rs` (asserts zero network voice call sites, exactly one bundled voice dir, no library view, lock-screen no Stop CONTROL, no Auto/head-unit service, no in-app diag surface, no distribution artifacts + naming-boundary grep: no `mdreader` in user-visible strings, Ressoa-external only)

---

## Phase 8: Verification (ATTESTS — only this phase declares success)

**Purpose**: Deterministic gates + on-device trials. ONLY these tasks attest completeness; agent claims attest nothing.

- [ ] T048 Host gates 1–8 via `./scripts/verify.sh` (format, nextest vs `expected-tests.txt`, doc, lint-drift, supply-chain `cargo deny` + `cargo audit`, deny-list, closed-set no `_ =>`, pipeline shell-bounds) — record exit status + log
- [ ] T049 Manifest + shell-purity via `./scripts/check-manifest.sh` + `./scripts/check-shell-purity.sh` (service `mediaPlayback` + `FOREGROUND_SERVICE*` + minSdk 29; zero transport/rate/position logic in `android-shell/`; naming-boundary: no `mdreader` in user-visible strings)
- [ ] T050 Android artifact via `cargo tauri android build --target aarch64` (gate 9) + maintainer sideload `adb install -r <apk>` — record APK hash
- [ ] T051 On-device SC-001…SC-008 trials per `quickstart.md` §3 on the reference phone only (30-min locked commute at every speed; 10/10 ≤3 s; 10/10 per interruption class; 10/10 blind per confirmation sounds-on/off; 10/10 resume matrix + replay caps; 10/10 purity; rate persistence + unpitched; diag `adb pull` bounds + zero prose) — record per-SC 10/10 sheets
- [ ] T052 Diagnostic-record audit via `diag scan` CLI on the pulled JSONL (schema conformance + negative control, zero prose, `error` on every failure/refusal, `Trim`/`Gap` markers sane) — attach scan output

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies — start immediately. T002a–T005 are `[P]` (different files).
- **Foundational (Phase 2)**: depends on Phase 1 skeletons. T006–T010 `[P]` run as **Wave F1** (5 subagents, one per test file); T011–T012 after red is shown.
- **US phases (3–6)**: ALL depend on Foundational red→green cycle. Within each US phase: `### Tests` BLOCK all implementation in that phase.
- **Polish (Phase 7)**: depends on all desired US phases complete. T045–T047 `[P]`.
- **Verification (Phase 8)**: depends on everything; runs SEQUENTIALLY (T048→T052), never parallel (shared device + shared target dir).

### User Story Dependencies

- **US1 (P1)**: after Foundational — no deps on other stories. MVP = Setup + Foundational + US1.
- **US2 (P1)**: after Foundational — may reuse US1 surfaces but independently testable (own `prop_*` + mirror tests).
- **US3 (P2)**: after Foundational — reuses ingest hash + `put` points; independently testable via bookmark/offer tests.
- **US4 (P2)**: after Foundational — reuses `Progress`/rate/diag shapes; independently testable via purity/ladder/trace tests.

### Within Each User Story

Tests (BLOCKING, FAIL-red) → types/enums → services/paths → surfaces → integration. Models before services before endpoints before integration. Story checkpoint before next priority.

### Parallel Opportunities (subagent waves)

Waves may overlap but no US impl starts before its own Tests are red (per-phase gate preserved).

- **Wave S**: T002a, T002b, T003, T004, T005 (5 subagents) after T001.
- **Wave F1 (tests)**: T006, T007, T008, T009, T010 (5 subagents, background OK).
- **Wave U1-tests**: T013, T014, T015, T016 (4 subagents) — same wave allowed across US phases once Foundational is green (max viable: all `[P]` test tasks U1–U4 in one background wave of 13 subagents if capacity allows; tasks sharing `player` context may reuse one subagent per crate).
- **Wave U1-impl**: T017, T018, T020 `[P]` (3 subagents, different files); T019, T021, T022 follow deps.
- **Wave U2**: T023, T024, T025, T026 `[P]` (4 subagents); then T027, T028, T030, T031 `[P]` (4 subagents, different files); T029, T032 follow deps.
- **Wave U3/U4-tests**: T033, T034, T038, T039, T040 `[P]` (5 subagents).
- **Wave U3/U4-impl**: T036, T037, T042, T043, T044 `[P]` (5 subagents, different files); T035, T041 follow deps.
- **Wave P7**: T045, T046, T047 `[P]` (3 subagents).

---

## Parallel Example: User Story 1

```bash
# Wave U1-tests — 4 subagents, background, fail-red gate:
Subagent A: "T013 prop_skip_section_fallback in crates/player/tests/prop_skip.rs (proptest; FAIL-red first)"
Subagent B: "T014 integration_ingest in crates/session/tests/integration_ingest.rs (FAIL-red first)"
Subagent C: "T015 test_surface_us1 in src-tauri/tests/test_surface_us1.rs (FAIL-red first)"
Subagent D: "T016 test_play_latency in crates/player/tests/test_play_latency.rs (FAIL-red first)"
# Gate: cargo nextest shows all four FAIL before T017+ starts.

# Wave U1-impl — 3 subagents, different files:
Subagent A: "T017 DocIngest.kt picker+snapshot (contracts/ingest.md)"
Subagent B: "T018 session cli.rs ingest verb"
Subagent C: "T020 player skip.rs section ±10 s fallback"
# Then: T019 (depends A,B) → T021 → T022. Commit atomically per file (hook §5).
```

## Parallel Example: User Story 2

```bash
# Wave U2-tests — 4 subagents:
Subagent A: "T023 prop_focus8 in crates/player/tests/prop_focus8.rs"
Subagent B: "T024 prop_routing in crates/player/tests/prop_routing.rs"
Subagent C: "T025 prop_resume in crates/player/tests/prop_resume.rs"
Subagent D: "T026 test_earcon_mirror in crates/player/tests/test_earcon_mirror.rs"
# Wave U2-impl[P] — 4 subagents, different files:
Subagent A: "T027 interruption.rs 8-reason mapping"
Subagent B: "T028 earcon.rs procedural synthesis"
Subagent C: "T030 player cli.rs earcon verb"
Subagent D: "T031 MediaMirror.kt haptics"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 (T001–T005) → Phase 2 tests red (T006–T010) → wiring (T011–T012).
2. US1 tests red (T013–T016) → US1 impl (T017–T022).
3. **STOP and VALIDATE**: host green + 30-min locked commute + 10/10 ≤3 s trials. Sideloadable MVP.

### Incremental Delivery

1. Foundation ready → US1 (MVP) → US2 (interruptions + blind) → US3 (resume) → US4 (purity/rate/record) → Polish → Verification.
2. Each story adds value without breaking previous stories (checkpoints enforce).

### Subagent Team Strategy

1. Whole team: Setup + Foundational Wave F1 (up to 5 background subagents).
2. Once Foundational green: fan out per wave above — Developer/Subagent per `[P]` task; shared-crate tasks reuse one subagent to preserve context.
3. Verification (Phase 8) single-threaded: one agent, sequential, evidence attached. Its output — not any implementer's claim — is the success verdict.

---

## Notes

- `[P]` = different files, no dependencies → subagent wave, background where viable.
- `[USn]` maps task to spec.md story for traceability.
- Quote data-model constraints verbatim in code (e.g. `Progress` fixed-at-start, `detail ≤200 ch`, `error` required on failure *and* refusal, `TransientPaused` duck-vs-pause split, zero-sections ⇒ indeterminate + null title).
- `DecodeFailed` rides `EngineError::Unreachable` on the wire (`decode_failed` in diag only) — no `EngineError` edit, no semver bump.
- New public APIs require rustdoc (`missing_docs` deny, gate 3).
- After EVERY file edit: atomic conventional commit with detailed summary (rule 5). Suggested: `test(004): T013 prop_skip red` → `feat(004): T020 section skip` → … → `verify(004): T048 gates-1-8 PASS`.
- Stop at any checkpoint to validate the story independently; measurements that miss bounds stop the line for any later distribution decision (spec Assumptions).
