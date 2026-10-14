# Tasks: Ressoa Dogfood Build (004-ressoa-dogfood-build)

**Input**: `specs/004-ressoa-dogfood-build/{spec.md, plan.md, research.md, data-model.md, contracts/, quickstart.md}`
**Prerequisites**: plan.md (required) · spec.md (user stories P1/P2) · research.md (R1–R7) · data-model.md · contracts/{ingest,surface,earcon,diagnostic-trace}.md
**Constitution**: `.specify/memory/constitution.md` v1.5.0 (Rust rigor, closed sets gate 7, supply-chain gates 5–6, single playback authority, manual manifest)

**Execution rules (binding for this file)**:
1. **TDD gate (strict-global, binding)**: ALL `### Tests` subsections across ALL phases (T002d–T002f, T006–T010, T013–T016, T023–T026, T033–T034, T038–T040b) MUST be written and shown to FAIL (cargo nextest red) BEFORE ANY implementation task (T003–T005, T011–T012, T017+ in any phase) begins. Per-phase gates still hold inside the global gate. The 2026-10-13 per-phase-only interpretation and waiver are REVOKED; strict-global satisfies user constraint 1. Foundational T006–T010 red additionally blocks ALL US-phase impl (retained). Record red output in commit messages.
2. **Property-based first**: invariants use `proptest` wherever applicable (`prop_*`); unit edge cases complement, never replace, the property. Snapshot/contract tests (T026/T034/T038) are normative where they assert fixed state-snapshot/contract shapes (mirror 1:1, 3 s offer window, purity key subset), with `prop_*` complementing where applicable per Notes U2 (T026 by T007+T023; T034 by T033; T038 by T006); timing/state invariants without prop cover (3 s TTL exact, glyph snapshot) remain snapshot-asserted with 10/10 on-device trials (T051) as complement — satisfaction of this rule, not an exemption.
3. **[P] = subagent wave**: every `[P]` task is delegated to the maximum viable number of agents/subagents/background agents; tasks sharing context may reuse one subagent; waves run per the Parallel Examples below.
4. **Verification attests**: ONLY the Final Phase verification tasks attest success/completeness — NEVER the implementing agent's claim.
5. **Deterministic hook**: whenever editing a file finishes, make an atomic conventional commit with a detailed edit-summary (`feat(004): …`, `test(004): …`, `docs(004): …`).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: parallelizable (different files WITHIN the same wave, no dependencies on incomplete tasks; same file allowed ONLY across different waves/phases with an explicit serialize order, never batched in one wave) → subagent-driven, in waves. Same-file serialize orders (binding): `src-tauri/src/lib.rs`: T011→T019→T037→T035; `crates/player/src/cli.rs`: T030 then T030b; `crates/session/src/store.rs`: T036→T042.
- **[Story]**: US1/US2/US3/US4 maps to spec.md user stories
- Every task names its exact file path; tasks are specific enough to execute without extra context

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Fixture, scaffolds, and shared type skeletons that everything else builds on. No behavior yet.

