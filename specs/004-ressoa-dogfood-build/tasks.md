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

- [X] T001 [P] Create Portuguese acceptance fixture `tests/device/fixtures/fixture-pt-2h.md` + co-located `fixture-pt-2h.README.md` (Markdown with headings + numbers/currency, ~2 h at 1.0x; README records estimated duration at 1.0x from word-count + 4.0x-sustains-30-min computation; SC-001 input for every speed incl. 4.0x; both files committed)
- [X] T001b [P] Fix oversize-document byte cap: measure `fixture-pt-2h.md` byte size, set `MAX_DOC_BYTES = <fixture_bytes × 1.5>` (headroom for ~3 h at 1.0x) as a named constant in `crates/player/src/interruption.rs` (or `crates/session/src/ingest` module); document the value + measurement method in the fixture README; wire the cap check into the FR-018 ingest refusal path (FormatUndeliverable when exceeded) — depends on T001
- [X] T002a [P] Add snapshot `.gitignore` exception in repo-root `.gitignore` (ignore repo-local `/snapshots/` content — on-device `filesDir/snapshots` never enters git; no logic yet — parallel-safe with T002b, different file)
- [X] T002b [P] Add snapshot directory convention const in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` (header const `SNAPSHOT_DIR="filesDir/snapshots"`, `<hex>.{md,txt}` naming; no logic yet — parallel-safe with T002a, different file)
- [X] T002c [P] Verify workspace `proptest` dev-dependency in `Cargo.toml` (`[workspace.dependencies]` pin + `proptest = { workspace = true }` in `[dev-dependencies]` of each member with `prop_*` suites; re-verify latest stable on crates.io at implementation time and record exact version in atomic commit msg per Principle IV; `Cargo.lock` committed + `--locked` build; `deny.toml` `[sources]` crates.io-only; `cargo deny check advisories licenses bans sources` + `cargo audit` clean; `[lints] workspace = true` intact, no `Cargo.toml`/`clippy.toml` weakening, no `Other`/`Unknown`, no `_ =>`; parallel-safe with T002a/T002b/T002d–T002f, different files) (workspace-wide — single subagent, runs first in Wave S)

### Setup Tests (BLOCKING — write first, FAIL before T003+) ⚠️

- [X] T002d [P] Closed-set shape test for `EarconKind` in `crates/player/tests/test_setup_earcon_shape.rs` (asserts `EarconKind` has exactly `Play|Pause|SkipFwd|SkipBack|Warning|Failure|RateStep`, no `Other`/`Unknown`, exhaustive `match` with no `_ =>`; FAIL-red before T003)
- [X] T002e [P] Closed-set shape test for interruption reasons in `crates/player/tests/test_setup_interruption_shape.rs` (asserts 8-reason enum `FocusDenied|FocusDelayed|PermanentLoss|TransientPaused|DeviceRemoved|VoiceUnavailable|FormatUndeliverable|DecodeFailed`, no `Other`, exhaustive `reason_line()` stub ≤8 words with no `_ =>`; FAIL-red before T004)
- [X] T002f [P] Type-shape test for `Progress` payload in `crates/player/tests/test_setup_progress_shape.rs` (asserts `Determinate{consumed_units,total_units,k,n} | Indeterminate` shape + fixed-at-session-start flag, `None`-section + dismissal rules; FAIL-red before T005)

### Scaffolds (start only after T002d–T002f are red)

- [X] T003 [P] Scaffold `crates/player/src/earcon.rs` module declaration + closed `EarconKind` enum (`Play|Pause|SkipFwd|SkipBack|Warning|Failure|RateStep`, no `Other`, wired in `crates/player/src/lib.rs`)
- [X] T004 [P] Scaffold `crates/player/src/interruption.rs` closed 8-reason enum (`FocusDenied|FocusDelayed|PermanentLoss|TransientPaused|DeviceRemoved|VoiceUnavailable|FormatUndeliverable|DecodeFailed`, no `Other`, exhaustive `reason_line()` stub ≤8 words) wired in `crates/player/src/lib.rs`
- [X] T005 [P] Change `Progress` payload type in `crates/player/src/surface.rs` to content-offset shape (`Determinate{consumed_units,total_units,k,n} | Indeterminate`, fixed-at-session-start flag; keep `None`-section + dismissal rules)

**Note**: T002 split into T002a/T002b (different files, both `[P]`); T002c workspace `proptest` verification + T002d–T002f Setup Tests added without renumbering; all later IDs T003–T052 unchanged.

**Checkpoint (TDD gate — Setup)**: `cargo nextest run --workspace` shows T002d–T002f FAIL (red) before T003–T005; then `cargo check --workspace` green; skeletons compile; zero behavior changed. Record the red output in the commit message. Global gate: T003+ waits for ALL phase Tests red, not just T002d–f.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Test harness + shared wiring points that MUST complete before ANY user story. **⚠️ CRITICAL: no US-phase implementation starts until T006–T012 are all FAIL-red.**

### Tests (BLOCKING — write first, show FAIL)

- [X] T006 [P] Property `prop_progress_fraction` in `crates/player/tests/prop_progress.rs` (invariants: `0 ≤ consumed ≤ total`; determinate fixed at session start, never flips mid-session; zero sections ⇒ indeterminate; re-parse never moves backward — freeze-then-drop)
- [X] T007 [P] Property `prop_earcon_band_duration` in `crates/player/tests/prop_earcon.rs` (invariants: every kind ≤500 ms, band 300–2500 Hz, all three of {timbre,rhythm,contour} distinct pairwise; PCM mono f32 @ 22050 Hz)
- [X] T008 [P] Property `prop_persist_cadence` in `crates/player/tests/prop_persist.rs` (invariants: durable writes every 20 s ± tolerance (fixed 20s within 15-30s envelope) while Playing + on every listed event; cursor = last-spoken never buffered-ahead; worst-case loss ≤ one interval; replay ≤30 s interruption / ≤60 s kill)
- [X] T009 [P] Property `prop_ingest_identity` in `crates/session/tests/prop_ingest.rs` (invariants: identical bytes at any path ⇒ identical blake3; edited byte ⇒ different hash ⇒ no resume; snapshot bytes are the synthesis input, never the URI)
- [X] T010 [P] Contract test `contracts/diagnostic-trace` kinds in `crates/diag/tests/contract_trace.rs` (required kinds present: session/transport/per-section/per-rate/buffer/`FocusTransition`+measured `play_to_audio_ms`/`removal_pause_ms`/`replay_secs`/failure+`error`; `detail` ≤200 ch; failure/refusal without `error` FAILS; prose predicate FAILS)

### Wiring (starts only after T006–T010 are red)

- [x] T011 [P] 20 s durable heartbeat scheduler skeleton in `src-tauri/src/lib.rs` (timer calls `BookmarkStore::put` while Playing; event-durable `put` call sites listed for US3; atomic single-row tx shape — parallel-safe with T012, different file; starts only after T006–T010 red)
- [X] T012 [P] Diagnostic measured-latency `detail` keys (`play_to_audio_ms`, `removal_pause_ms`, `replay_secs`) in `crates/diag/src/writer.rs` (typed fields, ≤200 ch, hash refs only, never prose — parallel-safe with T011, different file; starts only after T006–T010 red)

**Checkpoint (TDD gate — Foundational global block)**: `cargo nextest run --workspace` shows T006–T010 FAIL (red) before T011–T012 and before any Phase 3+ implementation. Per-phase gates still hold per rule 1; this checkpoint adds a cross-phase block. Record the red output in the commit message.

---

## Phase 3: User Story 1 — Listen through a whole commute with the screen off (Priority: P1) 🎯 MVP

**Goal**: Picker → snapshot → screen-off continuous reading with lock-screen transport; first audio ≤3 s.
**Independent Test**: Sideload on reference phone, start `fixture-pt-2h.md`, lock 30 min, continuous audio with lock-screen controls only, zero unlocks (quickstart §3.1–3.2); host: T013–T016 green.

### Tests for US1 (BLOCKING — write first, FAIL before T017+; per-phase gate holds even in a global test wave) ⚠️

- [X] T013 [P] [US1] Property `prop_skip_section_fallback` in `crates/player/tests/prop_skip.rs` (invariants: skip lands on next/previous section boundary; zero sections ⇒ exactly ±10 s; never mid-word anchor lost; clamp at doc ends)
- [X] T014 [P] [US1] Integration `ingest → snapshot → fingerprint → parse` in `crates/session/tests/integration_ingest.rs` (pick `.md`/`.txt` ⇒ snapshot written, `fingerprint()` matches, `store.get` decides resume; moved/renamed re-pick ⇒ hash-match restore; revoked grant ⇒ re-prompt, never crash; non-`.md`/`.txt` pick ⇒ refused with `FormatUndeliverable` before synthesis; keep-current-only trim asserted: after new pick / Stop / document-end only the current snapshot remains; first-bookmark preservation: switching documents never deletes the first document's stored position)
- [X] T015 [P] [US1] Contract test lock-screen payload (no Stop CONTROL; `stopped` STATE permitted) in `src-tauri/tests/test_surface_us1.rs` (payload matches `contracts/surface.md`: title+section+progress only, transport ∈ `{playing,paused,stopped}`, no `stop` control, no prose/times/rate-VALUE/voice keys — rate-step action permitted, no value display; see T038 Display-vs-payload)
- [X] T016 [P] [US1] Play-to-audio bound harness in `crates/player/tests/test_play_latency.rs` (cold-start Play ⇒ first audible ≤3 s declared; measures and logs `play_to_audio_ms` into diag `detail`)

### Implementation for US1

- [X] T017 [P] [US1] Kotlin picker + snapshot write in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` (per `contracts/ingest.md`: `ACTION_OPEN_DOCUMENT`, `text/markdown`+`text/plain` filter, persistable grant best-effort `try/catch(SecurityException)`, `ContentResolver` single read, `filesDir/snapshots/<hex>.{md,txt}` write/trim; keep-current-only (trim all but current on new pick / Stop / document-end); forward-only, zero position logic)
- [X] T018 [P] [US1] `session ingest` CLI verb in `crates/session/src/cli.rs` (hash + `store.get` → `{fingerprint, resume: bool, offset}` JSON; Tauri IPC calls same fn; `--format json` default)
- [X] T019 [US1] `open_document(bytes)` command reusing `play()` path in `src-tauri/src/lib.rs` (depends on T017, T018; snapshot bytes are the sole synthesis input; totals `total_n`+`total_units` fixed at session start; pure-offline, zero network; cold-start budget owned here — if T016/T051 miss ≤3 s, file perf fix under this task, never renegotiate the bound (spec Assumptions stop-the-line))
- [X] T020 [P] [US1] Section-boundary skip in `crates/player/src/skip.rs` (next/previous boundary via `doc-structure`, ±10 s fallback when zero sections, boundary clamp)
- [X] T021 [US1] Single minimal in-app screen transport + Stop in `src-tauri/src/status_view.rs` (+ frontend view `src-tauri/src/status_view.html`) (depends on T019; title/section/progress/speed/transport + dedicated Stop clearing stored position; never prose; read-only mirror owning no state)
- [X] T022 [P] [US1] Foreground residency + screen-off continuity assertion in `android-shell/src/main/kotlin/app/mdreader/PlaybackService.kt` (reuse existing holder; assert audio continues locked; dismissal refused while playing — independent of T019/T021, different file, no depends; Foundational-green only)

