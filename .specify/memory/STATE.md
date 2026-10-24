# Session state — 004-ressoa-dogfood-build (shell bring-up wave)

> **RESUME POINT (2026-10-24) — 005-adopt-design-sona, Phase 1 (Setup) COMPLETE (T001–T003).**
> Branch `005-adopt-design-sona` cut at dev HEAD `226c06c`
> (`226c06cdd211d5541f85926aa0e131ac12cc6f84`). T001 deviation disclosed: the
> tree was DIRTY at branch creation (33 pending clarify/plan/assessment
> artifacts from clarify runs + plan were never committed) — remediated ON the
> branch by 33 per-file commit-hook commits (tasks.md `edf6190` first, with
> the two pending refinements + the US3 parallel-example paths corrected to
> the child-module layout; then spec/plan/research/data-model/quickstart/3
> contracts/checklist; 20 clarify research notes; 5 assessment records; then
> memory `cdad44d`), giving the clean tree the task requires before the
> baseline measurement.
>
> **T002 baseline — three runs, all recorded** (`/home/luis/development/.mdreader-005-tr/logs/`):
> (1) run 1 (`baseline-2026-10-24.log`) RED at gate-5: gates 1–4 PASS,
> cargo deny advisories named RUSTSEC-2026-0385 (`ctutils` 0.4.3 via digest
> 0.11.3 ← hmac/sha1/sha2/pbkdf2 ← zip 9.0.5 ← (build) sherpa-onnx-sys).
> Provenance verified: NO lockfile commits since `9501bba` — the RED is a
> newly published advisory, not a tree change (zip 9.0.5 entered at
> `ea59333`, the latest-stable refresh). Fix `0c2b6a7`:
> `cargo update -p ctutils` → 0.4.5 (single package; 42 unchanged; fail-
> forward per constitution IV — no waiver, no auto-substitute; deny all-ok
> + audit 2 allowed warnings). (2) run 2 (`…-run2.log`): gate-5 green, then
> `verify.sh` died with a bash syntax error at :335 — an ORPHANED `fi`
> leftover from `8635535` (T164 waiver retirement), latent because the
> Oct-22 green runs predate that commit and run 1 aborted before the parser
> reached the line. Fix `9b9a539`: delete the stray token (`bash -n` clean;
> diff vs 9501bba = exactly the 57-line waiver removal, nothing else).
> (3) run 3 (`…-run3.log`) **GREEN, VERIFY_EXIT=0**: gate-1 format PASS ·
> gate-2 tests PASS (906 tests run: 906 passed, 2 skipped — the same
> inventory as the 004 attestation) · gate-3 documentation PASS · gate-4
> lint-drift-clippy PASS · gate-4 lint-drift PASS · gate-5 supply-chain
> PASS · gate-6 deny-list PASS · gate-7 closed-set PASS · gate-8-shell
> PASS · gate-8-full DEFERRED (owner=synthesis) — never a pass. Run 3 is
> the recorded green baseline of the feature.
>
> **T003 anchor audit — 19/19 PASS** (explore agent, per-anchor evidence):
> `enqueue_earcon` backtaint :511, FIFO `push_short_mix` :288-290
> (consume_into :318-342 pops front in lockstep with speech = the residual
> FIFO serialization D3 cites); `EarconKind` :10-26 with 7 variants
> (Play/Pause/SkipFwd/SkipBack/Warning/Failure/RateStep); `EventKind`
> :66/:69-88 exactly nine (SessionStart/SessionEnd/TransportChange/
> FocusTransition/OutputDevice/Failure/Refusal/Trim/Gap), no catch-all;
> status_view.html anchors all present (slates :21-22, white fill :26,
> --warning/--failure :29-30, shimmer gradient :400, @keyframes
> indeterminate :748-752, rgba tints :179-180, reasonBanner alert+polite
> :252 same element, «Seção: —» :234, speedDots aria-hidden :245,
> #progress :249 NO aria attrs, ZERO :focus-visible, ZERO
> prefers-reduced-motion, `infinite` only :402); status_view.rs 534 lines
> pure render; MediaMirror.kt constants block :701-744 (DISPLACED from the
> tasks' :728-743 citation) with PROGRESS_FILL=0xFFFFFFFF :728 +
> setContentText guard :1419; frontendDist "../dist" :6; verify.sh +
> expected-tests.txt (99 lines, no surface_audit/earcon_burst entries);
> logo-primary-concha-r.svg census-clean (0× #B9A6F5, 0× #5AD6A8);
> tokens.md §1 ledger :10-20 with harp #E8C46A :17; 004 contracts
> earcon.md/surface.md present; DESIGN-SONA CHANGELOG.md/VERSION ABSENT
> (as T014 expects); src-tauri/build.rs exists (auto-detected); NO
> surface_audit module yet; backend.rs has one #[cfg(test)] mod :664,
> proptest dev-dep :32 (not yet imported in src); constitution v1.12.0
> confirmed; diag contract_schema + prop_no_prose live under
> crates/diag/tests/ (writer.rs and scan.rs have NO inline tests).
>
> **Task-level bugs / adjudications recorded (dispatch inputs):**
> (i) T007(i)'s literal set claim is UNSATISFIABLE over contract A1#3's
> pinned slice :728-743 even post-T008 — that slice permanently carries the
> two admitted --signal-* tokens (FFD400 :736, FF4D63 :743) and the accent
> usage site (:1407 setColor(DESIGN_SONA_TITLE)) is OUTSIDE it; the ONLY
> DESIGN_SONA_ usage in MediaMirror.kt is :1407 (FILL/WARNING/FAILURE/
> SECTION/TRACK are declared but UNWIRED). Adjudication (contract wins):
> the accent twin asserts the fill token == ledger --sona-voice (C1:
> literal-white veto = today's NAMED RED) plus non-signal declared/used
> hexes ⇒ exactly {F2F0E6}, with FFD400/FF4D63 recognized as sanctioned
> --signal-* declarations; repeat verbatim in T007's commit summary.
> (ii) T005/T006's «Taxa: 1.0×» bench leak is ABSENT from the shipped
> dist/index.html (offline-synth bench, 4 948 B) — the S2 byte-identity
> mandate stands; the Taxa rationale no longer binds to present content.
> (iii) T011/T012 edit points: writer.rs has no inline tests — the
> contract_schema twins live in crates/diag/tests/. (iv) T021's
> versionFooter citation: the id exists only at :254 (:223 is the
> .version-footer class rule).
>
> NEXT: Phase 2 (T004 audit scaffolding → T005 RED served-identity → T006
> wiring), then the [P] agent waves (Agent B: accent.rs + MediaMirror.kt ·
> Agent C: crates/player + crates/diag).

> **RESUME POINT (2026-10-24) — 005-adopt-design-sona, clarify run 5 (batch mode).**
> User-directed protocol again (2 background agents per candidate: code dive +
> research note; batch answers at the end). 3 candidates (E1–E3). Today's date
> for all registrations: 2026-10-24.
> (a) Integrated autonomously (evidence-converged, veto reversers recorded in
> each bullet): **E1** — `--sona-etwahl` = FOCUS RING (`:focus-visible` on
> enumerated interactive controls); the «fluxo ativo» role is RESERVED
> non-painted for the full Lente; edits: FR-001 role map, US3 narrative
> («turquesa para o anel de foco»), SC-001 item (iii) exclusivity (off-role
> `#8FD8E8` paint = fail), batch-Q1 task gained item (5) (the 7 stale
> etwahl-progress rows: `tokens.md:16/:28/:47/:65`, `components.md:17`,
> `DESIGN-SONA.md:60/:85`, `preview.html:238`), C1 record annotated. Evidence:
> 0 occurrences of `#8FD8E8`/`etwahl` in audited files; 0 `:focus-visible` in
> `status_view.html`; recipe is the design dir's own (`tokens.md:20`).
> **E2** — CLOSED MOTION RULE: the indeterminate sliding pill (C5) is the ONLY
> continuous motion on the driving surface and freezes at a visible track
> position under `prefers-reduced-motion: reduce`; new SC-001 item (v) host
> check (media query present, zeros `animation`/`transform`, no new paint/element);
> SC-007 gains the freeze leg («Remover animações» on, announcements unchanged
> 10/10); FR-008 carries the rule clause; SC-006 pointer includes (iv)+(v).
> Yellow pulse TTL-bounded (3 s), failure = static red (no pulse), banner
> slide one-shot ≤200 ms, no other continuous loop. WCAG 2.2.2 strict reading
> + SCR40/C39 OS-setting as the sanctioned mechanism; census-clean (zero
> painted pixels). Historical note: the pulsing yellow dot does NOT exist in
> shipped code yet (US1 AC2 is the first implementation).
> Research notes: `specs/005-adopt-design-sona/research/clarify-e{1,2}-*.md`.
> (b) RESOLVED — maintainer ACCEPTED the recommendation ("accept the
> recommendations", 2026-10-24), batch E option A: **E3** — SC-002
> haptic side-channel disposition. DIVE FINDING (unchanged): haptics are DORMANT in the
> shipped binary (Rust always publishes `haptic=None` — `lib.rs:6047`; the
> 7-pattern table + `VibratorManager` in `MediaMirror.kt` never fire; no app
> off-switch; system haptic-feedback setting does NOT gate direct
> VibratorManager calls). Research: cross-modal redundancy fuses/confounds
> audio identification (Brewster segregation doctrine; ISO 7731 unimodal
> unit-of-test; two-pulse discrimination peaks at 50–700 ms ISI vs the 20 ms
> shipped gap; 10/15 ms buzzes below the ~50 ms perceivable floor).
> `research/clarify-e3-sc002-haptic-side-channel.md`.
> **INTEGRATED:** SC-002 haptic-disposition clause (audio gate sound-only;
> app haptics inert; administrator no-buzz check at the pilot; the haptic
> half stays in 004's T051-04 sounds-off condition, never this gate),
> B1 record annotated (patterns = Kotlin render table, UNWIRED —
> `haptic=None` published), Clarifications bullet E3, research-note
> correction addendum. ALL of E1–E3 resolved — spec is plan-ready.
> (c) Eliminated pre-spawn by direct evidence (never became candidates):
> DESIGN-SONA has NO haptic prescriptions (all «vibração» hits are visual
> metaphor) and NO soundscape/ambient audio content → Option A scope + closed
> inventory already govern; `--sona-hush` `#A9B8CE` = shipped section color
> (bit-exact, no reconciliation needed).
> Checklist re-validated after E3: 16/16 → 16/16, no marker changes. Spec is
> plan-ready. PLAN run complete (2026-10-24, same session): `plan.md` from
> the template (Summary / Technical Context / Constitution Check PASS — no
> violations, no Complexity rows / Project Structure / Post-Design re-check)
> + `research.md` (R1–R13) + `data-model.md` (SonaToken 10-member lexicon,
> SurfaceInventory, NotificationSurface, EarconConfirmation 7 confirmations,
> RateLadder, ProgressIndication, MovementSpec, HapticDisposition, §VII diag
> extension, DesignDirState, ContrastPair) + `contracts/` {status-surface,
> earcon-mapping, surface-audit} + `quickstart.md` (host gates via the new
> `surface-audit` checks + device trials incl. the SC-002 protocol and the
> SC-007 freeze leg). `hooks: {}` — skipped per rules. Working tree still
> UNCOMMITTED (clarify + plan artifacts). NEXT: `/speckit.tasks`.

> **RESUME POINT (2026-10-24) — 005-adopt-design-sona, clarify run 4 (batch mode).**
> Ran with the user-directed protocol (2 background agents per candidate: code
> dive + research note; batch answers at the end). 5 candidates (D1–D5), ALL
> RESOLVED. (a) Integrated autonomously (evidence-converged): D1 — restyle is
> COLOR-ONLY (typography/spacing = design-dir prose, not `--sona-*` tokens;
> FR-001 clause + Session bullet); D4 — launcher icon explicitly OUT of scope
> (brand surface; regen-clobber; zero driving impact; Assumptions bullet).
> (b) Maintainer ACCEPTED the recommendations ("accept the recommendations")
> for the batch and all three were integrated in the same run: D2 — admit
> `#FFD400` `--signal-caution` / `#FF4D63` `--signal-failure` as role-locked
> signal paints (audited hex set 8→10; presence off-role = violation; never
> adjacent, 2.25:1; FR-001 + SC-001 + clarif bullet; values match shipped
> `status_view.html:29-30,179-180`; design-dir errata `tokens.md:48` /
> `components.md:18` ride the batch-Q1 DESIGN-SONA maintenance task); D3 —
> FR-009 truncate/replace «latest wins» sub-rule (earcon↔earcon; additive mix
> stays earcon↔narration-only; system Ostinato/Sforzando interrupt; typed
> truncation diagnostic counter; `short_mix` stays single-channel) + SC-002
> rapid-sequence sub-test (2–3 heterogeneous actions in 100–150 ms); D5 — the
> value-blind promise is now VERIFIED: SC-001 item (iv) semantics inventory
> (aria-labels, role="progressbar"+valuenow/valuetext, hidden role="status"
> live region for the ladder value, dots stay aria-hidden, reason banner
> re-typed from mismatched alert+polite to status) + NEW SC-007 TalkBack
> device trial (10/10 announcements; the spec's 7th success criterion).
> Research notes: `specs/005-adopt-design-sona/research/clarify-d{1..5}-*.md`.
> Checklist re-validated: 16/16 → 16/16, no marker changes. Spec is
> plan-ready. NEXT: `/speckit.plan`.


> **RESUME POINT (2026-10-22) — read this first.** Phase 17 is COMPLETE
> (T140–T163, all committed) and the final verification has RE-ATTESTED the
> tree at `9501bba` ("test(004): T163 model the FR-007 five-second cue in the
> canonical guard oracle") in the attestation commit `21892dd`
> ("verify(004): T095 final re-attestation (deterministic gate 2)"); the task
> markers are committed on top of it (`ce54aea`).
>
> **Landed in Phase 17 (24 tasks, all committed):** T140 `f30feca` (RED
> resume) · T141 `aa4ecf4` (resume restarts the stream) · T142 `59a12a2` (RED
> session-live) · T143 `97f0046` (session live from the first chunk) · T144
> `80803f5` (RED offer-dialog reachability) · T145 `88e4f2d` (offer dialog
> reachable) · T146/T154 `173b7d2` (DeviceRemoved diag contract row) · T147
> `3fc7934` (offer route read-only) · T148 `7bb8b25` (in-app Play functional) ·
> T149 `a651c61` (bounded resident queue) · T150/T153 `74fbb95` (surface
> contract record + shell contract paths) · T151 `d17da93` (comment-aware
> literal scanner) · T152 `8753164` (first consumed sample latency) · T155
> `07f59a2` (parked open re-check cadence) · T156 `fa04ca6` (purity-gate
> allowlist) · T157 `507a991`/`9786886`/`2a94589` (expected-tests sync) · T158
> `024d55e` (producer back-pressure) · T159 `5af0f33` (buffer bound in the diag
> record) · T160 `3e4fd0a` (resume divergence reachability) · T161 `1ac0ae3`
> (FR-003 first-audible observation restored) · T162 `b03f1a2`/`5e0acaf` (stale
> gate-2 comment) · T163 `9501bba` (the canonical-guard oracle repaired).
>
> **Final verification (the only thing that attests) — DETERMINISTIC, TWICE.**
> `scripts/verify.sh` was run TWICE on the attested tree, back to back, and
> BOTH runs exited **0** with a byte-identical per-gate result sequence — the
> determinism claim is DEMONSTRATED, not asserted:
> run 1 (03:27:34 – 03:43:03 -0400, 7 265 lines, `VERIFY_EXIT=0`) and run 2
> (03:50:02 – 04:05:58 -0400, 7 261 lines, `VERIFY_EXIT=0`); the exit status is
> the SCRIPT's (`PIPESTATUS[0]` under `bash -c`), and the two per-gate result
> sequences compare EMPTY under `diff`. Gates 1–7 + gate-8-shell PASS in both
> runs (906 tests run, 906 passed, 2 skipped — up from 902, T163's four new
> pins); `gate-8-full: DEFERRED (owner=synthesis)` (SC-044 open — NEVER a
> pass); `check-manifest.sh` and `check-shell-purity.sh` PASS in both runs.
> Standalone: clippy exit **0** (also as gate 4's fail-closed
> `lint-drift-clippy`), fmt exit **0**, nextest inventory **99 live / 99
> pinned** with the two-way diff empty in BOTH directions, `cargo deny` exit 0
> (`advisories ok, bans ok, licenses ok, sources ok`), `cargo audit` exit 0
> with the 2 allowed warnings. The RUSTSEC-2026-0381 waiver (Exceptions Log
> entry 13, PROVISIONAL, constitution v1.11.0) holds its mechanical bound: the
> COMMITTED lockfile carries exactly `webpki-roots` 0.26.11 + 1.0.10.
> Gate 9 `cargo tauri android build --target aarch64` exit 0 — APK sha256
> `2e3555f3…ece08f5`, AAB sha256 `c226ecc3…cc6d42b`, non-hollow (7/7 app
> classes in the dex). Logs of record:
> `/home/luis/development/.mdreader-t095-reattest-tmp/logs/` (`run1.log`,
> `run2.log`, `android.txt`, `clippy.txt`, `fmt.txt`, `nextest_list.txt`,
> `deny.txt`, `audit.txt`).
>
> **RESOLVED — the §3.1 flake is GONE.** The previously-disclosed
> intermittency (`player::prop_canonical_guard` red ~1 run in 40) was a STALE
> ORACLE, not a production defect, and T163 `9501bba` repaired the oracle with
> the production left byte-identical (`git diff --exit-code
> crates/player/src/rewind.rs` empty). The FR-007 5 s cue is now modelled, the
> `[cap, cap+5)` counterexample is pinned, the repair is mutation-proven
> (removing the cue turns 4 of 9 red), a 1 200 000-case equivalence sweep found
> zero disagreements, and a genuine proptest-recorded seed replaced the
> hand-copied hash that replayed no counterexample. T163's own 105 consecutive
> runs recorded 0 failures; all NINE `player::prop_canonical_guard` tests PASS
> in BOTH attested runs. Full detail in the attestation's §3.1.
>
> **Still open — device-only by design:** **T051** (the SC-001…SC-010 trials
> per `quickstart.md` §3 on the reference phone, Redmi 2201117TY) and **T052**
> (the on-device `diag scan` audit of the pulled JSONL). No trial sheet exists;
> both stay `[ ]` in `tasks.md` and the attestation explicitly does not claim
> them.
>
> **Next action, in order:**
> 1. T051 + T052 on the reference phone — the only un-attested criteria.
> 2. Do NOT push to `origin/main` without an explicit request (the branch is
>    far ahead — 514 commits).
> 3. Optional maintainer housekeeping: `cargo deny` reports the
>    RUSTSEC-2026-0381 ignore as `advisory-not-detected` (a warning, not a
>    finding) because the graph now carries the patched 1.0.10; removing the
>    two ignore records is a constitution exit-condition decision, not a
>    verification-session change.
>
> Housekeeping: `git stash list` is empty; the working tree is clean. Note the
> host's `/tmp` is a 2.0 GB tmpfs that another project (a `dotnet build`
> fixture of `livingroots-smoke-harness`) fills to 100 % — use a disk-backed
> `TMPDIR` for cargo/Gradle and the scratch space.

Updated: 2026-10-17 (Wave B device verification GREEN + user-heard audio
confirmed; zip-9 substitution complete — patch, lock, governance v1.8.0;
verify.sh gates 1–4 PASS, gate-5 fixed; F1/F3 wiring IMPLEMENTED (wave-b4):
player is_drained + 500 ms playback-watch task feeds session clock from
backend clock, republishes at consumed-unit crossings, commands
document-end teardown; host-fallback + idempotency tests green; Kotlin
ShellHost.commandDocumentEnd landed; 5 new src-tauri tests + 3 player
tests pass; awaiting device build/install + F1 end-transition check;
upstream PR #4059 open).

## Position
- **Wave A (bootstrap) GREEN ON DEVICE**: `PlaybackService` foreground
  (`isForeground=true foregroundId=1`, channel `mdreader_media`, "Ressoa" holder,
  `POST_NOTIFICATIONS granted=true`). Committed `7ca4c1d`.
- **Wave B (Rust→Kotlin mirror publish) VERIFIED ON DEVICE (wave-b3)**:
  - R8 regression FIXED: explicit `-keep` (`b856bfc`, force-added
    `gen/android/app/proguard-rules.pro`) + ShellHost docstring (`728557b`);
    wave-b3 dex contains `publishMirror` (grep count 1), `BUILD_EXIT=0`.
  - APK sha256 `04ff6c7e1b8a9d77e92012cfe25dd6d217c8ed34949bfa8554970724d67ce56a`
    installed (`adb install -r`); verified with pid 18124 run (~22:04).
  - Start publish: notification title "Teste de bancada" + section
    "Teste de bancada" + determinate progress 0/2000 (2 units), category
    transport, flags 0x62 (non-dismissable while playing) — **no JNI errors**.
  - Pause publish: MediaSession `state=2`, flags 0x62→0x60 (ONGOING cleared →
    dismissable via pause, per design). Stop publish: `state=1`.
  - MediaSession: `actions=54` (play/pause/prev/next, no Stop), active.
- **FINDINGS (bench/dogfood flow; the mirror transport itself is sound)**:
  - F1 no end-of-audio detection: `play()` (lib.rs:2165) enqueues the PCM,
    publishes once, returns; the backend exposes no drain signal
    (`backend.rs:300` is only `enqueue_pcm`); `dispatch_document_end` is
    reachable only via the Kotlin bridge intent, which cannot know (SC-035).
    Session stays Playing forever after the audio ends (2× confirmed:
    21:54 and 22:04 runs; notification frozen + non-dismissable). Intended
    wiring located (2026-10-16): Kotlin `Residency.documentEnd()` and Rust
    `dispatch_document_end` both exist; the missing link is Rust drain
    detection + the JNI command that invokes the Kotlin teardown.
  - F2 bench cannot restart after Stop: `▶ Play` → "illegal transition from
    stopped via play" (UI FALHA); only relaunch replays. The failed attempt
    also requestStart()s the paused stream (zombie track starved to 4265
    underruns at removal — seen in audio_flinger). Reclassified 2026-10-16:
    from-Stopped every verb is rejected by design (terminal; FR-032f
    "starting that document again requires opening the application") — the
    bench lacks a cold-start affordance; surface UX gap, not a product bug.
  - F3 single publish at start only: no mid-play/section-change updates
    (nothing calls `publish_current_state` during playback; the 20 s heartbeat
    only persists bookmarks) — matters for the T051-06 section-change state.
    Fix rides the same watcher: publish when `consumed_units_at(position)`
    changes.
  - **F1/F3 RESOLVED (wave-b4, 2026-10-17)**: `CpalBackend::is_drained`
    (player) + 500 ms playback-watch task (src-tauri) feeds the session
    clock from the backend clock, republishes at consumed-unit crossings,
    and commands the document-end teardown via
    `shell_jni::command_document_end_to_kotlin` → `ShellHost.commandDocumentEnd`
    → `Residency.documentEnd` → `dispatch_document_end`. Host fallback
    completes in-process; both paths idempotent. 5 src-tauri + 3 player
    tests green. Remaining: Android build/install + on-device F1 end-transition
    check + full verify.sh (gates 6–8 still pending).
  - F4 Stop re-posts nothing to the notification (content identical;
    MediaSession updates) — benign, noted.
  - Audio content flow CONFIRMED by the user's ears (pt-BR voice heard
    end-to-end in the bench runs). The MIUI `f`-counter freeze (~0.47 s while
    the mixer stayed fed, 0 underruns) was a log artifact, not a stall.
- **Gate 5**: zip-9 substitution COMPLETE — vendor `2177c67` (sha256
  `14bbeabb…`, one line zip 2→9), `[patch.crates-io]` + lock zip 9.0.4
  `02164b6`, ignores dropped + entry 11 deleted + entry 12 added, constitution
  v1.7.0 `8435260` (MSRV build-note: rustc ≥1.88), bzip2-1.0.6 admitted +
  v1.8.0 `9208578` (upstream PR #4059 keeps default features → admission
  survives the exit).
- Full `scripts/verify.sh` on the zip-9 tree: gates 1–4 PASS (775 tests);
  gate-5 FAILED pre-admission (libbz2-rs-sys bzip2-1.0.6) → admitted v1.8.0
  → targeted re-check green (`cargo deny` all-ok; `cargo audit` 0). Gates
  6–8 on this tree land with the next full run.
- T051 (human trials; needs picker = Wave C) + T052 pending.
- `gate-8-full: DEFERRED (owner=synthesis)` — unchanged (Exceptions Log 8).

## Next
1. **F1/F3 fix (wave-b4, tests first)**: (a) `crates/player` backend exposes
   `is_drained()` (queue empty with speech consumed) + unit test; (b) src-tauri
   ~1 s watcher while Playing — drained → JNI `command_document_end` →
   `ShellHost` → `Residency.documentEnd()` → `document_end` intent →
   `dispatch_document_end`; and `consumed_units_at` change →
   `publish_current_state`; (c) `shell_jni.rs` JNI entry + Kotlin `ShellHost`
   method. Then Android build + install + on-device check (2-segment doc:
   bar 0/2000 → 1000/2000 flip; `Completed` + notification reverted ~10 s).
2. Full `scripts/verify.sh` re-run (+ `scripts/expected-tests.txt` if new
   test targets) and wave-b4 commit.
3. Wave C — picker trigger (`DocIngest.pickDocument` / `handlePickResult`);
   then T051 sheets (human trials) + T052 diag scan (`run-as` fails on the
   release APK — needs debuggable variant or another pull route).

## Commits this session (004)
`c14e294` (T076 workspace clippy, this session) · `92129da` · `7ca4c1d` ·
`1849761` · `72f369e` · `18dd86a` · `7a25821` ·
`c6c3da4` (STATE) · `b856bfc` (R8 keep) · `728557b` (docstring) ·
`cc9164e` (STATE R8) · `2177c67` (vendor sherpa-onnx-sys) · `410a9b1`
(STATE wave-b3) · `02164b6` (patch + lock zip 9.0.4) · `8435260` (v1.7.0
governance) · `9208578` (v1.8.0 bzip2-1.0.6 admission).

## Verification summary
- T076 clippy (commit `c14e294`): `cargo clippy --workspace --all-targets --
  -D warnings` is clean for every file in this lane. Residual red is blocked
  on sibling lanes, NOT clippy debt: `src-tauri/src/lib.rs` (152 body + 20
  `multiple_crate_versions`), `tests/test_earcon_mirror.rs` (7, T074),
  `tests/integration/test_lockscreen_us4.rs` (8, T079). `cli.rs`/`surface.rs`
  committed via curated `update-index` blobs (sibling lane's rewrites left in
  worktree).
- Host: zip-9 tree gates 1–4 PASS (775 tests); gate-5 green after v1.8.0;
  6 mirror tests pinned.
- Device: Wave A + Wave B mirror path VERIFIED (start/pause/stop publishes;
  notification content + flags); F1–F4 bench findings recorded.
- Upstream: PR #4059 open (no upstream CI on PRs); zip 9 source-compatible.
- Brand / Design: Dupla Assinatura homologada (1º Concha Harmônica "R" primária,
  2º Ressonador Canônico selo motor). Arquitetura UI/UX multi-modo homologada:
  "A Lente Sinestésica" (P2) como Default Player; Cockpit Eyes-Free (P4) para
  condução; Terminal Suíço (P3) para sumário; Matriz Rust (P5) para diagnóstico.
  Assets PNG e SVG gerados via Storytold (PhotoCraft + VectorCraft + FigCraft).