- [ ] T001 [P] Create Portuguese acceptance fixture `tests/device/fixtures/fixture-pt-2h.md` + co-located `fixture-pt-2h.README.md` (Markdown with headings + numbers/currency, ~2 h at 1.0x; README records estimated duration at 1.0x from word-count + 4.0x-sustains-30-min computation; SC-001 input for every speed incl. 4.0x; both files committed)
- [ ] T001b [P] Fix oversize-document byte cap: measure `fixture-pt-2h.md` byte size, set `MAX_DOC_BYTES = <fixture_bytes × 1.5>` (headroom for ~3 h at 1.0x) as a named constant in `crates/player/src/interruption.rs` (or `crates/session/src/ingest` module); document the value + measurement method in the fixture README; wire the cap check into the FR-018 ingest refusal path (FormatUndeliverable when exceeded) — depends on T001
- [ ] T002a [P] Add snapshot `.gitignore` exception in repo-root `.gitignore` (ignore repo-local `/snapshots/` content — on-device `filesDir/snapshots` never enters git; no logic yet — parallel-safe with T002b, different file)
- [ ] T002b [P] Add snapshot directory convention const in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` (header const `SNAPSHOT_DIR="filesDir/snapshots"`, `<hex>.{md,txt}` naming; no logic yet — parallel-safe with T002a, different file)
- [ ] T002c [P] Verify workspace `proptest` dev-dependency in `Cargo.toml` (`[workspace.dependencies]` pin + `proptest = { workspace = true }` in `[dev-dependencies]` of each member with `prop_*` suites; re-verify latest stable on crates.io at implementation time and record exact version in atomic commit msg per Principle IV; `Cargo.lock` committed + `--locked` build; `deny.toml` `[sources]` crates.io-only; `cargo deny check advisories licenses bans sources` + `cargo audit` clean; `[lints] workspace = true` intact, no `Cargo.toml`/`clippy.toml` weakening, no `Other`/`Unknown`, no `_ =>`; parallel-safe with T002a/T002b/T002d–T002f, different files) (workspace-wide — single subagent, runs first in Wave S)

### Setup Tests (BLOCKING — write first, FAIL before T003+) ⚠️

- [ ] T002d [P] Closed-set shape test for `EarconKind` in `crates/player/tests/test_setup_earcon_shape.rs` (asserts `EarconKind` has exactly `Play|Pause|SkipFwd|SkipBack|Warning|Failure|RateStep`, no `Other`/`Unknown`, exhaustive `match` with no `_ =>`; FAIL-red before T003)
- [ ] T002e [P] Closed-set shape test for interruption reasons in `crates/player/tests/test_setup_interruption_shape.rs` (asserts 8-reason enum `FocusDenied|FocusDelayed|PermanentLoss|TransientPaused|DeviceRemoved|VoiceUnavailable|FormatUndeliverable|DecodeFailed`, no `Other`, exhaustive `reason_line()` stub ≤8 words with no `_ =>`; FAIL-red before T004)
- [ ] T002f [P] Type-shape test for `Progress` payload in `crates/player/tests/test_setup_progress_shape.rs` (asserts `Determinate{consumed_units,total_units,k,n} | Indeterminate` shape + fixed-at-session-start flag, `None`-section + dismissal rules; FAIL-red before T005)

### Scaffolds (start only after T002d–T002f are red)

- [ ] T003 [P] Scaffold `crates/player/src/earcon.rs` module declaration + closed `EarconKind` enum (`Play|Pause|SkipFwd|SkipBack|Warning|Failure|RateStep`, no `Other`, wired in `crates/player/src/lib.rs`)
- [ ] T004 [P] Scaffold `crates/player/src/interruption.rs` closed 8-reason enum (`FocusDenied|FocusDelayed|PermanentLoss|TransientPaused|DeviceRemoved|VoiceUnavailable|FormatUndeliverable|DecodeFailed`, no `Other`, exhaustive `reason_line()` stub ≤8 words) wired in `crates/player/src/lib.rs`
- [ ] T005 [P] Change `Progress` payload type in `crates/player/src/surface.rs` to content-offset shape (`Determinate{consumed_units,total_units,k,n} | Indeterminate`, fixed-at-session-start flag; keep `None`-section + dismissal rules)

**Note**: T002 split into T002a/T002b (different files, both `[P]`); T002c workspace `proptest` verification + T002d–T002f Setup Tests added without renumbering; all later IDs T003–T052 unchanged.

**Checkpoint (TDD gate — Setup)**: `cargo nextest run --workspace` shows T002d–T002f FAIL (red) before T003–T005; then `cargo check --workspace` green; skeletons compile; zero behavior changed. Record the red output in the commit message. Global gate: T003+ waits for ALL phase Tests red, not just T002d–f.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Test harness + shared wiring points that MUST complete before ANY user story. **⚠️ CRITICAL: no US-phase implementation starts until T006–T012 are all FAIL-red.**

### Tests (BLOCKING — write first, show FAIL)

- [ ] T006 [P] Property `prop_progress_fraction` in `crates/player/tests/prop_progress.rs` (invariants: `0 ≤ consumed ≤ total`; determinate fixed at session start, never flips mid-session; zero sections ⇒ indeterminate; re-parse never moves backward — freeze-then-drop)
- [ ] T007 [P] Property `prop_earcon_band_duration` in `crates/player/tests/prop_earcon.rs` (invariants: every kind ≤500 ms, band 300–2500 Hz, all three of {timbre,rhythm,contour} distinct pairwise; PCM mono f32 @ 22050 Hz)
- [ ] T008 [P] Property `prop_persist_cadence` in `crates/player/tests/prop_persist.rs` (invariants: durable writes every 20 s ± tolerance (fixed 20s within 15-30s envelope) while Playing + on every listed event; cursor = last-spoken never buffered-ahead; worst-case loss ≤ one interval; replay ≤30 s interruption / ≤60 s kill)
- [ ] T009 [P] Property `prop_ingest_identity` in `crates/session/tests/prop_ingest.rs` (invariants: identical bytes at any path ⇒ identical blake3; edited byte ⇒ different hash ⇒ no resume; snapshot bytes are the synthesis input, never the URI)
- [ ] T010 [P] Contract test `contracts/diagnostic-trace` kinds in `crates/diag/tests/contract_trace.rs` (required kinds present: session/transport/per-section/per-rate/buffer/`FocusTransition`+measured `play_to_audio_ms`/`removal_pause_ms`/`replay_secs`/failure+`error`; `detail` ≤200 ch; failure/refusal without `error` FAILS; prose predicate FAILS)

### Wiring (starts only after T006–T010 are red)

- [ ] T011 [P] 20 s durable heartbeat scheduler skeleton in `src-tauri/src/lib.rs` (timer calls `BookmarkStore::put` while Playing; event-durable `put` call sites listed for US3; atomic single-row tx shape — parallel-safe with T012, different file; starts only after T006–T010 red)
- [ ] T012 [P] Diagnostic measured-latency `detail` keys (`play_to_audio_ms`, `removal_pause_ms`, `replay_secs`) in `crates/diag/src/writer.rs` (typed fields, ≤200 ch, hash refs only, never prose — parallel-safe with T011, different file; starts only after T006–T010 red)

**Checkpoint (TDD gate — Foundational global block)**: `cargo nextest run --workspace` shows T006–T010 FAIL (red) before T011–T012 and before any Phase 3+ implementation. Per-phase gates still hold per rule 1; this checkpoint adds a cross-phase block. Record the red output in the commit message.

---

## Phase 3: User Story 1 — Listen through a whole commute with the screen off (Priority: P1) 🎯 MVP

**Goal**: Picker → snapshot → screen-off continuous reading with lock-screen transport; first audio ≤3 s.
**Independent Test**: Sideload on reference phone, start `fixture-pt-2h.md`, lock 30 min, continuous audio with lock-screen controls only, zero unlocks (quickstart §3.1–3.2); host: T013–T016 green.

### Tests for US1 (BLOCKING — write first, FAIL before T017+; per-phase gate holds even in a global test wave) ⚠️

- [ ] T013 [P] [US1] Property `prop_skip_section_fallback` in `crates/player/tests/prop_skip.rs` (invariants: skip lands on next/previous section boundary; zero sections ⇒ exactly ±10 s; never mid-word anchor lost; clamp at doc ends)
- [ ] T014 [P] [US1] Integration `ingest → snapshot → fingerprint → parse` in `crates/session/tests/integration_ingest.rs` (pick `.md`/`.txt` ⇒ snapshot written, `fingerprint()` matches, `store.get` decides resume; moved/renamed re-pick ⇒ hash-match restore; revoked grant ⇒ re-prompt, never crash; non-`.md`/`.txt` pick ⇒ refused with `FormatUndeliverable` before synthesis; keep-current-only trim asserted: after new pick / Stop / document-end only the current snapshot remains; first-bookmark preservation: switching documents never deletes the first document's stored position)
- [ ] T015 [P] [US1] Contract test lock-screen payload (no Stop CONTROL; `stopped` STATE permitted) in `src-tauri/tests/test_surface_us1.rs` (payload matches `contracts/surface.md`: title+section+progress only, transport ∈ `{playing,paused,stopped}`, no `stop` control, no prose/times/rate-VALUE/voice keys — rate-step action permitted, no value display; see T038 Display-vs-payload)
- [ ] T016 [P] [US1] Play-to-audio bound harness in `crates/player/tests/test_play_latency.rs` (cold-start Play ⇒ first audible ≤3 s declared; measures and logs `play_to_audio_ms` into diag `detail`)

### Implementation for US1

- [ ] T017 [P] [US1] Kotlin picker + snapshot write in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` (per `contracts/ingest.md`: `ACTION_OPEN_DOCUMENT`, `text/markdown`+`text/plain` filter, persistable grant best-effort `try/catch(SecurityException)`, `ContentResolver` single read, `filesDir/snapshots/<hex>.{md,txt}` write/trim; keep-current-only (trim all but current on new pick / Stop / document-end); forward-only, zero position logic)
- [ ] T018 [P] [US1] `session ingest` CLI verb in `crates/session/src/cli.rs` (hash + `store.get` → `{fingerprint, resume: bool, offset}` JSON; Tauri IPC calls same fn; `--format json` default)
- [ ] T019 [US1] `open_document(bytes)` command reusing `play()` path in `src-tauri/src/lib.rs` (depends on T017, T018; snapshot bytes are the sole synthesis input; totals `total_n`+`total_units` fixed at session start; pure-offline, zero network; cold-start budget owned here — if T016/T051 miss ≤3 s, file perf fix under this task, never renegotiate the bound (spec Assumptions stop-the-line))
- [ ] T020 [P] [US1] Section-boundary skip in `crates/player/src/skip.rs` (next/previous boundary via `doc-structure`, ±10 s fallback when zero sections, boundary clamp)
- [ ] T021 [US1] Single minimal in-app screen transport + Stop in `src-tauri/src/status_view.rs` (+ frontend view `src-tauri/src/status_view.html`) (depends on T019; title/section/progress/speed/transport + dedicated Stop clearing stored position; never prose; read-only mirror owning no state)
- [ ] T022 [P] [US1] Foreground residency + screen-off continuity assertion in `android-shell/src/main/kotlin/app/mdreader/PlaybackService.kt` (reuse existing holder; assert audio continues locked; dismissal refused while playing — independent of T019/T021, different file, no depends; Foundational-green only)