**Checkpoint**: US1 host-green only (`cargo nextest` green for T013–T016); device steps (30-min locked session + 10/10 ≤3 s trials per quickstart §3.1–3.2) deferred to T051 — no on-device attestation here.

---

## Phase 4: User Story 2 — Survive interruptions without looking (Priority: P1)

**Goal**: Classified focus/routing outcomes + blind earcon/haptic confirmation; replay ≤30 s at sentence-start.
**Independent Test**: 10/10 per class (nav-duck vs call/alarm pause vs headphone ≤1 s vs permanent loss, no unlock, no spontaneous resume) + 10/10 blind per confirmation type sounds-on and -off (quickstart §3.3–3.4); host: T023–T026 green.

### Tests for US2 (BLOCKING — write first, FAIL before T027+; per-phase gate holds even in a global test wave) ⚠️

- [X] T023 [P] [US2] Property `prop_focus_latest_wins` in `crates/player/tests/prop_focus8.rs` (invariants: 8 reasons exhaustive, no `_ =>`; transient-immediately-followed-by-permanent ⇒ final paused no-auto-resume with latest reason; permanent-loss-followed-by-transient ⇒ no-op (sticky permanent, transient ignored); `Deferred` ⇒ zero-audio hold with no reason; duck keeps `Playing` with `DUCK_GAIN 0.20 ±0.02` + restore without place loss, never owes rewind; call/alarm pause ≤1s)
- [X] T024 [P] [US2] Property `prop_routing_pause_bound` in `crates/player/tests/prop_routing.rs` (invariants: `BecomingNoisy`/`DeviceDisappeared` ⇒ pause decision within normative ≤1000 ms per FR-006 (gating acceptance); 250 ms is an informative local engineering target mirroring plan.md Performance Goals, non-gating, logged only; never routes to speaker)
- [X] T025 [P] [US2] Property `prop_resume_snap_cap` in `crates/player/tests/prop_resume.rs` (invariants: resume at containing-sentence-start + 5–10 s cue, never mid-word; total backward ≤30 s interruption / ≤60 s kill; persisted cursor is last-spoken)
- [X] T026 [P] [US2] Earcon mirror 1:1 test in `crates/player/tests/test_earcon_mirror.rs` (every trigger ⇒ same-call lock-screen state change [giant transport glyph + progress + warning/failure banner snapshot assert] + `reason_ttl_ms=3000` for pause-demanding (distinct from `dismissal`); undeliverables (`VoiceUnavailable`/`FormatUndeliverable`/`DecodeFailed`) ⇒ never start + persistent line + `error` logged; earcon PCM never advances bookmark; ducked gain never quiets earcon)

### Implementation for US2

- [X] T027 [P] [US2] 8-reason mapping + ≤8-word lines in `crates/player/src/interruption.rs` (`TransientPaused` split: `MAY_DUCK`→duck ~20% keep playing / non-duckable→pause; call/alarm pause ≤1s; `FocusDenied`→`Refused`, `FocusDelayed`→`Deferred` hold; `PermanentLoss`/`DeviceRemoved`→pause latest-wins; undeliverables (`VoiceUnavailable`/`FormatUndeliverable`/`DecodeFailed`)→never start + persistent line + `error` logged; no `Other`)
- [X] T028 [P] [US2] Procedural earcon synthesis in `crates/player/src/earcon.rs` (grammar per `contracts/earcon.md` incl. failure E4 ~330 Hz transposition; mono f32 22050; constructors CLI-drivable)
- [X] T029 [US2] Duck-exempt short-mix bypass in `crates/player/src/backend.rs` (depends on T028; earcon skips `SampleQueue::gain`, same stream/clock, files no focus request, dropped during duck/pause, re-fired post-GAIN once Playing restored, never moves bookmark)
- [X] T030 [P] [US2] `player earcon` CLI verb in `crates/player/src/cli.rs` (`earcon <play|pause|skip-fwd|skip-back|warning|failure|rate> --format json` ⇒ PCM descriptor + duration/band asserts) (T030 alone in Wave U2-impl; T030b deferred to Wave U3/U4-impl — same file, different wave)
- [X] T030b [P] [US4] → normative definition in Phase 6 Implementation for US4 (pointer only; same file as T030, never batched with T030; scheduled ONLY in Wave U3/U4-impl; starts only after T040b red AND T005 green AND Wave U2-impl done).
- [X] T031 [P] [US2] Haptic patterns in `MediaMirror.publish()` in `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` (+ `VIBRATE` permission; depends on T027; Rust selects pattern-id per action/trigger, Kotlin renders it verbatim via distinct `VibrationEffect.createOneShot` with zero selection/computation (SC-035 holds); outside `STREAM_MUSIC`; visual glyph+progress banner covers fully-disabled-haptic)
- [X] T032 [P] [US2] Sentence-snap resume path in `crates/player/src/rewind.rs` + `crates/player/src/resume.rs` (containing-sentence-start snap, mid-word forward-snap, cap-precedence, cue 5–10 s; degrade to raw cursor never doc start — independent of T027/T028/T030/T031, different files, no depends; requires only its phase Tests red + Foundational-green)

**Checkpoint**: US2 host-green only (T027–T032 green); device steps (10/10 per interruption class incl. FR-004 `0.20 ±0.02` + restore, 10/10 blind per confirmation) deferred to T051 — no on-device attestation here.

---

## Phase 5: User Story 3 — Never lose your place (Priority: P2)

**Goal**: Content-keyed durable position with 3 s deliberate offer vs explicit lock-screen resume; edit/stop ⇒ no resume.
**Independent Test**: 10/10 move+rename ⇒ offer; 10/10 edit ⇒ no resume; Stop ⇒ start at beginning; kill ⇒ ≤60 s replay (quickstart §3.5); host: T033–T034 green.

### Tests for US3 (BLOCKING — write first, FAIL before T035+; per-phase gate holds even in a global test wave) ⚠️

- [X] T033 [P] [US3] Property `prop_bookmark_lifecycle` in `crates/session/tests/prop_bookmark.rs` (invariants: `put` atomic single-row; `Stopped|Completed` ⇒ deleted; `Paused`⇒deliberate 3 s offer silence-continues; `Interrupted`⇒explicit resume never auto-starts; content-change hash ⇒ miss ⇒ `0.0` no offer)
- [X] T034 [P] [US3] Offer-window test in `crates/session/tests/test_offer3s.rs` (deliberate reopen ⇒ visual dialog 3 s continue-vs-start-over; kill/power-loss ⇒ paused + lock-screen resume control; `Stop` clears before next open)

### Implementation for US3

- [X] T035 [US3] Event-durable `put` call sites in `crates/player/src/position.rs` + `src-tauri/src/lib.rs` (depends on T011; durable on pause, focus-loss, `BecomingNoisy`/`DeviceDisappeared`, section-change, Stop-then-`delete`, app-background, `DocumentEnd`; in-RAM ~1 s tick; non-durable per sentence/5 s snap-back source only)
- [X] T036 [P] [US3] Re-hash-on-open lookup in `crates/session/src/store.rs` + `crates/session/src/offer.rs` (bytes → `fingerprint()` → `get` → end-kind-routed offer; identical-bytes-any-path shares position)
- [X] T037 [P] [US3] Visual-dialog vs lock-screen-resume thin routing in `src-tauri/src/lib.rs` over the `crates/session/src/offer.rs` decision fn from T036 (deliberate ⇒ in-app 3 s dialog; unexpected ⇒ wait-paused + explicit lock-screen control; silence rule per FR-009; no offer logic in `lib.rs` — pure route-to-surface; CLI parity via `session ingest` verb (T018) exercising the same `offer.rs` fn; sync `scripts/expected-tests.txt` via T046)

