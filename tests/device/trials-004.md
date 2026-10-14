# T051 — On-device trials, `004-ressoa-dogfood-build` (procedure set; T051 executes, T052 + Phase 8 attest)

Per `specs/004-ressoa-dogfood-build/tasks.md` Phase 8 / `plan.md` §Testing /
constitution Principles III, V, VIII. This file is the **procedure set only** —
it records no verdicts. Conventions, the recording sheet, and the observation
channels are in `tests/device/README.md`; the 001-era procedure set is
`tests/device/trials.md` and stays untouched.

**Scope**: reference phone only — Redmi 2201117TY (Snapdragon 680, Android 13 /
API 33) per `specs/003-reference-device/`. Nothing here runs on any other
device; a pass on a faster device is conforming, a fail on a slower one is not
evidence of failure (the reference class is normative).

**Fixture**: `tests/device/fixtures/fixture-pt-2h.md` — committed Portuguese
Markdown, 62 424 bytes, headings + numbers/currency, ~2 h at 1.0x, ~30 min at
4.0x. SC-001 runs at every rung including 4.0x (measured per rung, never
combined: 7 × 30 min individually, not one 210-min trial).

**Every section is 10 trials unless stated otherwise. Any single trial failure
fails the criterion** — stop the section, capture evidence, report the partial
count (README §5 recording sheet).

---

## T051-01 · SC-001 — Commute loop (locked, screen off, lock-screen controls only)

**Requirement**: one 30-minute session per rate rung (7 separate trials), zero
unlocks, continuous audible reading.

**Procedure — per rung r ∈ {0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0}**:
1. Open `fixture-pt-2h.md` via the system file picker (not a path the app owns).
2. Press Play; confirm audible audio.
3. Press power (screen off); lock manually.
4. Leave untouched 30 minutes.
5. Wake to the lock screen **without unlocking**; confirm media controls are
   present and audio is still playing (mirror transport `playing` via
   `adb shell dumpsys media_session`).
6. Pass iff: continuous audio for the full 30 min, zero unlocks, controls
   reachable. Repeat independently for each of the 7 rungs.

## T051-02 · SC-002 — First audible ≤ 3 s (cold start)

**Requirement**: 10/10 cold-start trials, first audible speech ≤ 3 s after Play.

**Cold start definition**: process-killed, fixture unopened, radios off.
**Procedure — per trial**: force-stop the app; confirm the fixture is unopened;
airplane mode on; open the fixture via the picker; press Play; measure
press → first audible speech at the transducer. Pass iff ≤ 3 s. Record the
measured value per trial (the host harness `test_play_latency.rs` measures the
adapter half only — the transducer half is this trial).

## T051-03 · SC-003 — Interruptions (10/10 per class)

**Requirement**: 10/10 per class — nav-transient → duck ~20% + restore;
call/alarm → pause + 3 s reason, never auto-resume; headphone removal → pause
≤ 1 s, never on speaker; permanent loss → pause + 3 s, never auto-resume.

**Procedure — per class (4 classes × 10 trials)**:
1. **Nav-transient**: start a navigation prompt (or a MAY_DUCK focus requester);
   confirm the reading ducks to ~20% and continues; end the prompt; confirm
   restore without place loss (never owes rewind).
2. **Call/alarm**: place a call (or trigger an alarm); confirm pause ≤ 1 s with
   the pt-BR reason line visible ≤ 3 s; confirm NO auto-resume after the
   interruption ends.
3. **Headphone removal**: unplug headphones mid-playback; confirm pause ≤ 1 s,
   never routed to the speaker.
4. **Permanent loss**: revoke the audio route permanently; confirm pause + 3 s
   reason, never auto-resume, latest reason wins.

## T051-04 · SC-004 — Blind confirmations (10/10 per type, sounds-on and -off)

**Requirement**: no-look, 10/10 per type (play, pause, skip, rate-step, warning,
failure) via earcon + state change + haptic — with sounds ON and with sounds
OFF (DND/muted).