**Checkpoint**: US1 host-green only (`cargo nextest` green for T013–T016); device steps (30-min locked session + 10/10 ≤3 s trials per quickstart §3.1–3.2) deferred to T051 — no on-device attestation here.

---

## Phase 4: User Story 2 — Survive interruptions without looking (Priority: P1)

**Goal**: Classified focus/routing outcomes + blind earcon/haptic confirmation; replay ≤30 s at sentence-start.
**Independent Test**: 10/10 per class (nav-duck vs call/alarm pause vs headphone ≤1 s vs permanent loss, no unlock, no spontaneous resume) + 10/10 blind per confirmation type sounds-on and -off (quickstart §3.3–3.4); host: T023–T026 green.

### Tests for US2 (BLOCKING — write first, FAIL before T027+; per-phase gate holds even in a global test wave) ⚠️

- [ ] T023 [P] [US2] Property `prop_focus_latest_wins` in `crates/player/tests/prop_focus8.rs` (invariants: 8 reasons exhaustive, no `_ =>`; transient-immediately-followed-by-permanent ⇒ final paused no-auto-resume with latest reason; permanent-loss-followed-by-transient ⇒ no-op (sticky permanent, transient ignored); `Deferred` ⇒ zero-audio hold with no reason; duck keeps `Playing` with `DUCK_GAIN 0.20 ±0.02` + restore without place loss, never owes rewind; call/alarm pause ≤1s)
- [ ] T024 [P] [US2] Property `prop_routing_pause_bound` in `crates/player/tests/prop_routing.rs` (invariants: `BecomingNoisy`/`DeviceDisappeared` ⇒ pause decision within normative ≤1000 ms per FR-006 (gating acceptance); 250 ms is an informative local engineering target mirroring plan.md Performance Goals, non-gating, logged only; never routes to speaker)
- [ ] T025 [P] [US2] Property `prop_resume_snap_cap` in `crates/player/tests/prop_resume.rs` (invariants: resume at containing-sentence-start + 5–10 s cue, never mid-word; total backward ≤30 s interruption / ≤60 s kill; persisted cursor is last-spoken)
- [ ] T026 [P] [US2] Earcon mirror 1:1 test in `crates/player/tests/test_earcon_mirror.rs` (every trigger ⇒ same-call lock-screen state change [giant transport glyph + progress + warning/failure banner snapshot assert] + `reason_ttl_ms=3000` for pause-demanding (distinct from `dismissal`); undeliverables (`VoiceUnavailable`/`FormatUndeliverable`/`DecodeFailed`) ⇒ never start + persistent line + `error` logged; earcon PCM never advances bookmark; ducked gain never quiets earcon)