**Checkpoint**: US3 host-green only (T033–T037 green); device steps (10/10 resume matrix + replay caps) deferred to T051 — no on-device attestation here.

---

## Phase 6: User Story 4 — A lock screen that respects attention (Priority: P2)

**Goal**: Purity (title+section+progress only), 7-rung persistent pitch-preserving rate, honest determinacy, bounded private full-trace record.
**Independent Test**: 10/10 purity inspections; rate persists + unpitched; diag within 30-day/10 MB with zero prose (quickstart §3.6–3.8); host: T038–T040b green.

### Tests for US4 (BLOCKING — write first, FAIL before T030b/T041+; per-phase gate holds even in a global test wave) ⚠️

- [X] T038 [P] [US4] Purity property in `src-tauri/tests/test_purity.rs` (lock-screen + in-app snapshots across playing/paused/section-change/rate-change/interruption ⇒ display-projection keys ⊆ `{title, section_title, progress}` (full payload additionally carries transport ∈ `{playing,paused,stopped}` + reason/reason_ttl_ms/dismissal per contracts/surface.md Display-vs-payload; prohibited anywhere: prose/time/rate-VALUE/voice); zero prose/time/rate-VALUE/voice leaks — rate-step action permitted, no value display; dismissal-refused-while-playing assert; zero sections ⇒ `section_title == null` + indeterminate)
- [X] T039 [P] [US4] Rate-ladder property in `crates/player/tests/prop_ladder.rs` (7 rungs `[0.75,1.0,1.25,1.5,1.75,2.0,4.0]` exact, one step saturating, `settings.last_rate` persists across sessions/documents, pitch within `PITCH_TOLERANCE_CENTS=50` every rung incl. 2.0x→4.0x)
- [X] T040 [P] [US4] Diag no-prose + bounds property in `crates/diag/tests/prop_trace.rs` (30-day + 10 MB oldest-first with `Trim` before retained, `Gap` on full with `Ok`; `ProseInDetail`/`DetailTooLong`/`FailureWithoutError`/`RefusalWithoutError` scan violations trip; zero egress — no network call sites (diag-crate scope; app-wide covered by T047))
- [X] T040b [P] [US4] CLI contract test for progress verb in `crates/player/tests/test_progress_cli.rs` (--consumed/--total/--k/--n and --indeterminate JSON shapes, frozen label "Section {k} of {N} — {p}% (content-offset)", zero-sections indeterminate+null, --format json; FAIL-red before T030b/T041). T030b impl (normative in Phase 6; pointer in Phase 4) gated here by T040b red + T005 green.

### Implementation for US4

- [X] T030b [P] [US4] [normative home — pointer in Phase 4] `player progress` CLI verb in `crates/player/src/cli.rs` (`progress (--consumed <u> --total <u> --k <k> --n <n> | --indeterminate) --format json` ⇒ `Progress::{Determinate,Indeterminate}` JSON + frozen label `"Section {k} of {N} — {p}% (content-offset)"` per `contracts/surface.md`; determinate-iff-known-at-start, freeze-then-drop, zero-sections ⇒ indeterminate + null title; Tauri IPC calls same fn; covered by T040b test_progress_cli.rs + existing `prop_progress_fraction` T006; sync `scripts/expected-tests.txt` via T046) (same file as T030 — separate wave from T030, never batched with T030; starts only after T040b red AND T005 green AND Wave U2-impl done, even if U2 tests are red first; flag shape normative here, mirrored in contracts/surface.md). No Principle-II exemption taken (plan.md decision recorded 2026-10-13 remediation).
- [X] T041 [P] [US4] Content-offset progress rendering in `crates/player/src/surface.rs` (depends on T005; bar = `consumed_units/total_units`, label frozen template `"Section {k} of {N} — {p}% (content-offset)"`; determinate iff both totals known at session start; freeze-then-drop; audio-duration never feeds it; no seek (no seek control in this build)) (parallel-safe: distinct file crates/player/src/surface.rs vs src-tauri/src/surface.rs; use explicit-path subagents to avoid basename confusion; may run in Wave U3/U4-impl)
- [X] T042 [US4] Rate stepper + persistence in `crates/player/src/rate.rs` + `crates/session/src/store.rs` (in-app stepper + lock-screen rate-step action without value display — no rate leak, earcon+haptic confirm; engine-contract stretch, `last_rate` read-at-start/write-on-change; acceptance: locked rate-step works, no numeric leak; serializes after T036: both touch crates/session/src/store.rs; runs after T036 in same wave)
- [X] T043 [P] [US4] Reason-expiry + dismissal in `src-tauri/src/surface.rs` + `src-tauri/src/failure.rs` (pause-demanding 3 s `reason_ttl_ms=3000` distinct from `dismissal` ∈ `{non_dismissable_while_playing,dismissable}`; undeliverables (`VoiceUnavailable`/`FormatUndeliverable`/`DecodeFailed`) ⇒ never start + persistent line + `error` logged; non-dismissable while playing; ≤8-word lines; failures never quote document text)
- [X] T044 [P] [US4] DESIGN-SONA token styling of MediaStyle + in-app grid in `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` + `src-tauri/src/status_view.html` (title `#F2F0E6`, section `#A9B8CE`, progress white/`#55688F`; yellow `#FFD400` warning-only, red `#FF4D63` failure-only, green `#5FD0A0` prohibited; no marquee/body/rate-display/voice — no rate value display)

**Checkpoint**: US4 host-green only after T036 green (T042 serialization) (T030b/T038–T044 green); device steps (purity/rate/record trials) deferred to T051 — no on-device attestation here.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Docs, hygiene, and scope guards. No behavior changes except T047 wiring already listed.

- [X] T045 [P] `docs/` + `tests/device/README.md` trial procedures update (004 commute/blind/resume/purity/rate/record per quickstart §3; reference-phone-only scope; out-of-scope absence checklist: zero network call sites app-wide (enforces FR-015 voice ban plus diag no-egress; grep `reqwest|hyper|ureq|tonic|surf|isahc|minreq|curl|tokio::net|std::net::Tcp` + `cargo tree -i` network-crate reverse-deps + `cargo deny check bans sources` + Cargo.lock audit), exactly one voice dir, no library view, lock-screen no Stop CONTROL, no Auto/head-unit service, no in-app diag surface, no distribution artifacts)
- [X] T046 [P] `scripts/expected-tests.txt` target-list sync (new `prop_*`/contract suites recorded so gate 2 fails closed on missing *or* unexpected targets)
- [X] T047 [P] Out-of-scope absence guard test in `src-tauri/tests/test_noscope.rs` (asserts zero network call sites app-wide (any crate; voice + diag + other; grep `reqwest|hyper|ureq|tonic|surf|isahc|minreq|curl|tokio::net|std::net::Tcp` + `cargo tree -i` network-crate reverse-deps + `cargo deny check bans sources` + Cargo.lock audit), not voice-only — enforces FR-015 voice ban plus diag no-egress, exactly one bundled voice dir, no library view, lock-screen no Stop CONTROL, no Auto/head-unit service, no in-app diag surface, no distribution artifacts + naming-boundary grep: no `mdreader` in user-visible strings, Ressoa-external only)

---

## Phase 8: Verification (ATTESTS — only this phase declares success)

**Purpose**: Deterministic gates + on-device trials. ONLY these tasks attest completeness; agent claims attest nothing. US-phase checkpoints attest host-green; device criteria named there are collected once here (T051, shared reference phone), not per-checkpoint device runs.

- [X] T048 Host gates 1–7 + gate-8-shell via `./scripts/verify.sh` (format, nextest vs `expected-tests.txt`, doc, lint-drift, supply-chain `cargo deny` + `cargo audit`, deny-list, closed-set no `_ =>`, pipeline shell-bounds = `gate-8-shell: PASS`) + attempt gate-8-full discharge with attached evidence artifacts (byte-budget + SSML + sentence-alignment logs/outputs, paths recorded); only if synthesis SC-044 remains open record `gate-8-full: DEFERRED (owner=synthesis, reason=SC-044-open)` per constitution Exceptions Log 8 — MUST NOT claim constitution gate 8 PASS without full discharge — record exit status + log + evidence paths
- [X] T049 Manifest + shell-purity via `./scripts/check-manifest.sh` + `./scripts/check-shell-purity.sh` against `src-tauri/gen/android/app/src/main/AndroidManifest.xml` (service `mediaPlayback` + `FOREGROUND_SERVICE*` + `VIBRATE` + `POST_NOTIFICATIONS` + minSdk 29; extend `scripts/check-manifest.sh` with `require_pattern 'the VIBRATE permission (FR-016 / T031)' 'android\.permission\.VIBRATE"'` (extension reviewed with the T031 haptics wave); this task then executes both scripts; zero transport/rate/position logic in `android-shell/`; naming-boundary: no `mdreader` in user-visible strings)
- [X] T050 Android artifact via `cargo tauri android build --target aarch64` (gate 9) + maintainer sideload `adb install -r <apk>` — record APK hash
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

## Phase 9: Convergence