**Procedure — per type (6 types) × per audio mode (on/off)**: cover the screen
(opaque); trigger the control; the blind tester names the confirmation from the
earcon + haptic + state change alone. Pass iff 10/10 correct identifications.
The rate-step type is value-blind: the tester confirms the STEP, never the
rung value.

## T051-05 · SC-005 — Resume matrix + replay caps

**Requirement**: 10/10 per case — move+rename ⇒ offer continue-from-position;
edit content ⇒ no resume; in-app Stop ⇒ next open starts at beginning;
deliberate-pause offer window 3 s (silence = continue from stored); kill/power-loss
⇒ ≤ 60 s replay (interruption replay ≤ 30 s).

**Procedure — per case**:
1. **Move + rename**: pause mid-document; move and rename the file on disk;
   reopen via the picker; confirm the continue offer (10/10) and the position.
2. **Edit**: pause; edit the document's bytes; reopen; confirm NO resume, start
   at 0.0 (10/10).
3. **In-app Stop**: Stop on the single in-app screen; reopen; confirm start at
   the beginning (10/10).
4. **Offer window**: deliberate pause; reopen; confirm the in-app 3 s
   continue-vs-start-over dialog; silence = continue from stored.
5. **Kill / power-loss**: kill mid-playback (or power-cycle); reopen; confirm
   the explicit lock-screen resume control (never auto-start) and replay
   ≤ 60 s (interruption ≤ 30 s).

## T051-06 · SC-006 — Lock-screen purity + dismissal refusal

**Requirement**: 10/10 inspections across playing/paused/section-change/
rate-change/interruption — title + section + progress only, zero prose/time/
rate/voice leaks; dismissal attempted while playing refused 10/10.

**Procedure — per state (5 states × 10)**: screenshot/lock-screen inspection at
each state; pass iff the display projection is exactly title + section +
progress and no prohibited field appears. Separately: attempt to swipe the
notification away while playing; pass iff refused 10/10.

## T051-07 · SC-007 — Rate persistence + unpitched

**Requirement**: chosen rung persists across sessions/documents (10/10 per
rung); voice quicker/slower never pitched (PITCH_TOLERANCE_CENTS = 50) + blind
listener check.

**Procedure**: for each rung, set it, kill the app, reopen (same and a different
document); confirm the rung persisted. Blind listener check at each rung
(especially 2.0× → 4.0×): the pitch deviation stays within 50 cents — a blind
listener confirms no chipmunk/slowed pitch.

## T051-08 · SC-008 — Record bounds + zero prose

**Requirement**: `adb pull` the app-private JSONL; the full trace is present
(session/transport/per-section/per-rate/buffer/interruptions + measured
latencies + failures); retained within 30-day/10 MB bounds; zero entries with
prose.

**Procedure**: run the session set; `adb shell run-as <pkg> cat files/diag.jsonl`
> pull; verify the kinds and the measured latency keys (`play_to_audio_ms`,
`removal_pause_ms`, `replay_secs`); verify bounds and zero prose. This is also
the input to T052 (`diag scan`).

## T051-09 · SC-009 — Offline (airplane mode)

**Requirement**: all radios off → 15-minute session on the reference phone
using the fixed fixture → 10/10 trials with zero network-dependent voice errors,
playback uninterrupted start to end.

## T051-10 · SC-010 — In-app screen purity + Stop clears

**Requirement**: 10/10 inspections — title, section, progress, speed control,
transport (play, pause, skip, Stop) only; zero prose/time/rate/voice leaks; Stop
clears the stored position 10/10; all user-visible strings verified pt-BR.

## T051-11 · Lock-screen privacy (manual inspection)

**Requirement**: enable the phone's lock-screen privacy-hide-contents setting →
the app obeys, the lock screen hides contents, no workaround attempted. Record
the inspection note (quickstart §3 item 11).

---

## Per-SC recording

Record per-SC 10/10 sheets per README §5 (trials, partial counts, evidence
paths). These sheets are the input to the Phase 8 verification tasks (T048–T052),
which alone attest. No agent self-attests.