### Implementation for US2

- [ ] T027 [P] [US2] 8-reason mapping + ≤8-word lines in `crates/player/src/interruption.rs` (`TransientPaused` split: `MAY_DUCK`→duck ~20% keep playing / non-duckable→pause; call/alarm pause ≤1s; `FocusDenied`→`Refused`, `FocusDelayed`→`Deferred` hold; `PermanentLoss`/`DeviceRemoved`→pause latest-wins; undeliverables (`VoiceUnavailable`/`FormatUndeliverable`/`DecodeFailed`)→never start + persistent line + `error` logged; no `Other`)
- [ ] T028 [P] [US2] Procedural earcon synthesis in `crates/player/src/earcon.rs` (grammar per `contracts/earcon.md` incl. failure E4 ~330 Hz transposition; mono f32 22050; constructors CLI-drivable)
- [ ] T029 [US2] Duck-exempt short-mix bypass in `crates/player/src/backend.rs` (depends on T028; earcon skips `SampleQueue::gain`, same stream/clock, files no focus request, dropped during duck/pause, re-fired post-GAIN once Playing restored, never moves bookmark)
- [ ] T030 [P] [US2] `player earcon` CLI verb in `crates/player/src/cli.rs` (`earcon <play|pause|skip-fwd|skip-back|warning|failure|rate> --format json` ⇒ PCM descriptor + duration/band asserts) (T030 alone in Wave U2-impl; T030b deferred to Wave U3/U4-impl — same file, different wave)
- [ ] T030b [P] [US4] → normative definition in Phase 6 Implementation for US4 (pointer only; same file as T030, never batched with T030; scheduled ONLY in Wave U3/U4-impl; starts only after T040b red AND T005 green AND Wave U2-impl done).
- [ ] T031 [P] [US2] Haptic patterns in `MediaMirror.publish()` in `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` (+ `VIBRATE` permission; depends on T027; Rust selects pattern-id per action/trigger, Kotlin renders it verbatim via distinct `VibrationEffect.createOneShot` with zero selection/computation (SC-035 holds); outside `STREAM_MUSIC`; visual glyph+progress banner covers fully-disabled-haptic)
- [ ] T032 [P] [US2] Sentence-snap resume path in `crates/player/src/rewind.rs` + `crates/player/src/resume.rs` (containing-sentence-start snap, mid-word forward-snap, cap-precedence, cue 5–10 s; degrade to raw cursor never doc start — independent of T027/T028/T030/T031, different files, no depends; requires only its phase Tests red + Foundational-green)

**Checkpoint**: US2 host-green only (T027–T032 green); device steps (10/10 per interruption class incl. FR-004 `0.20 ±0.02` + restore, 10/10 blind per confirmation) deferred to T051 — no on-device attestation here.

---

## Phase 5: User Story 3 — Never lose your place (Priority: P2)

**Goal**: Content-keyed durable position with 3 s deliberate offer vs explicit lock-screen resume; edit/stop ⇒ no resume.
**Independent Test**: 10/10 move+rename ⇒ offer; 10/10 edit ⇒ no resume; Stop ⇒ start at beginning; kill ⇒ ≤60 s replay (quickstart §3.5); host: T033–T034 green.

### Tests for US3 (BLOCKING — write first, FAIL before T035+; per-phase gate holds even in a global test wave) ⚠️

- [ ] T033 [P] [US3] Property `prop_bookmark_lifecycle` in `crates/session/tests/prop_bookmark.rs` (invariants: `put` atomic single-row; `Stopped|Completed` ⇒ deleted; `Paused`⇒deliberate 3 s offer silence-continues; `Interrupted`⇒explicit resume never auto-starts; content-change hash ⇒ miss ⇒ `0.0` no offer)
- [ ] T034 [P] [US3] Offer-window test in `crates/session/tests/test_offer3s.rs` (deliberate reopen ⇒ visual dialog 3 s continue-vs-start-over; kill/power-loss ⇒ paused + lock-screen resume control; `Stop` clears before next open)

### Implementation for US3

- [ ] T035 [US3] Event-durable `put` call sites in `crates/player/src/position.rs` + `src-tauri/src/lib.rs` (depends on T011; durable on pause, focus-loss, `BecomingNoisy`/`DeviceDisappeared`, section-change, Stop-then-`delete`, app-background, `DocumentEnd`; in-RAM ~1 s tick; non-durable per sentence/5 s snap-back source only)
- [ ] T036 [P] [US3] Re-hash-on-open lookup in `crates/session/src/store.rs` + `crates/session/src/offer.rs` (bytes → `fingerprint()` → `get` → end-kind-routed offer; identical-bytes-any-path shares position)
- [ ] T037 [P] [US3] Visual-dialog vs lock-screen-resume thin routing in `src-tauri/src/lib.rs` over the `crates/session/src/offer.rs` decision fn from T036 (deliberate ⇒ in-app 3 s dialog; unexpected ⇒ wait-paused + explicit lock-screen control; silence rule per FR-009; no offer logic in `lib.rs` — pure route-to-surface; CLI parity via `session ingest` verb (T018) exercising the same `offer.rs` fn; sync `scripts/expected-tests.txt` via T046)

**Checkpoint**: US3 host-green only (T033–T037 green); device steps (10/10 resume matrix + replay caps) deferred to T051 — no on-device attestation here.

---

## Phase 6: User Story 4 — A lock screen that respects attention (Priority: P2)