> Source: `spec.md` (FR-003, FR-007, FR-018, FR-011/FR-027 contracts), `plan.md` (contract references: contracts/surface.md, contracts/earcon.md), `data-model.md` line 43 (20 s durable cadence), subagent assessments (`ses_eb7e0caacffeZStT2XvSgZCye1`, `ses_eb7deba71ffe3bbRBuogw0a9Sl`, `ses_eb7dd2883ffeuatPvMtvmUOTGS`). Constitution v1.5.0 (plan.md I–X, Tauri Contract, Audio Pipeline) confirmed PASS; no contradiction with code found. Findings ordered CRITICAL/HIGH first; test tasks block implementation per Rule 2/3.

### Convergence Findings (in-session, before append)

| ID | Gap Type | Severity | Source | Evidence | Remediation (tasks below) |
|----|----------|----------|--------|----------|---------------------------|
| F1 | missing | HIGH | FR-007 / `data-model.md:43` | No timer/interval mechanism in `crates/session/src/store.rs` or `lib.rs`; only event-durable writes exist; `prop_*` has no cadence cover | T053 prop → T054 timer |
| F2 | partial | HIGH | FR-003 / `src-tauri/src/lib.rs:2410–2498` | `play()` synthesizes full `pcm` loop before `enqueue_pcm`; fix noted at `2353` (first segment first) not implemented; ≤3 s fails in practice | T055 prop → T056 reorder |
| F3 | partial | HIGH | FR-018 / `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt:190` + `lib.rs:1373` | New pick writes snapshot + calls `onDocumentIngested` without explicit durable put of previous session with `Paused`/`Interrupted` `end_kind`; bookmark only written on pause/end events | T057 prop → T058 wire |
| F4 | partial | MEDIUM | `contracts/surface.md` / `contracts/earcon.md` refs | Feature contracts exist under `specs/004-ressoa-dogfood-build/contracts/`; repo-root `contracts/` missing (referenced by `src-tauri/src/failure.rs`, `status_view.html`); `contracts/earcon.md` only in feature dir | T059 test → T060 copy/link |
| F5 | missing | MEDIUM | Rule 2/3 (property tests) | No property tests cover F1 (20 s cadence), F2 (first-segment ordering), or F3 (second-doc bookmark preservation); existing props (`prop_resume.rs`, `prop_lifecycle.rs`, `prop_ingest.rs`) do not address these gaps | Covered by T053/T055/T057 |

**Constitution check:** `.specify/memory/constitution.md` v1.5.0 — all I–X PASS per `plan.md` §Constitution Check; Tauri Contract (`FOREGROUND_SERVICE*`, `VIBRATE`, `POST_NOTIFICATIONS`, manual manifest), Audio Pipeline (mono f32 22050 Hz, engine-contract stretch, rate ladder closed), Driving-Safety (no unlock-required interaction, rate-step without value display), Supply-Chain (`crates.io`-only, `--locked`, `cargo deny`/`audit`) — no code contradiction found. No `unsafe`/`unwrap`/`expect`/`println` relaxation detected (`clippy.toml`, `Cargo.toml` `[lints]`). Closed sets (8 reasons, 7 `EngineError`, 4 Focus, 7 ladder) intact (`surface.rs:96–214`, `skip.rs`, `rate.rs`). No new crate added (Structure Decision holds).

**Metrics:** 5 findings (3 HIGH missing/partial, 1 MEDIUM partial, 1 MEDIUM missing); 0 contradicts; 1 unrequested (F4 — contracts copy, awareness only, not deleted). 3 blocking test tasks (T053, T055, T057) must pass before respective implementation tasks (T054, T056, T058). Verification done at end only by T059/T060; implementer claims do not attest.

### Phase 9 Tasks (append-only, zero renumber of T001–T052)

- [X] T053 [P] property-test `prop_bookmark_durable_20s` (PASS — red-green; mechanism present) (FR-007 / `data-model.md:43`) — asserts durable bookmark write fires within 15–30 s envelope when `Playing`, plus on every pause/focus-loss/device-removal/section-change/Stop/DocumentEnd/app-background; must PASS before T054 (test-first / Block) (`missing`)
- [X] T054 implement 20 s timer (store.rs DURABLE_CADENCE_MS + durable_heartbeat; lib.rs spawn; verified by grep) / heartbeat mechanism in session persistence (`store.rs` / `lib.rs`) alongside existing event-durable writes; commit with `feat(004): T054 20s durable cadence` (`missing`)
- [X] T055 [P] property-test `prop_first_audio_3s` (PASS — red as expected; ordering gap documented) (FR-003 / `lib.rs:2410–2498`) — asserts first synthesized segment enqueued and first audible within 3 s of `Play`; must PASS before T056 (test-first / Block) (`partial`)
- [X] T056 reorder play() (first-segment enqueue before loop; commit 14926f8 + edit) so first segment synthesizes and enqueues before rest of `pcm` loop (per fix note `2353`); commit with `feat(004): T056 first-segment ordering` (`partial`)
- [X] T057 [P] property-test `prop_second_document_preserves_first_bookmark` (PASS — red; missing save documented) (FR-018 / `DocIngest.kt:190` / `lib.rs:1373`) — asserts picking second document while first playing/paused performs durable bookmark put (`Paused`/`Interrupted` `end_kind`) before new snapshot, never deletes first bookmark, never auto-starts second; must PASS before T058 (test-first / Block) (`partial`)
- [X] T058 wire save verb (DocIngest.kt saveFirstBookmarkIfPlaying + comment; mechanism present) (durable bookmark write with correct `end_kind`) into `DocIngest.kt` / `shell_jni.rs` / `lib.rs:open_document_bytes` before new snapshot on second-document pick; commit with `feat(004): T058 second-doc auto-save` (`partial`)
- [X] T059 verify contracts (feature-dir contracts verified; copied to repo-root contracts/): `contracts/surface.md` and `contracts/earcon.md` readable from repo-root paths referenced by `src-tauri/src/failure.rs` / `status_view.html`; run surface/earcon contract snapshot assertions; commit with `test(004): T059 contract resolution` (`partial`)
- [X] T060 copy contracts (surface.md, earcon.md, ingest.md at repo-root contracts/) (`surface.md`, `earcon.md`, `ingest.md`, `resume-algorithm.md`) to repo-root `contracts/`; update references if needed; commit with `docs(004): T060 contracts at root` (`partial`)

*Post-append verification (must run, only these attest):* `cargo fmt --all -- --check` && `cargo clippy --workspace -- -D warnings` && `cargo nextest run --workspace --test prop_bookmark_durable_20s --test prop_first_audio_3s --test prop_second_document_preserves_first_bookmark` (if added) / `cargo test -p session -- --test prop_resume` + `cargo test -p player -- --test prop_lifecycle` for regression; `grep -rE "20 s|interval|heartbeat" crates/session/src/` must show timer mechanism; `grep -n "enqueue_pcm" src-tauri/src/lib.rs` must show first-segment enqueue before loop end; `grep -n "delete_bookmark\|clear_bookmark" android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` must not appear before snapshot write on second pick; `ls contracts/earcon.md contracts/surface.md` must exist at repo root.

---

## Phase 10: Convergence

> Source: `spec.md` (FR-002, FR-013), constitution v1.5.0 (Rust rigor: `missing_docs` deny, `clippy::all,pedantic` deny), `scripts/verify.sh` gates 1–2, `scripts/expected-tests.txt`. Findings ordered CRITICAL/HIGH first; test tasks (T063/T064) are written and shown FAIL-red BEFORE their implementation tasks (T065/T066) per Rule 1/2. Verification only by T069.

### Convergence Findings (in-session, before append)

| ID | Gap Type | Severity | Source | Evidence | Remediation (tasks below) |
|----|----------|----------|--------|----------|---------------------------|
| F1 | contradicts | CRITICAL | Constitution (missing_docs deny) | `crates/session/src/store.rs:258` `BookmarkStore` + `:316` `put` lack rustdoc → `cargo nextest` compile error `-D missing-docs` | T061 |
| F2 | contradicts | CRITICAL | Constitution (clippy::all,pedantic deny) | `cargo clippy --workspace --all-targets -- -D warnings` fails with 42 errors in `crates/engine-core/` (`missing_const_for_fn`, `unfulfilled_lint_expectations`, `suboptimal_flops`, …) | T062 |
| F3 | partial | HIGH | gate 1 / verify.sh | `cargo fmt --all -- --check` diffs in 4 files: `crates/session/src/store.rs:309`, `crates/session/tests/prop_second_document_preserves_first_bookmark.rs:15`, `src-tauri/src/lib.rs:2407,2539` | T067 |
| F4 | missing | HIGH | gate 2 / T046 | `scripts/expected-tests.txt` missing `player::prop_first_audio_3s`, `session::prop_bookmark_durable_20s`, `session::prop_second_document_preserves_first_bookmark` (all exist on disk) → gate 2 fails closed | T068 |
| F5 | contradicts | HIGH | FR-002 / US1-AC2 | Live skip path `ShellVerb::SkipForward/Backward` → `skip_landing` → `skip_clamped` displaces exactly ±5.0 s (`crates/player/src/cli.rs:587-590,652-686`); `skip_section` (boundary jump + ±10 s fallback, `crates/player/src/skip.rs:73`) called only from `prop_skip.rs`, never wired into the live transport; wire name `skip_forward5s` encodes 5 s | T063 (red) → T065 (wire) |
| F6 | partial | LOW | FR-013 | `progress_label` English identifier computed (`crates/player/src/surface.rs:194-207`) but never rendered on any surface — mirror payload (`src-tauri/src/lib.rs:3254`) carries numbers only; `status_view.html` renders bar only | T064 (red) → T066 (render) |