**Goal**: Purity (title+section+progress only), 7-rung persistent pitch-preserving rate, honest determinacy, bounded private full-trace record.
**Independent Test**: 10/10 purity inspections; rate persists + unpitched; diag within 30-day/10 MB with zero prose (quickstart §3.6–3.8); host: T038–T040b green.

### Tests for US4 (BLOCKING — write first, FAIL before T030b/T041+; per-phase gate holds even in a global test wave) ⚠️

- [ ] T038 [P] [US4] Purity property in `src-tauri/tests/test_purity.rs` (lock-screen + in-app snapshots across playing/paused/section-change/rate-change/interruption ⇒ display-projection keys ⊆ `{title, section_title, progress}` (full payload additionally carries transport ∈ `{playing,paused,stopped}` + reason/reason_ttl_ms/dismissal per contracts/surface.md Display-vs-payload; prohibited anywhere: prose/time/rate-VALUE/voice); zero prose/time/rate-VALUE/voice leaks — rate-step action permitted, no value display; dismissal-refused-while-playing assert; zero sections ⇒ `section_title == null` + indeterminate)
- [ ] T039 [P] [US4] Rate-ladder property in `crates/player/tests/prop_ladder.rs` (7 rungs `[0.75,1.0,1.25,1.5,1.75,2.0,4.0]` exact, one step saturating, `settings.last_rate` persists across sessions/documents, pitch within `PITCH_TOLERANCE_CENTS=50` every rung incl. 2.0x→4.0x)
- [ ] T040 [P] [US4] Diag no-prose + bounds property in `crates/diag/tests/prop_trace.rs` (30-day + 10 MB oldest-first with `Trim` before retained, `Gap` on full with `Ok`; `ProseInDetail`/`DetailTooLong`/`FailureWithoutError`/`RefusalWithoutError` scan violations trip; zero egress — no network call sites (diag-crate scope; app-wide covered by T047))
- [ ] T040b [P] [US4] CLI contract test for progress verb in `crates/player/tests/test_progress_cli.rs` (--consumed/--total/--k/--n and --indeterminate JSON shapes, frozen label "Section {k} of {N} — {p}% (content-offset)", zero-sections indeterminate+null, --format json; FAIL-red before T030b/T041). T030b impl (normative in Phase 6; pointer in Phase 4) gated here by T040b red + T005 green.

### Implementation for US4

- [ ] T030b [P] [US4] [normative home — pointer in Phase 4] `player progress` CLI verb in `crates/player/src/cli.rs` (`progress (--consumed <u> --total <u> --k <k> --n <n> | --indeterminate) --format json` ⇒ `Progress::{Determinate,Indeterminate}` JSON + frozen label `"Section {k} of {N} — {p}% (content-offset)"` per `contracts/surface.md`; determinate-iff-known-at-start, freeze-then-drop, zero-sections ⇒ indeterminate + null title; Tauri IPC calls same fn; covered by T040b test_progress_cli.rs + existing `prop_progress_fraction` T006; sync `scripts/expected-tests.txt` via T046) (same file as T030 — separate wave from T030, never batched with T030; starts only after T040b red AND T005 green AND Wave U2-impl done, even if U2 tests are red first; flag shape normative here, mirrored in contracts/surface.md). No Principle-II exemption taken (plan.md decision recorded 2026-10-13 remediation).
- [ ] T041 [P] [US4] Content-offset progress rendering in `crates/player/src/surface.rs` (depends on T005; bar = `consumed_units/total_units`, label frozen template `"Section {k} of {N} — {p}% (content-offset)"`; determinate iff both totals known at session start; freeze-then-drop; audio-duration never feeds it; no seek (no seek control in this build)) (parallel-safe: distinct file crates/player/src/surface.rs vs src-tauri/src/surface.rs; use explicit-path subagents to avoid basename confusion; may run in Wave U3/U4-impl)
- [ ] T042 [US4] Rate stepper + persistence in `crates/player/src/rate.rs` + `crates/session/src/store.rs` (in-app stepper + lock-screen rate-step action without value display — no rate leak, earcon+haptic confirm; engine-contract stretch, `last_rate` read-at-start/write-on-change; acceptance: locked rate-step works, no numeric leak; serializes after T036: both touch crates/session/src/store.rs; runs after T036 in same wave)
- [ ] T043 [P] [US4] Reason-expiry + dismissal in `src-tauri/src/surface.rs` + `src-tauri/src/failure.rs` (pause-demanding 3 s `reason_ttl_ms=3000` distinct from `dismissal` ∈ `{non_dismissable_while_playing,dismissable}`; undeliverables (`VoiceUnavailable`/`FormatUndeliverable`/`DecodeFailed`) ⇒ never start + persistent line + `error` logged; non-dismissable while playing; ≤8-word lines; failures never quote document text)
- [ ] T044 [P] [US4] DESIGN-SONA token styling of MediaStyle + in-app grid in `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` + `src-tauri/src/status_view.html` (title `#F2F0E6`, section `#A9B8CE`, progress white/`#55688F`; yellow `#FFD400` warning-only, red `#FF4D63` failure-only, green `#5FD0A0` prohibited; no marquee/body/rate-display/voice — no rate value display)

**Checkpoint**: US4 host-green only after T036 green (T042 serialization) (T030b/T038–T044 green); device steps (purity/rate/record trials) deferred to T051 — no on-device attestation here.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Docs, hygiene, and scope guards. No behavior changes except T047 wiring already listed.

- [ ] T045 [P] `docs/` + `tests/device/README.md` trial procedures update (004 commute/blind/resume/purity/rate/record per quickstart §3; reference-phone-only scope; out-of-scope absence checklist: zero network call sites app-wide (enforces FR-015 voice ban plus diag no-egress; grep `reqwest|hyper|ureq|tonic|surf|isahc|minreq|curl|tokio::net|std::net::Tcp` + `cargo tree -i` network-crate reverse-deps + `cargo deny check bans sources` + Cargo.lock audit), exactly one voice dir, no library view, lock-screen no Stop CONTROL, no Auto/head-unit service, no in-app diag surface, no distribution artifacts)
- [ ] T046 [P] `scripts/expected-tests.txt` target-list sync (new `prop_*`/contract suites recorded so gate 2 fails closed on missing *or* unexpected targets)
- [ ] T047 [P] Out-of-scope absence guard test in `src-tauri/tests/test_noscope.rs` (asserts zero network call sites app-wide (any crate; voice + diag + other; grep `reqwest|hyper|ureq|tonic|surf|isahc|minreq|curl|tokio::net|std::net::Tcp` + `cargo tree -i` network-crate reverse-deps + `cargo deny check bans sources` + Cargo.lock audit), not voice-only — enforces FR-015 voice ban plus diag no-egress, exactly one bundled voice dir, no library view, lock-screen no Stop CONTROL, no Auto/head-unit service, no in-app diag surface, no distribution artifacts + naming-boundary grep: no `mdreader` in user-visible strings, Ressoa-external only)

---

## Phase 8: Verification (ATTESTS — only this phase declares success)

**Purpose**: Deterministic gates + on-device trials. ONLY these tasks attest completeness; agent claims attest nothing. US-phase checkpoints attest host-green; device criteria named there are collected once here (T051, shared reference phone), not per-checkpoint device runs.

- [ ] T048 Host gates 1–7 + gate-8-shell via `./scripts/verify.sh` (format, nextest vs `expected-tests.txt`, doc, lint-drift, supply-chain `cargo deny` + `cargo audit`, deny-list, closed-set no `_ =>`, pipeline shell-bounds = `gate-8-shell: PASS`) + attempt gate-8-full discharge with attached evidence artifacts (byte-budget + SSML + sentence-alignment logs/outputs, paths recorded); only if synthesis SC-044 remains open record `gate-8-full: DEFERRED (owner=synthesis, reason=SC-044-open)` per constitution Exceptions Log 8 — MUST NOT claim constitution gate 8 PASS without full discharge — record exit status + log + evidence paths
- [ ] T049 Manifest + shell-purity via `./scripts/check-manifest.sh` + `./scripts/check-shell-purity.sh` against `src-tauri/gen/android/app/src/main/AndroidManifest.xml` (service `mediaPlayback` + `FOREGROUND_SERVICE*` + `VIBRATE` + `POST_NOTIFICATIONS` + minSdk 29; extend `scripts/check-manifest.sh` with `require_pattern 'the VIBRATE permission (FR-016 / T031)' 'android\.permission\.VIBRATE"'` (extension reviewed with the T031 haptics wave); this task then executes both scripts; zero transport/rate/position logic in `android-shell/`; naming-boundary: no `mdreader` in user-visible strings)
- [ ] T050 Android artifact via `cargo tauri android build --target aarch64` (gate 9) + maintainer sideload `adb install -r <apk>` — record APK hash
- [ ] T051 On-device SC-001…SC-008 trials per `quickstart.md` §3 on the reference phone only (30-min locked commute at every speed; 10/10 ≤3 s; miss ⇒ stop-the-line for distribution, no bound renegotiation; 10/10 per interruption class; 10/10 blind per confirmation incl. rate-step sounds-on/off; 10/10 resume matrix + replay caps + 3 s offer-window timing; 10/10 purity + 10/10 dismissal-refused-while-playing; rate persistence + unpitched; diag `adb pull` bounds + zero prose; + quickstart §3 item 11 lock-screen privacy-hide-contents manual inspection: enable platform hide-contents → app obeys, contents hidden, no workaround attempted) — record per-SC 10/10 sheets + §3.9 inspection note
- [ ] T052 Diagnostic-record audit via `diag scan` CLI on the pulled JSONL (schema conformance + negative control, zero prose, `error` on every failure/refusal, `Trim`/`Gap` markers sane) — attach scan output

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies — start immediately. T001, T002a–T002c, T002d–T002f are `[P]` (Wave S); T003–T005 are `[P]` (Wave S-impl) (different files); T002d–T002f red before T003–T005.
- **Foundational (Phase 2)**: depends on Phase 1 skeletons. T006–T010 `[P]` run as **Wave F1** (5 subagents, one per test file); T011–T012 `[P]` run as **Wave F2** (2 subagents, different files) after red is shown.
- **US phases (3–6)**: ALL depend on Foundational red→green cycle. Within each US phase: `### Tests` BLOCK all implementation in that phase. Test tasks MUST run as one global background wave; NO implementation in ANY phase starts until the global test-red gate is met (rule 1).
- **Polish (Phase 7)**: depends on all desired US phases complete. T045–T047 `[P]`.
- **Verification (Phase 8)**: depends on everything; runs SEQUENTIALLY (T048→T052), never parallel (shared device + shared target dir).

### User Story Dependencies

- **US1 (P1)**: after Foundational — no deps on other stories. MVP = Setup + Foundational + US1.
- **US2 (P1)**: after Foundational — may reuse US1 surfaces but independently testable (own `prop_*` + mirror tests).
- **US3 (P2)**: after Foundational — reuses ingest hash + `put` points; independently testable via bookmark/offer tests.
- **US4 (P2)**: after Foundational + T036 (store.rs serialization for T042) — reuses `Progress`/rate/diag shapes; independently testable via purity/ladder/trace tests once T036 green; T042 serializes after T036.

### Within Each User Story