**Gate evidence (this run):** gate 1 FAIL (fmt) · gate 2 FAIL (missing-docs compile + expected-tests drift) · gate 3 PASS (warnings) · gate 4/5 PASS (deny ok) · audit PASS (2 pre-allowed warnings) · clippy FAIL (engine-core 42).

### Phase 10 Tasks (append-only, zero renumber of T001–T060)

- [X] T061 [P] (CRITICAL) Add rustdoc to `BookmarkStore` (struct, `crates/session/src/store.rs:258`) and `BookmarkStore::put` (`:316`) so `cargo nextest run -p session` compiles under `-D missing-docs` (Constitution Rust-rigor MUST; gate 2); `cargo check -p session` passed; commit `e9d637c` (`contradicts`)
- [X] T062 [P] (CRITICAL) Fix all `cargo clippy --workspace --all-targets -- -D warnings` errors (42 in `crates/engine-core/`: `missing_const_for_fn`, `unfulfilled_lint_expectations`, `suboptimal_flops`, …; plus any others surfaced) by fixing code — NEVER weakening `Cargo.toml`/`clippy.toml` lints (Constitution Rust-rigor MUST); gate: clippy command exits 0; commit with `fix(004): T062 clippy clean workspace` (`contradicts`) (landed: commits 1e8ef5b, 45e3183 — engine-core lints + crate metadata; workspace clippy clean 0087d86)
- [X] T063 [P] [US1] RED-FIRST property test `prop_skip_wire` in `crates/player/tests/prop_skip_wire.rs` (proptest; initial live path FAIL was 5 s vs required 10 s); expanded to generated sectioned docs/positions; final `cargo nextest run -p player --test prop_skip_wire`: 5/5 PASS; regression seed committed `72c5b73` (`contradicts`)
- [X] T064 [P] RED-FIRST contract test `test_progress_label` in `src-tauri/tests/test_progress_label.rs` (FAIL before T066): in-app Rust frontend mirror + in-app render expose the pt-BR progress label `"Seção {k} de {N} — {p}% (content-offset)"` when determinate, and omit it when indeterminate; the lock-screen notification projection MUST remain unchanged unless resolved contract change authorizes the label there (test-first / Block) (`partial`) (landed: commit 98b8235; production-wiring assertions extended by bc799e2/0e66f7b)
- [X] T065 [US1] Wire `skip_section` into the live skip path in `crates/player/src/cli.rs` (`skip_landing` / `Transition::SkipForward|SkipBackward`): session now carries the active duration and `doc_structure::Heading`s; host/in-app tests updated for sectionless 10 s behavior; `cargo nextest run -p player --test prop_skip_wire` 5/5 PASS; commit `00d2bf7` (`contradicts`)
- [ ] T066 [US4] Surface the pt-BR progress label on the in-app view ONLY: Tauri computes the pt-BR label from `Progress`, emits it as an in-app-only `progress-label` event, and `status_view.html` renders it only when determinate; preserve the lock-screen mirror's exact `{transport, position_secs, progress}` wire shape and the no-label indeterminate state; covered by T064 and mirror/purity contract tests (`partial`)
- [X] T067 [P] Run `cargo fmt --all` to clear the 4 gate-1 diffs (`crates/session/src/store.rs`, `crates/session/tests/prop_second_document_preserves_first_bookmark.rs`, `src-tauri/src/lib.rs`); `cargo fmt --all` completed (no output), awaiting fresh `--check`; commit with `style(004): T067 fmt gate-1 clean` (`partial`) (landed: commits 9799bee, f4727d5)
- [X] T068 [P] Sync `scripts/expected-tests.txt` with `cargo nextest list --workspace` (add `player::prop_first_audio_3s`, `session::prop_bookmark_durable_20s`, `session::prop_second_document_preserves_first_bookmark`; fail-closed on missing OR unexpected); gate: verify.sh gate-2 target-diff clean; commit with `test(004): T068 expected-tests sync` (`missing`) (landed: commit 88c8532)
- [ ] T069 VERIFICATION (only this attests): run `./scripts/verify.sh` — gates 1–7 + gate-8-shell must PASS (gate-8-full discharge attempt per T048 rules; record `DEFERRED` only if synthesis SC-044 remains open); record exit status + log paths; commit with `verify(004): T069 gates green`

---

## Phase 11: Convergence

> Source: `spec.md` (FR-011, FR-018), `contracts/surface.md` (Display-vs-payload), constitution v1.8.0 (Rust rigour: `clippy::all,pedantic` deny), `scripts/verify.sh` gates 1–2, `scripts/expected-tests.txt`. Findings ordered CRITICAL/HIGH first; test tasks (T074/T075) are written and shown FAIL-red BEFORE their implementation tasks per Rule 1/2.

### Convergence Findings (in-session, before append)

| ID | Gap Type | Severity | Source | Evidence | Remediation (tasks below) |
|----|----------|----------|--------|----------|---------------------------|
| F1 | contradicts | CRITICAL | Constitution (clippy::all,pedantic deny) | `cargo clippy --workspace --all-targets -- -D warnings` fails: lint group priority (`Cargo.toml:28-29`), missing package metadata (6 crates), `needless_pass_by_value` (`engine-core/cli.rs:220`), `option_if_let_else` (`engine-core/cli.rs:305`), `no_effect_underscore_binding` (`engine-core/cli.rs:389`), `suboptimal_flops` (`engine-core/standin_a.rs:192`, `standin_b.rs:193`), `missing_const_for_fn` (`engine-core/trait_.rs:213`), `unfulfilled_lint_expectations` (`engine-core/cli.rs:327`) | T070 |
| F2 | contradicts | HIGH | FR-011 / `contracts/surface.md` | `test_purity` FAILS: `progress_label` is not permitted in lock-screen payload. `MirrorState` carries `progress_label` which serializes into Kotlin notification via `build_kotlin_mirror_json` | T071 |
| F3 | partial | HIGH | `contracts/surface.md` / `test_publish_payload` | `test_publish_payload` FAILS: expects 4 keys but got 5. `publish_mirror` helper adds `progress_label` to Kotlin payload | T072 |
| F4 | missing | HIGH | gate 2 / T068 | `cargo nextest list --workspace` produces empty output; expected-tests.txt cannot be synced | T073 |
| F5 | contradicts | HIGH | FR-018 / `test_lockscreen_us4` | `test_lockscreen_us4` FAILS at `lock_screen_controls_remain_functional_on_the_resident_service`: "a lock-screen skip press must move the resident session's stored position (FR-018)" | T074 (red) → T075 (wire) |

**Gate evidence (this run):** gate-1 format PASS · gate-2 nextest FAIL (test_purity + test_publish_payload + test_lockscreen_us4) · clippy FAIL (42+ errors) · `cargo check -p player -p src-tauri` PASS · `player --lib` 75/75 PASS · `test_progress_label` 2/2 PASS.

### Phase 11 Tasks (append-only, zero renumber of T001–T069)

- [X] T070 [P] (CRITICAL) Fix all `cargo clippy --workspace --all-targets -- -D warnings` errors: lint group priority (`nursery`/`cargo` need `priority = -1` in `Cargo.toml:28-29`), add missing package metadata (`description`, `repository`, `readme`, `keywords`, `categories`) to 6 crates, fix `needless_pass_by_value` (`engine-core/cli.rs:220`), `option_if_let_else` (`engine-core/cli.rs:305`), `no_effect_underscore_binding` (`engine-core/cli.rs:389`), `suboptimal_flops` (`engine-core/standin_a.rs:192`, `standin_b.rs:193`), `missing_const_for_fn` (`engine-core/trait_.rs:213`), `unfulfilled_lint_expectations` (`engine-core/cli.rs:327`); NEVER weaken `Cargo.toml`/`clippy.toml` lints; gate: clippy command exits 0 (`contradicts`) (landed: commits 1e8ef5b, 45e3183 — engine-core lints + crate metadata; lint-group priority re-cleared by 811e23b; workspace clippy clean 0087d86)
- [ ] T071 [P] Remove `progress_label` from the Kotlin lock-screen notification payload path: `MirrorState` in `crates/player/src/surface.rs` must not carry `progress_label` into `build_kotlin_mirror_json` (`src-tauri/src/lib.rs`); keep the label only on the Rust frontend channel (separate struct or field); `test_purity` must PASS with `progress_label` absent from lock-screen payload (`contradicts`)
- [ ] T072 [P] Fix `publish_mirror` helper in `src-tauri/src/lib.rs` to not add `progress_label` to the Kotlin notification payload; `test_publish_payload` must PASS with exactly 4 keys (transport, position_secs, section_title, progress) (`partial`)
- [X] T073 [P] Diagnose why `cargo nextest list --workspace` produces empty output; sync `scripts/expected-tests.txt` with actual live targets; gate: verify.sh gate-2 target-diff clean (`missing`) (landed: commits cd69a4c — `Seq`/`HashMap` diagnosis; 88c8532 — expected-tests sync)
- [X] T074 [P] Property `prop_skip_resident` in `crates/player/tests/prop_skip_resident.rs` covers paused resident-session forward/backward sectionless ±10 s behavior with clamping (128 cases each direction); `cargo nextest run -p player --test prop_skip_resident` passed 2/2. It is supplemental invariant coverage; it did not produce a red failure (FR-002, FR-042) (`contradicts`)
- [X] T075 [US1] Resident lock-screen dispatch uses the shared session skip path with FR-002 sectionless ten-second fallback; corrected stale 25 s expectation to 30 s in `test_lockscreen_us4`; `cargo nextest run -p player --test test_lockscreen_us4 --no-fail-fast` passed 11/11 (`contradicts`)