Tests (BLOCKING, FAIL-red) → types/enums → services/paths → surfaces → integration. Models before services before endpoints before integration. Story checkpoint before next priority.

**Same-file serialize guard (binding; `[P]` means distinct-files-within-wave — same-file `[P]`s MUST be in different waves, never batched)**: `src-tauri/src/lib.rs`: T011→T019→T037→T035 (T011 `[P]` Wave F2, T019 non-`[P]` Wave U1-impl, T037 `[P]` Wave U3/U4-impl, T035 non-`[P]` after Wave U3/U4-impl); `crates/player/src/cli.rs`: T030 then T030b (both `[P]`, different waves — T030 Wave U2, T030b Wave U3/U4-impl, never batched); `crates/session/src/store.rs`: T036→T042 (T036 `[P]`, T042 non-`[P]`, same wave serial). T041 is `[P]` (parallel-safe: files are DISTINCT `crates/player/src/surface.rs` vs `src-tauri/src/surface.rs` — use explicit-path subagents to avoid basename confusion; may run in Wave U3/U4-impl). T011/T012, T022, T032 above are the only missed-`[P]` additions; no other non-`[P]` task is promoted by this fix. T011 shares `src-tauri/src/lib.rs` with T019/T035/T037 — phase ordering (F2 (T011) → U1 (T019) → U3 (T037→T035)) serializes them; keep atomic commits per rule 5 to avoid merge conflicts.

### Parallel Opportunities (subagent waves)

Wave S + F1 + U1-tests + U2-tests + U3/U4-tests MAY combine into one global test wave of up to 22 subagents (3 T002d–T002f + 5 T006–T010 + 4 T013–T016 + 4 T023–T026 + 2 T033–T034 + 4 T038–T040b); impl waves (S-impl, F2, U1-impl, U2-impl, U3/U4-impl) all wait for global red. Per-phase gates still hold inside the global gate — no implementation in a phase starts until that phase's own Tests are all red AND the global test-red gate is met.

- **Wave S**: T001 + T002a, T002b, T002c, T002d, T002e, T002f (7 subagents total, different files; T002c workspace-wide — single subagent, runs first in Wave S; T001 fixture independent); gate: T002d–T002f red before T003+.
- **Wave S-impl**: T003, T004, T005 (3 subagents, different files) after S-tests red.
- **Wave F1 (tests)**: T006, T007, T008, T009, T010 (5 subagents, background OK).
- **Wave F2**: T011, T012 `[P]` (2 subagents, different files) after F1 red.
- **Wave U1-tests**: T013, T014, T015, T016 (4 subagents) — MAY run together with U2/U3/U4 tests as the U-story subset (14 = 4 + 4 + 6 subagents) of the one global background wave of up to 22 subagents once Foundational is green (tasks sharing `player` context may reuse one subagent per crate); U1 impl still waits for T013–T016 red.
- **Wave U1-impl**: T017, T018, T020, T022 `[P]` (4 subagents, different files) only after T013–T016 red; T019, T021 follow deps (T019 depends T017+T018; T021 depends T019).
- **Wave U2**: T023, T024, T025, T026 `[P]` (4 subagents, MAY join the global test wave); then T027, T028, T030 (single file; T030 alone in U2, T030b deferred to Wave U3/U4-impl), T031, T032 `[P]` (5 subagents, different files) only after T023–T026 red; T029 follows dep (depends T028).
- **Wave U3/U4-tests**: T033, T034, T038, T039, T040, T040b `[P]` (6 subagents, MAY join the global test wave).
- **Wave U3/U4-impl**: T036, T037, T041, T043, T044, T030b `[P]` (6 subagents, different files WITHIN this wave — cross-wave same-file pairs T011/T037 (`lib.rs`), T030/T030b (`cli.rs`) serialize per guard below, never batched; T030b MUST NOT start until T040b red AND T005 green; T041 parallel-safe with explicit-path subagents: `crates/player/src/surface.rs` vs `src-tauri/src/surface.rs`) + T042 follows T036 (same store.rs, serial); T035, T042 follow deps and stay non-`[P]` (T035 serializes after T037: both touch `src-tauri/src/lib.rs`; T042 serializes after T036: both touch `crates/session/src/store.rs`; T030b follows T040b+T005).
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

# Wave U1-impl — 4 subagents, different files:
Subagent A: "T017 DocIngest.kt picker+snapshot (contracts/ingest.md)"
Subagent B: "T018 session cli.rs ingest verb"
Subagent C: "T020 player skip.rs section ±10 s fallback"
Subagent D: "T022 PlaybackService.kt residency (independent, Foundational-green only)"
# Then: T019 (depends A,B) → T021. Commit atomically per file (hook §5).
```

## Parallel Example: User Story 2

```bash
# Wave U2-tests — 4 subagents:
Subagent A: "T023 prop_focus8 in crates/player/tests/prop_focus8.rs"
Subagent B: "T024 prop_routing in crates/player/tests/prop_routing.rs"
Subagent C: "T025 prop_resume in crates/player/tests/prop_resume.rs"
Subagent D: "T026 test_earcon_mirror in crates/player/tests/test_earcon_mirror.rs"
# Wave U2-impl[P] — 5 subagents, different files:
Subagent A: "T027 interruption.rs 8-reason mapping"
Subagent B: "T028 earcon.rs procedural synthesis"
Subagent C: "T030 player cli.rs earcon verb (single file)"
Subagent D: "T031 MediaMirror.kt haptics"
Subagent E: "T032 rewind.rs + resume.rs sentence-snap (independent)"
# Then: T029 (depends T028). T035/T042 stay non-[P] per serialize guard.
# T030b runs in Wave U3/U4-impl after T040b red + T005 green.
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 (T001–T005 incl. T002a/T002b/T002c/T002d–T002f) → Phase 2 tests red (T006–T010) → wiring (T011–T012).
2. US1 tests red (T013–T016) → US1 impl (T017–T022).
3. **STOP and VALIDATE**: host green + 30-min locked commute + 10/10 ≤3 s trials. Sideloadable MVP.