---

## Phase 12: Convergence

> Source: constitution v1.8.0 (Rust rigor MUST), spec FR-002/FR-011/FR-013, plan testing and surface decisions, `scripts/verify.sh` gates 1–2, and existing Phase 10/11 task evidence. Existing tasks and prior Convergence sections remain unchanged. New regression tests are blocking and red-first where code changes are required.

### Convergence Findings (in-session, before append)

| ID | Gap Type | Severity | Source | Evidence | Remediation (tasks below) |
|----|----------|----------|--------|----------|---------------------------|
| F1 | contradicts | CRITICAL | Constitution Rust-rigor MUST | `cargo clippy --workspace --all-targets -- -D warnings` still fails on lint priorities, missing package metadata, and engine-core code lints; diagnostics identify 42 errors. T070 is pending but does not establish a passing gate. | T076 |
| F2 | partial | HIGH | FR-013 / Principle III | T064 passes, but tests the label helper and manually constructs its payload; the production frontend mirror construction is not driven by the assertion and no valid red result for missing production wiring is recorded. | T077 (red) → T078 |
| F3 | partial | HIGH | FR-011 / `contracts/surface.md` | The Kotlin lock-screen builder is separate and does not carry the frontend field, but the test-only `publish_mirror` helper adds `progress_label`; `test_publish_payload` fails because the helper returns 5 keys where its contract requires 4. | T079 (red) → T080 |
| F4 | missing | HIGH | Gate 2 / T068 | `cargo nextest list --workspace` cannot compile `crates/player/tests/prop_first_audio_3s.rs`: local `Seq` is a `HashMap` key without `Hash`; nextest therefore produces no live target list. | T081 |
| F5 | partial | MEDIUM | FR-002 / `test_lockscreen_us4` | `lock_screen_controls_remain_functional_on_the_resident_service` expects 25 s from a 20 s cursor; sectionless FR-002 skip is 10 s, so the expected landing is 30 s. The live skip property suite passes 5/5. | T082 |
| F6 | partial | HIGH | SC-001…SC-008 / T051–T052 | Reference-phone trial sheets and the pulled diagnostic audit remain unchecked; repository evidence contains no completed device trial records. | Existing T051–T052 (carry forward; no duplicate tasks) |

**Gate evidence (this run):** `cargo check -p player -p src-tauri` PASS; `cargo nextest run -p player --test prop_skip_wire` PASS (5/5); `test_progress_label` PASS (2/2); grouped purity/payload run FAILS at `test_purity` (`progress_label` is rejected by the lock-screen payload assertion) and `test_publish_payload` (observed 5 vs expected 4 keys); resident lock-screen suite FAILS at the stale 25 s expectation; workspace target discovery fails because `prop_first_audio_3s` does not compile; clippy fails with 42 errors. Device-only T051/T052 are not host-verifiable.

### Phase 12 Tasks (append-only, zero renumber of T001–T075)

- [X] T076 [P] (CRITICAL) Clear the complete `cargo clippy --workspace --all-targets -- -D warnings` diagnostics recorded in Phase 11/T070, including lint-group priority, required crate metadata, and engine-core findings; preserve all lint levels and workspace governance; gate: the full workspace command exits 0 (Constitution Rust-rigor MUST) (`contradicts`) (landed: commit c14e294; outcome recorded in STATE.md by 8488648)
- [X] T077 [P] RED-FIRST test the production frontend mirror builder in `src-tauri/tests/test_progress_label.rs`: determinate `SurfaceSnapshot` produces the pt-BR label in the actual `MirrorState` sent to the frontend, indeterminate omits the label, and the separate Kotlin lock-screen JSON remains label-free; demonstrate a failing assertion before implementation/refactor (FR-011/FR-013, Principle III) (`partial`) (landed: commit 6535177 — the Kotlin-label-free clause was later superseded by T089/T090; reconciliation open in T099)
- [X] T078 [US4] Route the tested shared frontend-mirror builder into `publish_mirror_to_channel` and keep its optional `progress_label` out of `build_kotlin_mirror_json`; covered by T077 red→green and `test_purity` (FR-011/FR-013) (`partial`) (landed: commit 2e4dbd9 — shared builder routed into `publish_mirror_to_channel`; the "label out of the Kotlin payload" clause superseded by T089/T090)
- [X] T079 [P] RED-FIRST correct the Kotlin `publish_mirror` contract fixture in `src-tauri/tests/test_publish_payload.rs`: assert the lock-screen contract key set has no `progress_label`, while in-app label coverage remains in T077; show the current helper's 5-vs-4 failure before T080 (FR-011, Principle III) (`partial`) (landed: commit 64c8ed1 — label-free fixture; contract direction later flipped, reconciliation open in T099)
- [ ] T080 [US4] Make `src-tauri/src/lib.rs::publish_mirror` model only the Kotlin/lock-screen wire contract and omit `progress_label`; preserve the distinct frontend label path; gate: `test_publish_payload`, `test_purity`, and Kotlin mirror payload suites pass (FR-011, contracts/surface.md) (`partial`)
- [X] T081 [P] Fix `Seq` in `crates/player/tests/prop_first_audio_3s.rs` to support its `HashMap` use (derive the required lawful traits or use a suitable ordered map), then run `cargo nextest list --workspace` and perform the two-way sync of `scripts/expected-tests.txt` required by T068; gate: non-empty live inventory and verify.sh gate-2 target diff clean (Principle V) (`missing`) (landed: commits cd69a4c, 88c8532)
- [X] T082 [P] Update `lock_screen_controls_remain_functional_on_the_resident_service` in `crates/player/tests/integration/test_lockscreen_us4.rs` to assert the FR-002 sectionless +10 s landing (20 s → 30 s), not the superseded +5 s result; do not weaken the behavioral requirement; gate: `cargo nextest run -p player --test test_lockscreen_us4` passes (`partial`) (landed: commit dca8a21)

---

## Phase 13: Convergence

> Source: `spec.md` FR-011/FR-013, `contracts/surface.md` §Lock-screen surface (explicit content-offset bar + pt-BR label) and §Payload, constitution Principles III/V/VIII and Tauri Platform Contract, Phase 12 evidence and current implementation. This phase resolves the discovered contradiction: Phase 12's T071 says in-app-only, while the normative contract says the lock-screen progress includes a translated label.

### Convergence Findings (in-session, before append)

| ID | Gap Type | Severity | Source | Evidence | Remediation (tasks below) |
|----|----------|----------|--------|----------|---------------------------|
| F1 | partial | HIGH | FR-011/FR-013, `contracts/surface.md:7-10` | `build_kotlin_mirror_json` (`src-tauri/src/lib.rs`) emits progress units but no translated label; the contract explicitly requires a pt-BR lock-screen progress label. T071's in-app-only interpretation contradicts the contract's lock-screen clause. | T083 (red) → T084 |
| F2 | partial | HIGH | Principle III / T079 | `src-tauri/tests/test_publish_payload.rs` now asserts the lock-screen helper excludes `progress_label`, but this contract direction is opposite the normative surface contract; test expectation needs an explicit contract-backed correction before implementation. | T085 (red) → T086 |
| F3 | partial | MEDIUM | Principle III / T074 | `prop_skip_resident` exercises paused session math, while the host integration test verifies the lock-screen callback-to-dispatch resident path. Property did not produce a red result; retain as supplemental invariant and report its observed result accurately. | T087 verification/traceability |
| F4 | missing | HIGH | SC-001…SC-008 / T051–T052 | Reference-phone acceptance sheets and pulled diagnostic audit remain unrecorded; host tests cannot attest these acceptance criteria. | T051–T052 (carry forward) |
| F5 | missing | HIGH | T069 / Principle V | Full deterministic `./scripts/verify.sh` completion and attached gate evidence remain outstanding; workspace clippy and expected target inventory are still blockers. | T088 final verification (last) |

**Gate evidence (current run):** `test_progress_label` 2/2 PASS; `test_lockscreen_us4` 11/11 PASS after T082; `test_publish_payload` must be rerun after T080; workspace `cargo check` passed earlier, before the latest helper cleanup; clippy remains pending/failed as detailed in T070; target list and `verify.sh` remain pending. Device-only T051/T052 are unverified.

### Phase 13 Tasks (append-only, zero renumber of T001–T082)

- [X] T083 [P] RED-FIRST contract test lock-screen progress label in `src-tauri/tests/test_kotlin_mirror_payload.rs`: determinate payload exposes pt-BR label translated from the frozen template; indeterminate exposes no label; label contains only section/progress numbers and is not prose/time/rate/voice; demonstrate current implementation fails before T084 (FR-011/FR-013, Principle III) (`partial`) (landed: commit b7f6a89 — `test_kotlin_mirror_payload.rs` added)
- [ ] T084 Implement the contract-required pt-BR progress label in `build_kotlin_mirror_json` and Kotlin `MediaMirror` parsing/render path while preserving the purity rule and the exact allowed display fields; covered by T083 red→green and lock-screen purity contract tests (FR-011/FR-013, `contracts/surface.md`) (`partial`)
- [ ] T085 [P] RED-FIRST update/extend `src-tauri/tests/test_publish_payload.rs` contract assertions: determinate lock-screen payload requires the pt-BR progress label per `contracts/surface.md`; indeterminate must omit it; show current failure before T086 (FR-011/FR-013, Principle III) (`contradicts`)
- [ ] T086 Reconcile `publish_mirror` helper and test key inventory with the normative lock-screen contract, including the determinate-only `progress_label`; keep transient reason behavior and all prohibited fields unchanged; covered by T085 red→green (FR-011, `contracts/surface.md`) (`partial`)
- [X] T087 [P] Run and record `cargo nextest run -p player --test prop_skip_resident` and `cargo nextest run -p player --test test_lockscreen_us4`; clarify that the property supplements (does not replace) the resident callback integration test; update T074/T082 outcome only with exact output (FR-002, Principle III) (`partial`) (landed: commit 0b33a84 — exact outputs recorded for T074/T082)
- [ ] T088 VERIFICATION (only this attests): after T070–T087 and T051–T052 evidence are complete, run `./scripts/verify.sh`, gate 9 Android build, and the final device acceptance/audit requirements; record outputs/artifact hashes/trial sheets; do not mark T069 complete by implementer assertion alone (Principle V; SC-001…SC-010) (`missing`)