### Incremental Delivery

1. Foundation ready → US1 (MVP) → US2 (interruptions + blind) → US3 (resume) → US4 (purity/rate/record) → Polish → Verification.
2. Each story adds value without breaking previous stories (checkpoints enforce).

### Subagent Team Strategy

1. Whole team: Setup Wave S (7 subagents: T001 + T002a/b/c/d/e/f) → S-impl (T003–T005) + Foundational Wave F1 (up to 5 background subagents).
2. Once Foundational green: fan out per wave above — Developer/Subagent per `[P]` task; shared-crate tasks reuse one subagent to preserve context.
3. Verification (Phase 8) single-threaded: one agent, sequential, evidence attached. Its output — not any implementer's claim — is the success verdict.

---

## Notes

- `[P]` = parallelizable (different files WITHIN the same wave, no dependencies on incomplete tasks; same file ONLY across different waves/phases with explicit serialize order per line 16, never batched in one wave) → subagent wave, background where viable (binding definition: line 16).
- `[USn]` maps task to spec.md story for traceability.
- Quote data-model constraints verbatim in code (e.g. `Progress` fixed-at-start, `detail ≤200 ch`, `error` required on failure *and* refusal, `TransientPaused` duck-vs-pause split, zero-sections ⇒ indeterminate + null title).
- `DecodeFailed` rides `EngineError::Unreachable` on the wire (`decode_failed` in diag only) — no `EngineError` edit, no semver bump.
- New public APIs require rustdoc (`missing_docs` deny, gate 3).
- After EVERY file edit: atomic conventional commit with detailed summary (rule 5). Suggested: `test(004): T013 prop_skip red` → `feat(004): T020 section skip` → … → `verify(004): T048 gates-1-7-plus-shell PASS`.
- Stop at any checkpoint to validate the story independently; measurements that miss bounds stop the line for any later distribution decision (spec Assumptions).
- **Analyze clarifications (non-behavioral, no scope change)**:
  - D1 — FR-005/FR-006 split deliberate: FR-005 is the outcome+reason mapping (duck vs pause + 3 s reason + never-auto-resume, closed 8 reasons); FR-006 is the timing bound (≤1 s) for the DeviceRemoved subset of FR-005. Tasks T027/T031 implement the mapping; T024 + routing tests attest the bound. Not a duplication to merge.
  - D2 — FR-007 single-source: tiered durability + snap-back + replay caps live only in FR-007 (20 s fixed heartbeat within 15–30 s envelope, event-durable writes, sentence-start + 5–10 s cue, ≤30 s interruption / ≤60 s kill). Tasks T008/T025/T032/T035 all cite FR-007 as single source; Edge-Case prose does not restate it.
  - A1 — glyph snapshot assertion: FR-016 "glanceable lock-screen state change" in tasks means giant transport glyph + progress + warning/failure banner asserted by T026 mirror test (same-call state change, `reason_ttl_ms=3000` for pause-demanding) and T038 purity keys ⊆ `{title, section_title, progress}`; haptic path covers sounds-off/DND. No spoken announcement.
  - A3 — trial cites FR-004 numbers: US2 duck trials attest FR-004 exactly (`DUCK_GAIN 0.20 ±0.02`, keep `Playing`, restore without place loss, never owes rewind per T023); "clearly quieter" is not judged by ear.
  - U2 — Principle-II decision (2026-10-13 remediation): no exemption; player progress CLI (T030b) + contract test (T040b) required. T026/T034/T038 remain example/contract snapshot tests because they assert fixed state-snapshot/contract shapes (mirror 1:1, 3 s offer window, purity key subset), with prop_* complementing where applicable per rule 2. T026/T034/T038 snapshot tests are complemented by prop_* where applicable: T026 by prop_earcon_band_duration (T007) + prop_focus8 (T023); T034 by prop_bookmark_lifecycle (T033); T038 by prop_progress_fraction (T006) for determinacy subset. Timing/state invariants without prop cover (3 s TTL exact, glyph snapshot) remain snapshot-asserted with 10/10 on-device trials (T051) as complement — recorded here as rule-2 satisfaction, not exemption.
  - C2 — CLI mapping for snapshot tests: T026 mirror via player earcon CLI (T030); T034 offer via session ingest CLI (T018); T038 purity via player progress CLI (T030b) + surface contract tests; no Principle-II exemption taken.
  - F2 — contracts/surface.md token edits serialize with T044 wave (T044 designs the DESIGN-SONA tokens in contracts/surface.md + MediaMirror + status_view.html; same file edits serialized with T044 wave).
  - G1 — FR-005/FR-006 split confirmed deliberate: no merge needed; FR-005 = outcome+reason mapping (D1 at line 297); FR-006 = timing bound ≤1 s for DeviceRemoved subset; T024/T027/T031 cover separately.
  - D1 — constitution compliance confirmed: `.specify/memory/constitution.md` v1.5.0 gates (Rust rigor, closed-set gate 7, supply-chain gates 5–6, single playback authority, manual manifest) already present in plan.md; tasks.md notes confirm no relaxation (see execution rules 1–5 + Notes D1/A3/U2/C2).
  - A1 — FR-002/FR-006 timing split and FR-004/FR-005 duck mapping overlap are deliberate; see spec.md clarifications (2026-10-13 session) and data-model.md interruption table.