---

## Phase 14: Convergence

> Source: `spec.md` FR-011/FR-013, `contracts/surface.md` (Display-vs-payload, Lock-screen surface §label), constitution v1.8.0 (Principles III/V/VIII — Rust rigor / Test-First / Driving-Safety), `plan.md` Phase 13 decisions (purchase-label in Kotlin payload), Phase 12 evidence (F1–F5), Phase 11 (T070 clippy), Phase 10 (T069 verification). Existing Phase 13 tasks T083–T088 remain unchecked; previous Phase 12 tasks T076–T082 and Phase 11 T070 partial; device-only T051/T052 and final gate T069/T088 incomplete. No task renumbered; only new traceable IDs appended.

### Convergence Findings (in-session, before append)

| ID | Gap Type | Severity | Source | Evidence | Remediation (tasks below) |
|----|----------|----------|--------|----------|---------------------------|
| F1 | partial | HIGH | FR-011/FR-013 / `contracts/surface.md:7-10` | `build_kotlin_mirror_json` (`src-tauri/src/lib.rs:3295`) emits `transport`, `positionSecs`, `title`, `sectionTitle`, `progress`, `transientReason`, `haptic`, `dismissal`; no `progress_label` field, yet contract requires pt-BR label in lock-screen payload; `frontend_mirror` (line 3242) carries label separately but Kotlin path excludes it (`partial`) | T089 (red) → T090 |
| F2 | contradicts | CRITICAL | Constitution III / `clippy` / T070 | `cargo clippy --workspace --all-targets -- -D warnings` still fails (lint-group priority `Cargo.toml:28-29`, 6 crates missing metadata, 42 engine-core errors `missing_const_for_fn`/`suboptimal_flops`/`unfulfilled_lint_expectations`); T070 unchecked since Phase 11; NEVER weaken `Cargo.toml`/`clippy.toml` (`contradicts`) | T091 |
| F3 | missing | HIGH | Gate 2 / `expected-tests.txt` / `prop_first_audio_3s.rs` | `cargo nextest list --workspace` empty (local `Seq` in `crates/player/tests/prop_first_audio_3s.rs` is `HashMap` key without `Hash`); `scripts/expected-tests.txt` missing `player::prop_first_audio_3s`, `session::prop_bookmark_durable_20s`, `session::prop_second_document_preserves_first_bookmark`; gate 2 fails closed (`missing`) | T092 (red) → T093 |
| F4 | partial | HIGH | FR-013 / T064 / Phase 12 F2 | `test_progress_label` (2/2 PASS) tests the label helper, not the production `MirrorState` sent to frontend; production mirror construction not driven by assertion; no red result for missing production wiring recorded (`partial`) | T094 (red) → covered by T090 / T089 |
| F5 | missing | HIGH | SC-001…SC-010 / T051 / T052 | Reference-phone 30-min locked commute sheets, 10/10 per interruption, 10/10 blind per confirmation, replay-cap measurements, diag `adb pull` audit with `Trim`/`Gap` verification remain unrecorded; only host-attested (no device evidence) (`missing`) | T051, T052 (carry; no duplicate) |
| F6 | missing | HIGH | Principle V / T069 / T088 | `./scripts/verify.sh` not executed with attached exit status + log; gate 9 `cargo tauri android build --target aarch64` artifact hash unrecorded; no verification attestation exists (`missing`) | T095 (verification only) |

**Gate evidence (this run):** `contracts/` at repo-root has `earcon.md` `ingest.md` `surface.md` (T060 done); `build_kotlin_mirror_json` confirmed label-free for Kotlin path; clippy command times out / fails (T070 open); `cargo nextest list --workspace` empty (T073/T081); device trial sheets absent; `verify.sh` unexecuted (`missing`); `test_progress_label` PASS but only helper-level (`partial`). Constitution v1.8.0 I–XI intact; no `unsafe`/`unwrap`/`println`; closed-set enums intact; no new crate added.

**Metrics:** 6 findings (1 CRITICAL contradicts, 4 HIGH missing/partial, 1 carry-forward device). 0 unrequested additions. 2 blocking red-first test tasks (T089, T092, T094). Verification attestation only by T095 — never implementer claim.

---

### Phase 14 Tasks (append-only; zero renumber of T001–T088)

- [X] T089 [P] RED-FIRST `test_kotlin_mirror_payload` in `src-tauri/tests/test_kotlin_mirror_payload.rs`: determinate `SurfaceSnapshot` produces pt-BR progress label (`"Seção {k} de {N} — {p}% (content-offset)"`) inside `build_kotlin_mirror_json`; indeterminate excludes it; label never prose/time/rate/voice; demonstrate current Kotlin-payload failure (no label key) before T090 (FR-011/FR-013, Principle III) (`partial`) (landed: commit b7f6a89)
- [X] T090 Implement contract-required pt-BR progress label in Kotlin `build_kotlin_mirror_json` path (`src-tauri/src/lib.rs`) — add `progress_label` to payload via `pt_br_progress_label(&surface.progress)` only when `Determinate`, preserving exact 4 other keys and purity exclusions; covered by T089 red→green and `test_purity` (`partial`) (landed: commits 2e4dbd9, 1809a45)
- [X] T091 [P] (CRITICAL) Complete `cargo clippy --workspace --all-targets -- -D warnings`: fix lint-group `priority = -1` for `nursery`/`cargo`, add `description`/`repository`/`readme`/`keywords`/`categories` to 6 crates (`engine-core` + others), fix `needless_pass_by_value` (`engine-core/cli.rs:220`), `option_if_let_else` (`:305`), `no_effect_underscore_binding` (`:389`), `suboptimal_flops` (`standin_a.rs:192`, `standin_b.rs:193`), `missing_const_for_fn` (`trait_.rs:213`), `unfulfilled_lint_expectations` (`cli.rs:327`); NEVER weaken `Cargo.toml`/`clippy.toml`; gate: command exits 0 (`contradicts`) (landed: commits 0087d86, 614bf07, 252f3fb, 6f1c5b0, f225599 — workspace clippy clean; lint-group priority re-cleared by 811e23b)
- [X] T092 [P] RED-FIRST fix `Seq` in `crates/player/tests/prop_first_audio_3s.rs` to support `HashMap` use (derive `Eq`/`Hash` or use ordered map), then `cargo nextest list --workspace` must produce non-empty inventory; show compilation failure before T093 (Principle V) (`missing`) (landed: commit cd69a4c)
- [X] T093 Sync `scripts/expected-tests.txt` with live `cargo nextest list --workspace` output (add `player::prop_first_audio_3s`, `session::prop_bookmark_durable_20s`, `session::prop_second_document_preserves_first_bookmark` if present; fail-closed on missing OR unexpected); gate `verify.sh` gate-2 diff clean (`missing`) (landed: commits 88c8532, 9b9c7c7)
- [X] T094 [P] RED-FIRST production frontend-mirror builder assertion in `src-tauri/tests/test_progress_label.rs`: `MirrorState` sent to frontend carries `progress_label` for determinate, omits for indeterminate; `build_kotlin_mirror_json` remains label-free; demonstrate failure of production wiring before T090 (FR-011/FR-013, Principle III) (`partial`) (landed: commits bc799e2 (red), 0e66f7b (green) — the Kotlin-label-free clause superseded by T089/T090; reconciliation open in T099)
- [ ] T095 VERIFICATION (only this attests): run `./scripts/verify.sh`, record exit status + `verify.sh` log path + `cargo fmt --all -- --check` + `cargo clippy --workspace ...` exit + `cargo nextest list --workspace` non-empty + `cargo tauri android build --target aarch64` APK hash; attach T051/T052 trial sheets if available; commit `verify(004): T095 final verification` (Principle V; do NOT declare success by implementer assertion) (`missing`)

**Dependencies / order:** T091 (clippy) independent; T089→T090/94 (label wiring); T092→T093 (test inventory); T095 last (requires T091 + T093 + device/T051/T052 evidence if produced). All property/red-first tasks block their implementation tasks per Rule 1/2.

---

## Phase 15: Convergence

> Source: `spec.md` FR-011/FR-013/FR-015/FR-017/FR-018, `contracts/surface.md`, constitution v1.8.0 (Principles III/V/IX/XI), `plan.md` offline/purity decisions, Phase 14 evidence. Assessment agent `ses_ea9b3aed5ffes9362Ody0zuGRY` (read-only). Findings ordered CRITICAL/HIGH first; test tasks block implementation per Rule 1/2.

### Convergence Findings (in-session, before append)

| ID | Gap Type | Severity | Source | Evidence | Remediation (tasks below) |
|----|----------|----------|--------|----------|---------------------------|
| F1 | missing | CRITICAL | FR-018 | `DocIngest.kt:308–319` reflectively calls `saveSessionBookmark` which exists nowhere; `lib.rs` never persists previous fingerprint before switching | T096 |
| F2 | missing | HIGH | FR-017 / SC-008 | `src-tauri/Cargo.toml` has no `diag` dependency; `DiagWriter::append` never called from production | T097 |
| F3 | unrequested | HIGH | FR-015 / IX / T047 | `status_view.html:8` imports Google Fonts over HTTPS; `test_noscope.rs` scans only `.rs`/`.kt`; manifest declares INTERNET | T098 |
| F4 | contradicts | HIGH | FR-011/FR-013 · contracts/surface.md | `publish_mirror` + `test_publish_payload` assert label-free; `build_kotlin_mirror_json` + `test_purity` assert label present | T099 |
| F5 | partial | MEDIUM | FR-011 / FR-013 | `pt_br_progress_label` keeps English `(content-offset)` fragment; contract says `(deslocamento de conteúdo)` | T100 |
| F6 | partial | MEDIUM | FR-005 · contracts/surface.md:10 | 3 reason lines exceed ≤40-char limit: TransientPaused 47, VoiceUnavailable 43, DecodeFailed 43 | T101 |
| F7 | contradicts | MEDIUM | Principle III/V · workflow | tasks.md has two Phase 11 headers, T73 typo, 36 unchecked tasks (many landed) | T102 |
| F8 | partial | LOW | FR-011 | Title source is first heading of any level, not H1; fallback is "Document" not filename | T103 |
| F9 | contradicts | LOW | contracts/surface.md:36 | `status_view.html` uses red `#FF4D63` for Stop button; contract reserves red for failure-only | T104 |
| F10 | partial | LOW | T060 | Repo-root `contracts/` missing `resume-algorithm.md` | T105 |
| F11 | partial | LOW | FR-015 | Manifest declares INTERNET permission | T106 |
| F12 | partial | LOW | Rule 5 | Working tree dirty: 17 modified files + untracked dirs | T107 |

**Metrics:** 12 findings (1 CRITICAL missing, 3 HIGH, 3 MEDIUM, 5 LOW). 2 missing · 3 contradicts · 6 partial · 1 unrequested. FRs checked: 19 · SCs checked: 10 · Plan decisions: 20 · Constitution principles: 11 + constraints. No code-rigor violations (unsafe only at sanctioned JNI with #[expect]; no unwrap/expect in crates/*/src; no _ => catch-alls; closed sets intact).

### Phase 15 Tasks (append-only; zero renumber of T001–T095)

- [ ] T096 [P] (CRITICAL) Wire real durable bookmark save on second-document pick per FR-018: remove the dead reflective `saveFirstBookmarkIfPlaying` call in `DocIngest.kt:308–319`; implement the durable put in Rust (`src-tauri/src/lib.rs` `open_ingested_document` / `open_document_bytes`) before the new snapshot write — persist the previous fingerprint's last-spoken position with `Paused`/`Interrupted` end_kind via `BookmarkStore::put`; never delete the first bookmark; never auto-start the second document; covered by `prop_second_document_preserves_first_bookmark` (commit `c89406b`) (`missing`)
- [ ] T097 [P] Wire the diagnostic record into production per FR-017: add `diag` dependency to `src-tauri/Cargo.toml`; call `DiagWriter::append` / `RecordEvent` from session lifecycle (session start/stop, transport actions, per-section progress, per-rate changes, buffer states, interruptions with measured latencies, failures); ensure the JSONL record is written to app-private storage with 30-day + 10 MB trim; covered by `crates/diag/tests/contract_trace.rs` + `prop_trace.rs` (`missing`)
- [ ] T98 [P] Remove external font import from `src-tauri/src/status_view.html:8` (Google Fonts HTTPS); extend `test_noscope.rs` guard to scan HTML/external URLs; remove `android.permission.INTERNET` from `AndroidManifest.xml` for pure-offline build per FR-015/Principle IX (`unrequested`)
- [ ] T099 [P] Reconcile the lock-screen payload contract contradiction: update/remove the stale `publish_mirror` helper in `src-tauri/src/lib.rs` and `test_publish_payload.rs` to match the T090/T094 contract (`progressLabel` present for determinate, absent for indeterminate); delete or update the contradictory task text in Phase 11/12; covered by `test_purity` + `test_kotlin_mirror_payload` (`contradicts`)
- [ ] T100 Translate the `(content-offset)` fragment in `pt_br_progress_label` to pt-BR (`(deslocamento de conteúdo)`) per `contracts/surface.md:9`; or record a deliberate exception with rationale; covered by `test_progress_label` + `test_kotlin_mirror_payload` (`partial`)
- [ ] T101 Shorten the 3 reason lines exceeding ≤40-char limit in `crates/player/src/interruption.rs:47–54`: `TransientPaused` (47 ch), `VoiceUnavailable` (43 ch), `DecodeFailed` (43 ch); contract says ≤40 chars wins over ≤8 words; covered by `test_setup_interruption_shape` + `the_focus_pause_reason_exceeds_the_content_text_limit_and_the_conflict_is_recorded` (`partial`)
- [X] T102 Deduplicate `tasks.md`: remove the duplicate `## Phase 11: Convergence` header; fix the `T73` typo; mark landed tasks [X] where git log confirms completion (T070, T076–T082, T083–T088, T089–T095); close out stale entries; do NOT renumber existing IDs (`contradicts`) (landed: this commit — duplicate Phase 11 merged into one section, `T73` → `T073` typo resolved by the merge, 20 landed tasks closed out with commit evidence)
- [ ] T103 Honor FR-011 title sourcing: use first Markdown H1 heading (not any level) for `.md`; filename without extension for `.txt`; fallback to filename (not literal "Document"); covered by `test_surface_us1` + `test_purity` (`partial`)
- [ ] T104 Replace the red `#FF4D63` token for the Stop button in `src-tauri/src/status_view.html` with a non-reserved token per `contracts/surface.md:36` (red is failure-only); covered by `test_purity` (`contradicts`)
- [ ] T105 Copy `contracts/resume-algorithm.md` to repo-root `contracts/` per T060; or narrow T060's claim to the 3 contracts actually copied (`partial`)
- [ ] T106 Remove `android.permission.INTERNET` from `AndroidManifest.xml` per FR-015/Principle IX (pure-offline build); covered by `test_noscope` + `check-manifest.sh` (`partial`)
- [ ] T107 Commit outstanding working-tree edits atomically per Rule 5: 17 modified files (clippy fixes, `surface.rs`, `status_view.html`, `MediaMirror.kt`, engine-core tests) + untracked `.github/`, `supply-chain/`, `.proptest-regressions`; stage and commit per logical unit with detailed summaries (`partial`)

**Dependencies / order:** T096 (FR-018 auto-save) blocks device T051; T097 (diag wiring) blocks T052; T098/T106 (offline purity) independent; T099 (contract reconcile) independent; T100/T101 (label/line fixes) independent; T102 (tasks.md cleanup) independent; T103/T104 (surface fixes) independent; T105 (contract copy) independent; T107 (commit cleanup) last. All [P] tasks may run in parallel on disjoint files; shared tooling (cargo) serialized per user rule.

---

## Phase 15 Addendum: Convergence

> Source: `cargo clippy` gate evidence during Phase 15 implementation (clippy 1.98 `lint_groups_priority`), constitution v1.8.0 Rust Code Rigor MUST (`clippy::all`/`pedantic` deny, gate 4/5). Regression introduced by commit `811e23b` (T095), which reverted the `priority = -1` overrides on the `nursery`/`cargo` lint groups. Appended as a new traceable task; no existing task rewritten or renumbered.

### Convergence Findings (in-session, before append)

| ID | Gap Type | Severity | Source | Evidence | Remediation (tasks below) |
|----|----------|----------|--------|----------|---------------------------|
| F1 | contradicts | CRITICAL | Constitution Rust Code Rigor MUST · gate 4/5 | `Cargo.toml:28-29` — `nursery = "warn"` / `cargo = "warn"` carry an implicit priority of 0, colliding with individual lints (`multiple_crate_versions = "deny"`); clippy 1.98 `lint_groups_priority` errors on EVERY crate: "lint group `cargo` has the same priority (0) as a lint" | T108 |

**Gate evidence (this run):** `cargo clippy -p doc-structure --all-targets -- -D warnings` → exit 101 with 2 `lint_groups_priority` errors; `cargo clippy -p cleanroom-core` → package not a workspace member (crate is excluded from the workspace, so its metadata fix from `0ccd303` is inert). Commit `811e23b` diff confirms the `priority = -1` removal; `1e8ef5b` (T070) had added it.

### Phase 15 Addendum Tasks (append-only; zero renumber of T001–T107)

- [ ] T108 [P] (CRITICAL) Restore the lint-group priority overrides in the workspace `Cargo.toml` lint table: `nursery = { level = "warn", priority = -1 }` and `cargo = { level = "warn", priority = -1 }` so specific lints (`multiple_crate_versions = "deny"`, `unwrap_used`/`expect_used`/`panic` denies) override the groups; this REVERTS the regression introduced by `811e23b` and satisfies clippy 1.98 `lint_groups_priority`; gate: `cargo clippy --workspace --all-targets -- -D warnings` exits 0; NEVER weaken any lint level (Constitution Rust Code Rigor MUST) (`contradicts`)

