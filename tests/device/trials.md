# T086 — On-device trials (procedure set; T086 executes, T087 attests)

Per tasks.md Phase 8 / plan.md §Testing / constitution Principles III, V, VIII.
This file is the procedure set, not an attestation. Conventions, fixtures, the
recording sheet, and the routing table are in `tests/device/README.md`.

Every section is **10 trials unless stated otherwise**. Per-trial pass criteria
carry the exact bound from `spec.md`. **Any single trial failure fails the
criterion** — stop the section, capture evidence, report the partial count.

Observation channels (README §3): mirror position/transport via
`adb shell dumpsys media_session`; tracing lines on logcat tag `mdreader`;
record via `adb shell run-as <pkg> cat files/diag.jsonl` (debug builds).

---

## T-01 · SC-001 — Playback survives screen-off and manual lock

**Requirement** (SC-001 · FR-001): In 10 out of 10 trials, audio continues
playing for a full five minutes with the device screen off and manually locked.

**Setup**: corpus doc (a) (longer than 5 min), already parsed; playback running;
device unlocked.

**Procedure — per trial (run 10×)**:
1. Start playback; confirm audible audio and mirror transport `playing`.
2. Press power (screen off); lock manually (long-press power → Lock).
3. Leave untouched for 5 minutes.
4. Wake to the lock screen **without unlocking**; confirm media controls present
   and mirror transport still `playing`.

**Observe**: audible continuity across the 5 minutes; mirror state at t=5 min;
control reachability from the lock screen.
**Per-trial pass**: audio continued the full 5 minutes with no gap, restart, or
repeated phrase; transport `playing` at t=5 min; controls reachable while locked.
**10/10 rule**: all ten trials must pass; any failure fails SC-001.

## T-02 · SC-002 — Headphone / A2DP-loss pause within one second (both mechanisms)

**Requirement** (SC-002 · FR-016, FR-016a): In 10 out of 10 trials,
disconnecting headphones pauses playback within one second and audio is never
audible through the loudspeaker afterward; and in 10 out of 10 trials a
Bluetooth A2DP route loss pauses within one second even where the platform
delivers the removal only through the secondary device-removal signal
(`AudioDeviceCallback.onAudioDevicesRemoved`) rather than the becoming-noisy
signal, so neither mechanism is load-bearing alone.

**Setup**: playback running through wired headphones (Part A) / a connected
Bluetooth A2DP headset (Part B).

**Procedure — Part A, per trial (run 10×)**:
1. Start playback through wired headphones; confirm audio is audible at the
   headphones and mirror transport `playing`.
2. Unplug the headphones.
3. Time the pause from the moment of removal (operator count / harness
   timestamp of the `focus`/`transport` trace line).
4. Confirm silence at the loudspeaker for 10 s after the pause.

**Procedure — Part B, per trial (run 10×; odd trials normal, even trials with
the fixture switch `fault=disable_noisy_receiver` armed so only the secondary
device callback can deliver the removal)**:
1. Start playback through the Bluetooth A2DP headset.
2. Disconnect the A2DP route (turn the headset off / remove pairing).
3. Time the pause from the moment the route is lost.
4. Confirm silence at the loudspeaker for 10 s after the pause.

**Observe**: pause latency per removal; loudspeaker silence; which signal
delivered the removal (logcat).
**Per-trial pass**: pause within **≤ 1 s** of removal in every trial; zero
audio ever audible through the loudspeaker afterward; even-numbered Part B
trials prove the secondary signal alone still pauses within 1 s.
**10/10 rule**: all ten Part A trials and all ten Part B trials must pass; any
failure fails SC-002.

## T-03 · SC-003 — Duckable transient interruption: 15–25 % gain, keeps reading

**Requirement** (SC-003 · FR-013): In 10 out of 10 trials, a permitted
interruption reduces playback gain to 15–25 % of its normal level without
stopping the document, transport stays `Playing` with monotonically advancing
`position_secs` and zero rewind owed, and full volume returns when focus is
regained.

**Setup**: playback running; focus-injector APK installed.

**Procedure — per trial (run 10×)**:
1. Start playback; record the normal gain factor (baseline trace
   `focus.duck gain=1.00` state) and the mirror `position_secs`.
2. Injector requests `AUDIOFOCUS_GAIN_TRANSIENT_MAY_DUCK`; hold 8 s; abandon.
3. During the hold, read the app-applied gain factor from the
   `focus.duck gain=<factor>` trace line.
4. Confirm the mirror transport stays `playing` throughout and `position_secs`
   advances monotonically across the interruption.
5. After the regain, confirm the gain factor returns to 1.00 and the resume
   position equals the yield position (zero rewind owed — ducked playback never
   leaves `Playing` and owes no rewind).

**Observe**: gain factor during the hold; transport state; position sequence
before/during/after; post-regain gain.
**Per-trial pass**: gain factor within **0.15–0.25** for the whole hold;
transport `playing` throughout with monotonically advancing `position_secs`;
zero rewind on regain; full gain (1.00) restored on regain.
**10/10 rule**: all ten trials must pass; any failure fails SC-003.

## T-04 · SC-041 — Silence-demanding interruption pauses; rewind math on resume

**Requirement** (SC-041 · FR-013a, FR-015): In 10 out of 10 trials an
interruption that demands silence (phone call, alarm) pauses playback rather
than ducking, produces no audible document audio for the duration, and states
the same reason a permanent loss states; in 10 out of 10 trials a merely
permitted interruption still ducks instead of pausing (two-signal
differentiation); and in 10 out of 10 silence-paused transient trials, resume
lands at the start of the nearest completed sentence at or before the 5 %-back
cursor, with the cursor first moving back 5 % of the paused duration capped at
30 s, total repetition ≤ 30 s, zero mid-word landings.

> Precision note: FR-015 as amended 2026-10-11 (maintainer decision, T092)
> makes sentence-level snap the v1 precision, superseding the earlier
> paragraph/heading-snap wording; SC-041's parenthetical is read through that
> amendment. The stand-ins declare sentence granularity, so the sentence rule
> is the operative assertion.

**Setup**: playback running; focus-injector APK; stand-in audio with known
tone/gap map (harness fixture); synthetic pause of known duration.

**Procedure — Part A (silence-demanding), per trial (run 10×)**:
1. Start playback; confirm audible audio.
2. Injector requests `AUDIOFOCUS_GAIN_TRANSIENT` (silence-demanding) for 12 s;
   abandon.
3. Confirm: playback paused (not ducked), zero audible document audio for the
   whole 12 s, and the lock screen shows the pause reason statement (the same
   statement a permanent loss states, e.g. "Leitura pausada — outro app usando
   o áudio").
4. Regain focus; confirm playback resumes per the Part C math below.

**Procedure — Part B (differentiation), per trial (run 10×)**: repeat Part A's
shape with `AUDIOFOCUS_GAIN_TRANSIENT_MAY_DUCK` and confirm the outcome is
ducking (gain 0.15–0.25, transport stays `playing`) — the two signals produce
different outcomes.

**Procedure — Part C (resume math), per trial (run 10×)**:
1. Play to a mid-sentence position P; note P from the mirror.
2. Silence-demanding interruption for a known duration D (use D = 40 s in
   five trials and D = 15 min in five trials, so the 30 s cap is exercised
   both below and above saturation).
3. On regain, measure the resume position R from the mirror.
4. Assert the cursor moved back `min(0.05 × D, 30 s)` from the yield point P,
   and R is the start of the nearest completed sentence at or before that
   cursor (check against the stand-in's known sentence/gap map).
5. Assert total backward movement `P − R ≤ 30 s` (the cap is a hard ceiling;
   where the sentence containing the cursor starts earlier than the cap point,
   the resume lands at the cap point, snapped forward to the nearest following
   inter-word gap unless that gap lies beyond the cap point).
6. Assert the resume onset falls in a gap of the known-gap map, never in an
   energy peak (zero mid-word landings).

**Observe**: pause vs duck outcome per signal; pause-reason statement; D, P, R;
gap-map landing.
**Per-trial pass**: Part A — pause, silence throughout, permanent-loss reason
stated; Part B — duck, not pause; Part C — 5 %-capped-at-30 s cursor, sentence
boundary at or before the cursor, total repetition ≤ 30 s, zero mid-word
landings.
**10/10 rule**: all ten trials of each part must pass; any failure fails
SC-041.

## T-05 · SC-055 — Deferred grant and outright refusal (closed focus outcomes)

**Requirement** (SC-055 · FR-013b, FR-037): In 10 out of 10 trials where a
focus request is answered with a deferred grant, the document produces no audio
at all until the grant arrives and then plays from the point it would otherwise
have played from, with no rewind applied and no pause reason shown; and in 10
out of 10 trials where a focus request is refused outright, the refusal is
presented as a named failure with a stated reason on the lock-screen surface
and in the application, the session presents no playing state, and zero
instances occur of the refusal resolving into silence.

**Setup**: playback not yet started; focus-injector APK.

**Procedure — Part A (deferred grant), per trial (run 10×)**:
1. Arm the injector as a transient focus holder; start playback in the app —
   the app's `setAcceptsDelayedFocusGain(true)` request is accepted but
   deferred.
2. While deferred, confirm zero audio and mirror transport not `playing`.
3. Have the injector abandon focus; the grant arrives.
4. Confirm audio begins from the position the session would otherwise have
   played from, with no rewind applied and no pause reason shown.

**Procedure — Part B (refusal), per trial (run 10×)**:
1. Arm the injector to refuse the focus request outright.
2. Press Play.
3. Confirm the refusal surfaces as a named failure with a stated reason on the
   lock-screen surface (scrolling statement) and, on next open, in the
   application; confirm the session presents no playing state (mirror
   transport not `playing`) and zero audio.

**Observe**: audio presence during deferral; resume point vs the would-be
position; refusal presentation on both surfaces; transport state.
**Per-trial pass**: Part A — no audio until the grant, correct resume point,
no rewind, no pause reason; Part B — named failure + stated reason on both
surfaces, no playing state, zero instances of silence.
**10/10 rule**: all ten trials of each part must pass; any failure fails
SC-055.

## T-06 · SC-004 — Every lock-screen transport control works without unlock

**Requirement** (SC-004): In 10 out of 10 trials, every transport control
reachable from the lock screen produces the correct effect without requiring an
unlock.

**Setup**: playback running; device locked; corpus doc (a).

**Procedure — per trial (run 10×)**:
1. Lock the device. From the lock screen / notification shade, exercise in
   order: play, pause, resume, skip-forward, skip-backward, Stop, rate-up,
   rate-down, next-voice (FR-018, FR-018b, FR-019 — the full set reachable
   while driving).
2. After each press, confirm the effect without unlocking: mirror transport /
   position / rate from `adb shell dumpsys media_session`, plus audible
   confirmation (speech starts / stops / jumps ±5 s / changes speed / voice
   changes).
3. Confirm no control press required an unlock.

**Observe**: per-control effect vs expectation; unlock count (must be zero).
**Per-trial pass**: every control produced its correct effect (pause pauses,
Stop stops and discards position, skip moves exactly ±5 s, rate moves exactly
one rung, voice moves one step) with zero unlocks.
**10/10 rule**: all ten trials must pass; any failure fails SC-004.

## T-07 · SC-007 — Resume after unexpected process termination, replay ≤ 30 s

**Requirement** (SC-007 · FR-004): After an unexpected process termination and
relaunch, playback can be resumed at the last spoken position, replaying at
most 30 seconds of already-spoken audio, and in 10 out of 10 trials never
replaying more than that bound.

**Setup**: playback running mid-document; note the mirror position P at a
mid-sentence point.

**Procedure — per trial (run 10×)**:
1. Play to a distinct mid-document position P (record P from the mirror).
2. Terminate the process unexpectedly: `adb shell am kill <pkg>` (process
   death without a final write path).
3. Relaunch the application and open the same document.
4. Confirm a resume offer at the last spoken position; activate resume.
5. Measure the resume position R and compute the replay `P − R`.

**Observe**: P, R, replay duration; presence of the resume offer.
**Per-trial pass**: resume offered at the last spoken position and
**`P − R ≤ 30 s`** in every trial (replay beyond 30 s is a defect; discarding
the position entirely is a defect).
**10/10 rule**: all ten trials must pass; any failure fails SC-007.

## T-08 · SC-008 — Blind-test driving tasks: no unlock, no visual confirmation

**Requirement** (SC-008 · FR-018a): In 10 out of 10 locked trials with the
screen covered (blind-test harness: the tester cannot see the surface and a
trial fails on any unlock or any effect left unconfirmed aloud), every
driving-time task — play, pause, resume, skip in either direction, rate change
— completes without an unlock and zero trials require visual confirmation of
the effect.

**Setup**: opaque screen cover fitted; device locked; playback paused mid-document.

**Procedure — per trial (run 10×)**:
1. Cover the screen so neither the tester nor any camera can see the surface.
2. Without uncovering, perform in order: play, pause, resume, skip-forward,
   skip-backward, rate-up, rate-down.
3. Confirm each effect **audibly** (speech begins / stops / resumes / jumps
   / changes speed) — an effect that cannot be confirmed aloud fails the trial.
4. Any unlock attempt, or any task requiring a glance, fails the trial.

**Observe**: task completion per press; audible confirmation per effect; unlock
count.
**Per-trial pass**: all seven tasks completed locked, each confirmed aloud,
zero unlocks, zero visual confirmations.
**10/10 rule**: all ten trials must pass; any failure fails SC-008.

## T-09 · SC-009 — Audible audio within 3 seconds of the play action

**Requirement** (SC-009 · FR-027): In 10 out of 10 trials, audible audio
begins within 3 seconds of the play action on a mid-range arm64 device with
the document already parsed.

**Setup**: **normative reference device** (README §2); document already
parsed; playback not started (fresh session).

**Procedure — per trial (run 10×)**:
1. Ensure the document is already parsed (open it, wait for the parsed state,
   pause before pressing Play).
2. Press Play; the harness records the press timestamp
   (`audio.first_sample_ts_ms` minus press timestamp, or operator-timed with
   the trace line as the precise channel).
3. Measure the interval to the first audible sample.

**Observe**: press-to-first-audio interval per trial.
**Per-trial pass**: interval **≤ 3 s**. A longer silence is a failure to
start, not normal startup.
**10/10 rule**: all ten trials must pass; any failure fails SC-009. Measured
on the reference device; every verdict counts as-is, no device-class asterisk.

## T-10 · SC-011 — Service killed under memory pressure: paused, no auto-audio

**Requirement** (SC-011): In 10 out of 10 trials where the service is killed
under memory pressure, the next launch shows a paused state at the last spoken
position and produces no audio until the user activates resume.

**Setup**: playback running mid-document; note mirror position P.

**Procedure — per trial (run 10×)**:
1. Play to mid-document; record P.
2. Simulate memory pressure: `adb shell am send-trim-memory <pkg>
   RUNNING_CRITICAL`, then `adb shell am kill <pkg>`.
3. Relaunch the application and open the document.
4. Observe the presented state; do not press resume for 15 s.
5. Confirm the paused state is positioned at the last spoken position, an
   explicit resume control is present, and zero audio played on launch.
6. Activate resume; confirm playback continues from the last spoken position
   (replay within the SC-007 bound).

**Observe**: presented state and position vs P; audio presence before the
resume press.
**Per-trial pass**: paused state at the last spoken position with an explicit
resume control; **zero audio until the user activates resume**.
**10/10 rule**: all ten trials must pass; any failure fails SC-011.

## T-11 · SC-013 — Pre-roll never advances position, never exceeds 60 s

**Requirement** (SC-013 · FR-015b): Zero instances of pre-roll buffering
advancing the playback position, and zero instances of the pre-roll buffer
holding more than 60 seconds of audio or growing while a session remains
paused indefinitely, and zero instances of synthesis being performed while
paused beyond that bound.

**Setup**: playback paused mid-document (after a Play press prepared pre-roll);
tracing channel live.

**Procedure — per trial (run 10×)**:
1. Press Play, let pre-roll prepare, then pause; record mirror position P.
2. Hold the session paused for 10 minutes, sampling `preroll.buffered_secs`
   and the mirror `position_secs` at 1-minute intervals.
3. Assert at every sample: `position_secs` unchanged (== P),
   `preroll.buffered_secs ≤ 60`, and the buffer not growing; per FR-015c the
   buffer drains to empty once the session is idle (zero synthesis while
   idle).
4. After the hold, press Play and confirm pre-roll begins again and audio
   starts (refill path, FR-015a).

**Observe**: position sequence; pre-roll level sequence; synthesis activity
(zero while paused-idle).
**Per-trial pass**: zero position advance; **≤ 60 s** pre-roll at every
sample; non-growing; drains to empty while paused-idle; zero synthesis beyond
the bound; refill on the next Play press.
**10/10 rule**: all ten trials must pass; any failure fails SC-013.

## T-12 · SC-014 — Play from the lock screen with the app process not running

**Requirement** (SC-014): In 10 out of 10 trials, a Play press from the lock
screen with the application process not running produces audible audio without
the user opening the application, and where the platform refuses the start,
the refusal is stated and no playing state is shown.

**Setup**: a session has ever been started (resident component alive per
FR-009a); app UI closed (swiped from recents); device locked.

**Procedure — per trial (run 10×)**:
1. Start playback, pause (service stays resident), close the app UI (swipe
   from recents), lock the device.
2. From the lock screen, press Play on the resident notification.
3. Confirm audible audio begins without opening the application.
4. **Refusal half** (every trial, on an API-34+ device or with the resident
   component killed first via `adb shell am force-stop <pkg>`): press Play
   from the lock screen where no cold foreground-service start is permitted;
   confirm the refusal is stated (reason surfaced per FR-010) and no playing
   state is shown; confirm audio begins only after the user opens the
   application.

**Observe**: audio from the locked Play press; refusal presentation; playing
state during refusal.
**Per-trial pass**: locked Play press produces audible audio with the app not
open; the refused start states the reason and shows no playing state.
**10/10 rule**: all ten trials must pass; any failure fails SC-014.

## T-13 · SC-016 — Skip moves exactly 5 seconds; boundaries clamp

**Requirement** (SC-016 · FR-018): In 10 out of 10 trials, a skip press from
the lock screen moves the playback position by exactly five seconds in the
pressed direction, and a press that would cross a document boundary clamps to
that boundary without error or silence.

**Setup**: playback running; device locked; mirror position readable.

**Procedure — per trial (run 10×)**:
1. From the lock screen, at a known position P (mid-document, ≥ 10 s from
   both ends), press skip-forward; read the new position from the mirror.
2. Press skip-backward; read the position.
3. Assert forward displacement is exactly **+5 s** and backward exactly
   **−5 s** against the position at the moment of the press, independent of
   headings or paragraphing.
4. Near the document end, press skip-forward until the boundary; assert the
   position clamps to the document end and settles into a paused
   end-of-document state — no wrap, no skip into silence, no error.
5. Near the start, press skip-backward; assert clamping to position 0.

**Observe**: displacement per press; clamp behaviour at both boundaries.
**Per-trial pass**: exact ±5 s displacement; clean boundary clamps with no
error and no silence.
**10/10 rule**: all ten trials must pass; any failure fails SC-016.

## T-14 · SC-017 — Stored position survives close, kill, power-off, network loss

**Requirement** (SC-017 · FR-032a): In 10 out of 10 trials, the stored
position for a paused document survives application close, service kill, device
power-off, and network loss, and reopening the document offers to continue
from that position.

**Setup**: playback at a known mid-document position P; the four ending modes
rotate across trials (trials 1–3 application close, 4–6 service kill, 7–8
power-off, 9–10 network loss — each mode exercised at least twice).

**Procedure — per trial (run 10×)**:
1. Play to position P; record P.
2. End the session by the trial's assigned mode:
   - *App close*: close the application (swipe from recents).
   - *Service kill*: `adb shell am kill <pkg>`.
   - *Power-off*: long-press power → Power off (battery-death path is the
     same stored-state path; power-off while playing is the observable form).
   - *Network loss*: `adb shell svc wifi disable` + `adb shell svc data
     disable` mid-playback.
3. Reopen the document.
4. Confirm a resume offer at the last spoken position (within the ≤ 30 s
   replay bound of SC-007 where the ending left no final write).

**Observe**: offer presence; offered position vs P.
**Per-trial pass**: the stored position survived the assigned ending and the
reopen offers to continue from it.
**10/10 rule**: all ten trials must pass; any failure fails SC-017.

## T-15 · SC-018 — Stop discards the stored position

**Requirement** (SC-018 · FR-032): In 10 out of 10 trials, Stop discards the
stored position such that reopening the document begins from its first spoken
position, with no resume offer presented.

**Setup**: playback paused mid-document with a stored position; device unlocked.

**Procedure — per trial (run 10×)**:
1. Play to a mid-document position; pause (position stored).
2. Press Stop (transport Stop, not the resident-stop control).
3. Reopen the same document.
4. Confirm playback begins from the first spoken position and **no resume
   offer** is presented anywhere.

**Observe**: restart position; absence of any resume offer.
**Per-trial pass**: first-spoken-position start; zero resume offers.
**10/10 rule**: all ten trials must pass; any failure fails SC-018.

## T-16 · SC-020 — Deliberate-pause reopen: 3 s offer, auto-start ≤ 4 s

**Requirement** (SC-020 · FR-032c, FR-032d): In 10 out of 10 trials,
reopening a deliberately paused document shows the resume offer with both
continue and start-over available, and reading begins within 4 seconds of the
offer appearing (3 s offer window per FR-032c + 1 s post-expiry start budget,
stricter than the FR-027 3 s play-to-audio bound which still governs the
post-expiry start itself) whether or not the user acts, provided the
application is in the foreground when the window expires; where it is not, no
audio begins until the user activates resume.

**Setup**: document deliberately paused mid-document; harness timestamps the
offer appearance.

**Procedure — per trial (run 10×)**:
1. **Foreground half**: pause the document deliberately, close the app, reopen
   the document with the application in the foreground. Confirm the resume
   offer shows both *continue* and *start over*. Take no action; time from
   offer appearance to the start of audio.
2. Assert audio began **≤ 4 s** after the offer appeared (3 s window + 1 s
   post-expiry budget).
3. **Background half** (same trial): repeat with the application backgrounded
   at expiry; confirm no audio begins until the user activates resume, and the
   offer converts to the explicit-resume form (FR-032d).
4. In a separate pass of the same trial set, choose *start over* within the
   window and confirm the document begins from its first spoken position and
   the stored position is discarded.

**Observe**: offer contents; time-to-audio from offer appearance; behaviour
on foreground vs background expiry; start-over path.
**Per-trial pass**: both options present; auto-start ≤ 4 s when foregrounded
at expiry; zero audio when backgrounded until resume is activated; start-over
discards the position.
**10/10 rule**: all ten trials must pass; any failure fails SC-020.

## T-17 · SC-021 — Unexpected-end reopen: zero audio until resume

**Requirement** (SC-021 · FR-032d): In 10 out of 10 trials, reopening a
document left by an unexpected end, a power-off, a battery death, or a
network loss produces no audio at all until the user activates resume.

**Setup**: document mid-playback; the four ending modes rotate across trials
(trials 1–3 unexpected end / service kill, 4–5 power-off, 6–7 battery death —
simulated by power-off while playing, the same stored-state path — 8–10 network
loss).

**Procedure — per trial (run 10×)**:
1. Play to a mid-document position; end the session by the assigned mode.
2. Reopen the document.
3. Observe for 20 s without touching the app.
4. Confirm **zero audio** and a paused state with an explicit resume
   control (never auto-start, never an offer that auto-plays).
5. Activate resume; confirm playback continues from the last spoken position.

**Observe**: audio presence on reopen; presented state; resume behaviour.
**Per-trial pass**: zero audio until the user activates resume, in every
trial.
**10/10 rule**: all ten trials must pass; any failure fails SC-021.

## T-18 · SC-022 — Move/rename keeps the position; changed content does not

**Requirement** (SC-022 · FR-036): In 10 out of 10 trials, a document that
is moved or renamed on the device is still recognised as the same document and
offers to resume at its previously stored position, and in 10 out of 10 trials
a document with different content at the same path offers no resume.

**Setup**: corpus doc (a) paused mid-document with a stored position; copies
for the move/rename corpus.

**Procedure — Part A, per trial (run 10×)**:
1. Pause doc (a) at position P; record P.
2. Move or rename the file on the device (new path and/or new name).
3. Open the document at its new location.
4. Confirm the document is recognised as the same document (content
   fingerprint, not location) and the resume offer shows position P.

**Procedure — Part B, per trial (run 10×)**:
1. Pause doc (a) at position P.
2. Replace the file at the same path with different content (corpus doc (f)).
3. Open the document.
4. Confirm **no resume offer** and playback begins from the first spoken
   position.

**Observe**: offer presence and position in both parts.
**Per-trial pass**: Part A — same-document recognition with the stored
position offered; Part B — no offer, first-position start.
**10/10 rule**: all ten trials of each part must pass; any failure fails
SC-022.

## T-19 · SC-023 — Locked refusal / device failure states its reason, no doc text

**Requirement** (SC-023): In 10 out of 10 trials, a refusal to start or an
audio-device failure occurring while the device is locked presents a reason on
the lock-screen surface without an unlock, and in 10 out of 10 trials that
reason contains no text drawn from the document.

**Setup**: device locked; fault-injection fixture armed.

**Procedure — per trial (run 10×)**:
1. Lock the device; arm `fault=device_refused` (audio-device failure on the
   next stream open; alternate trials use a refused start via the locked Play
   path of SC-014).
2. Trigger playback (Play press from the lock screen or from the app before
   locking).
3. Without unlocking, read the lock-screen / notification-shade surface: a
   short reason statement must be present (scrolling form per FR-041/FR-037).
4. Compare the statement against the test document's text: it must share no
   sentence, phrase, or fragment with the document (failure reasons are
   app-generated text and never quote the document, FR-037).

**Observe**: reason presence on the lock screen; unlock count (zero); statement
vs document corpus.
**Per-trial pass**: reason presented locked without an unlock; zero document
text in the reason.
**10/10 rule**: all ten trials must pass; any failure fails SC-023.

## T-20 · SC-024 — Voice/engine change re-anchors to a boundary, boundary stated

**Requirement** (SC-024 · FR-038): In 10 out of 10 trials, resuming a
document after a voice or engine change begins at a sentence or node boundary
rather than mid-word, and the boundary chosen is reported to the reader.

**Setup**: playback paused mid-document on `standin_a` at position P; both
stand-ins registered.

**Procedure — per trial (run 10×)**:
1. Pause under `standin_a` at a mid-sentence position P; record P.
2. Change voice/engine to `standin_b` (different narration timeline).
3. Resume; read the resume position R from the mirror and the app's stated
   boundary (the application states which boundary it resolved to, FR-038).
4. Assert R is the start of the nearest sentence or node boundary at or before
   P in `standin_b`'s timeline — never a raw mid-word offset — and that the
   app names the boundary it resolved to.

**Observe**: R vs P; boundary statement in the app; landing on the stand-in's
known sentence map.
**Per-trial pass**: resume at a sentence/node boundary (zero mid-word
resumes); boundary reported to the reader.
**10/10 rule**: all ten trials must pass; any failure fails SC-024.

## T-21 · SC-025 — Record keeps the failure, stays on-device, trims at 10 MB

**Requirement** (SC-025 · FR-039, FR-039b): In 10 out of 10 trials, a
forced audio-device failure leaves a local record naming the failure and the
position at which it occurred, zero instances of any diagnostic record leaving
the device, and zero instances of entries older than 30 days remaining in the
record, and zero instances of the record being readable by another application
through a shared or device-wide log area; and in 10 out of 10 trials in which
the record is driven past 10 MB, the oldest entries are trimmed so the record
does not exceed the cap and the trim remains visible as a trim rather than as
an absence of events.

**Setup**: record readable via `adb shell run-as <pkg> cat files/diag.jsonl`;
fault-injection fixture; harness press-loop for the size test.

**Procedure — per trial (run 10×)**:
1. Arm `fault=device_refused`; trigger playback; let the failure surface.
2. Read the record: a `failure` entry with `error: device_refused` and the
   `position_secs` of the failure must exist.
3. Confirm the record exists only under `/data/data/<pkg>/files/` — not in
   `/sdcard`, not in any shared or device-wide log area — and the second-app
   fixture's read attempt is denied (zero readability by another application).
4. Confirm zero egress for the app uid during the trial (network stats show
   zero bytes transmitted; the record never leaves the device, FR-040).
5. **Aged-entry sub-check** (trials 1–5): pre-seed the record (debug
   tooling) with entries older than 30 days plus fresh entries; run the app;
   assert the aged entries are discarded oldest-first and a `trim` marker with
   `dropped_events ≥ 1` and `cutoff_ts_ms` is present.
6. **10 MB sub-check** (trials 6–10): drive the app through repeated
   transport cycles (harness press loop; the CLI verb calls the same code per
   Principle II) until the record exceeds 10 MB; assert the record size stays
   **≤ 10 MB**, the oldest entries are gone, and `trim` markers remain visible
   as trims (no silent gaps).

**Observe**: failure entry content and position; record location; second-app
read result; egress bytes; aged-entry disposal; size after the drive.
**Per-trial pass**: failure recorded with position; record on-device only,
unreadable by other apps, never transmitted; aged entries trimmed with visible
markers; size ≤ **10 MB** with visible trims.
**10/10 rule**: all ten trials must pass; any failure fails SC-025.

## T-22 · SC-026 — Rate ladder: exactly one step, saturating ends

**Requirement** (SC-026 · FR-019): In 10 out of 10 trials, each rate control
press moves the rate exactly one step along the declared ladder of 0.75x, 1.0x,
1.25x, 1.5x, 1.75x, 2.0x, 4.0x, a press at either end of the ladder holds
that end, and no press ever yields a rate outside the ladder.

**Setup**: playback running; rate controls reachable (lock screen and in-app).

**Procedure — per trial (run 10×)**:
1. From 1.0x, press rate-up six times; read the rate after each press from
   the mirror / app state: must traverse 1.25 → 1.5 → 1.75 → 2.0 → 4.0.
2. Press rate-up again at 4.0x: the rate must **hold at 4.0x** (saturate,
   no wrap, no failure, no out-of-ladder value).
3. Press rate-down eight times: must traverse back down and **hold at
   0.75x** at the bottom end.
4. Assert every observed rate is one of the seven rungs and every
   displacement is exactly one rung.

**Observe**: rate after each press; end behaviour.
**Per-trial pass**: one-rung steps throughout; both ends saturate; zero
out-of-ladder values.
**10/10 rule**: all ten trials must pass; any failure fails SC-026.

## T-23 · SC-027 — Lock-screen rate equals in-app rate; rate never on the lock screen

**Requirement** (SC-027): In 10 out of 10 trials, a rate reached from the
lock screen is the same rate shown inside the application, and the rate value
never appears on the lock screen or in the notification shade.

**Setup**: playback running; device locked.

**Procedure — per trial (run 10×)**:
1. From the lock screen, press rate-up/down until the rate changes (e.g. to
   1.5x); read the rate from the mirror.
2. Open the application and read the in-app rate: must equal the lock-screen
   change.
3. Inspect the lock screen and notification shade content (visual +
   `dumpsys notification` text): the rate value (e.g. "1.5x", "1.5×") must
   appear nowhere — the rate is shown inside the application only, never on
   the FR-028 surface.

**Observe**: lock-screen rate vs in-app rate; presence of any rate string on
the lock screen or shade.
**Per-trial pass**: identical rates; zero rate-value appearances on the
lock-screen surface.
**10/10 rule**: all ten trials must pass; any failure fails SC-027.

## T-24 · SC-028 — Notification permission denied: stated, playback still works

**Requirement** (SC-028 · FR-008, FR-021): In 10 out of 10 trials where the
notification permission is denied, the application states that lock-screen
controls are unavailable and playback can still be started, paused, and stopped
from inside the application.

**Setup**: API 33+ device; permission revocable.

**Procedure — per trial (run 10×)**:
1. Revoke notification permission: `adb shell pm revoke <pkg>
   android.permission.POST_NOTIFICATION` (or appops deny).
2. Open the application; confirm it **states** that lock-screen controls are
   unavailable (no silent omission, FR-021).
3. From inside the application, start, pause, and stop playback; confirm all
   three work.
4. Confirm no lock-screen media surface appears while the permission is
   denied.
5. Re-grant the permission at the end of the trial set.

**Observe**: the stated consequence; in-app transport function; absence of the
lock-screen surface.
**Per-trial pass**: unavailability stated; start/pause/stop all functional
in-app.
**10/10 rule**: all ten trials must pass; any failure fails SC-028.

## T-25 · SC-029 — Permanent loss with screen off: scrolling paused statement

**Requirement** (SC-029): In 10 out of 10 trials, a permanent focus loss
reached while the screen is off results in a paused state that is visibly
reported on the lock screen or in the notification shade, so the reader learns
the document stopped without unlocking the device, and the reported statement
scrolls rather than sitting static, names the document title, states that
reading is paused rather than resuming, and carries no other document text.

**Setup**: playback running; screen off; focus-injector APK.

**Procedure — per trial (run 10×)**:
1. Start playback; turn the screen off.
2. Injector requests permanent focus (`AUDIOFOCUS_GAIN`).
3. Wake to the lock screen **without unlocking**; observe the notification
   surface.
4. Confirm: paused state visibly reported; the statement **scrolls** (moving
   text, not static); it names the document title; it states reading is paused
   (never that it is resuming); it carries no other document text.

**Observe**: surface content and scroll behaviour; statement wording vs the
document corpus.
**Per-trial pass**: visible paused report unlocked-free; scrolling statement;
title named; "paused" not "resuming"; zero other document text.
**10/10 rule**: all ten trials must pass; any failure fails SC-029.

## T-26 · SC-030 — Title/section-title edge cases: placeholder, absence, fallback

**Requirement** (SC-030 · FR-028c): In 10 out of 10 trials, a document with
no usable title shows a stable placeholder on the notification surface, a
document with no section title at the current position shows no section title
rather than body text, and a position before the document's first heading
shows no section title while a position after a heading shows that heading.

**Setup**: corpus docs (b) (no metadata, no headings) and (c) (content before
first heading); doc (a) (titled, ≥ 2 headings).

**Procedure — per trial (run 10×)**:
1. Play doc (b) (no usable title): confirm the notification shows a **stable
   placeholder** (file name per FR-029) — never the first spoken sentence.
2. Play doc (b) and doc (c) before the first heading: confirm **no section
   title** is shown (not body text, not a forward-looking heading).
3. Play doc (a) to a position after its first heading: confirm the most
   recent preceding heading is shown; cross a heading boundary and confirm the
   section title advances.

**Observe**: placeholder stability; section-title presence/absence per
position.
**Per-trial pass**: stable placeholder for the untitled doc; no section title
before the first heading / headingless doc; correct preceding heading after a
heading.
**10/10 rule**: all ten trials must pass; any failure fails SC-030.

## T-27 · SC-031 — Focus-loss pause reason: shown ≤ 3 s, expired after 3 s

**Requirement** (SC-031 · FR-037a, FR-041): In 10 out of 10 trials, opening
the application within 3 seconds of that pause shows the reason, and in 10 out
of 10 trials opening it later than 3 seconds after the pause shows no such
reason while still showing the paused state and its resume control, and zero
instances occur of a focus-loss pause reason being shown in the application
after that 3-second window in any of those trials.

**Setup**: playback running; focus-injector APK; timer.

**Procedure — per trial (run 10×)**:
1. Trigger a focus-loss pause (permanent loss or silence-demanding
   interruption).
2. **Within-3 s half**: open the application at ~1 s after the pause; confirm
   the pause reason is shown.
3. **After-3 s half** (same trial): trigger another focus-loss pause; wait
   > 3 s (use 10 s); open the application; confirm **no** focus-loss reason
   is shown while the paused state and its resume control still are.
4. Assert zero instances of the focus-loss reason appearing after the window.

**Observe**: reason presence at ~1 s vs at > 3 s; paused state and resume
control presence.
**Per-trial pass**: reason shown within 3 s; reason absent after 3 s with
paused state + resume control intact; zero post-window reason instances.
**10/10 rule**: all ten trials must pass; any failure fails SC-031.

## T-28 · SC-032 (API-29 floor) — FGS mediaPlayback starts and stays alive

**Requirement** (SC-032): The application declares a minimum supported Android
version of 10 (API 29) — declared-manifest assertion discharged by T081's
`check-manifest.sh` (`minSdk(Version) 29`) — and in 10 out of 10 trials on a
device running that version or later, a foreground service with the
media-playback type starts and keeps playback alive with the screen off.

**Setup**: device running API 29 or later (record the API level); playback
started from a cold start.

**Procedure — per trial (run 10×)**:
1. Record `adb shell getprop ro.build.version.sdk` (must be ≥ 29).
2. Cold-start playback; confirm the service runs in the foreground with the
   media-playback type (`adb shell dumpsys activity services <pkg>` shows the
   foreground service; the ongoing media notification is present).
3. Turn the screen off; wait 2 minutes.
4. Confirm playback is still alive (audible / mirror `playing` / service still
   foreground).

**Observe**: API level; service foreground state with media-playback type;
liveness after 2 minutes screen-off.
**Per-trial pass**: service with the media-playback type starts and keeps
playback alive with the screen off, on an API-29+ device, in every trial.
**10/10 rule**: all ten trials must pass; any failure fails SC-032.

## T-29 · SC-033 — New output device: follow automatically, position unchanged

**Requirement** (SC-033 · FR-017a): In 10 out of 10 trials, a new output
device becoming available during playback causes audio to continue on that
device from the unchanged position, with zero instances of a pause, a restart,
a repeated phrase, or audio moving to a loudspeaker as a result.

**Setup**: playback running through the built-in speaker or a first headset;
a second private output device (wired headset or Bluetooth headset) available
to connect.

**Procedure — per trial (run 10×)**:
1. Start playback; record mirror position P.
2. Mid-sentence, connect the new private output device (plug the wired headset
   / connect the Bluetooth headset).
3. Confirm audio continues on the new device from the unchanged position P:
   no pause, no restart, no repeated phrase (position never moved backward).
4. Confirm audio never moved to the loudspeaker as a result of the switch.
5. Confirm the switch was recorded as an `output_device` event in the
   diagnostic record (FR-017a record requirement).

**Observe**: audio route after the switch; position continuity; loudspeaker
absence; record entry.
**Per-trial pass**: automatic follow to the new device from the unchanged
position; zero pause/restart/repeat; zero loudspeaker movement; event recorded.
**10/10 rule**: all ten trials must pass; any failure fails SC-033.

## T-30 · SC-033b — Coincident disappearance + appearance: privacy wins first

**Requirement** (SC-033b · FR-016, FR-017a): In 10 out of 10 trials where a
disappearance and an appearance coincide in one route change (unplugging wired
while car Bluetooth connects), the system pauses first under FR-016 and resumes
only under the FR-017a privacy-precedence rule, with zero instances of audio
moving unattended to the newly appearing device.

**Setup**: playback running through wired headphones; car Bluetooth available to
connect.

**Procedure — per trial (run 10×)**:
1. Start playback through wired headphones.
2. In one route change: unplug the wired headphones while car Bluetooth
   connects (the disappearance and appearance coincide).
3. Confirm the **disappearance takes precedence**: playback pauses first (≤ 1 s,
   per SC-002's bound).
4. Confirm audio does **not** move unattended to the newly appearing car
   Bluetooth (a public/shared-cabin output): no audio on it until the user
   explicitly resumes.
5. Explicitly resume; confirm playback continues on the car Bluetooth from the
   paused position (explicit-user-action path only).

**Observe**: pause-first ordering; audio destination during the coincident
change; resume behaviour.
**Per-trial pass**: pause first; zero unattended movement to the newly
appearing device; resume only by explicit user action.
**10/10 rule**: all ten trials must pass; any failure fails SC-033b.

## T-31 · SC-034 — Delivered audio is always 22050 Hz, never a device default

**Requirement** (SC-034 · FR-033): Zero instances of the audio the engine
delivers arriving at a sample rate other than the single declared rate of
22050 Hz, including during and after every rate-ladder step of FR-019, so
that a rate change is observable only as a change in speed; and zero instances
occur of the output stream being opened at a device default rate in place of
the declared one.

**Setup**: playback running; tracing channel live; all seven rungs exercised.

**Procedure — per trial (run 10×)**:
1. Start playback; read the `audio.stream` trace line on stream open:
   requested and delivered rate must be **22050 Hz**, mono, f32 — never
   `UNSPECIFIED`, never a device default.
2. Step the rate through all seven rungs (0.75 → 4.0 and back), reading the
   `audio.stream` trace line at each step: the delivered rate must remain
   22050 Hz at every rung (a rate change is observable only as speed).
3. Confirm zero `rate_mismatch` / `device_refused` failures unless the device
   genuinely refuses (a refusal is a named failure under FR-035 — never a
   substituted format, never a silent resample).
4. Where the platform reports track format (`dumpsys media.audio_flinger`),
   corroborate the track runs at 22050 Hz.

**Observe**: delivered rate at open and at every rung; failure kinds.
**Per-trial pass**: 22050 Hz delivered at open and after every ladder step;
zero device-default opens; mismatches only ever surface as named failures.
**10/10 rule**: all ten trials must pass; any failure fails SC-034.

## T-32 · SC-036 — New session opens at the last rate used, app-wide

**Requirement** (SC-036 · FR-019b): In 10 out of 10 trials, a new session
opens at the last rate used anywhere in the application, at 1.0x when none has
been chosen, the chosen rate survives the application being closed, and zero
instances occur of a document opening at a rate different from the one in
effect before it was opened.

**Setup**: two different documents; rate set via the session CLI
(`session rate set <v>`) or the app.

**Procedure — per trial (run 10×)**:
1. On doc (a), set the rate to a non-default rung (e.g. 1.5x); close the
   application.
2. Open doc (b) (a different document): confirm it opens at **1.5x** — the
   rate is one app-wide setting, not per-document.
3. Close and reopen the application: confirm 1.5x still in effect.
4. **First-use check** (fresh install / cleared settings): confirm a new
   session opens at **1.0x**.
5. Assert zero documents opened at a rate different from the one in effect
   before they were opened.

**Observe**: opening rate per document/session; persistence across close.
**Per-trial pass**: last-used rate carried across sessions and documents; 1.0x
on first ever use; zero divergent openings.
**10/10 rule**: all ten trials must pass; any failure fails SC-036.

## T-33 · SC-038 — Edited content is a different document: no resume, no match

**Requirement** (SC-038 · FR-036a): In 10 out of 10 trials, a document
edited while no session is active offers no resume, begins from its first
spoken position, and zero instances occur of a stored position being offered
or approximately matched against content it was not recorded against.

**Setup**: doc (a) paused mid-document with a stored position; no session
active.

**Procedure — per trial (run 10×)**:
1. Pause doc (a) at position P (position stored against its fingerprint).
2. With no session active, edit the document's content on disk (corpus doc
   (f) — same path, different bytes).
3. Open the document.
4. Confirm **no resume offer**, playback begins from the first spoken
   position, and no approximately matched position is offered anywhere.

**Observe**: offer presence; start position; any partial-match behaviour.
**Per-trial pass**: no offer; first-position start; zero approximate matches.
**10/10 rule**: all ten trials must pass; any failure fails SC-038.

## T-34 · SC-039 — Mid-document engine failure: named, offered, continues in place

**Requirement** (SC-039 · FR-023a): In 10 out of 10 trials, an engine
failing mid-document presents the named failure with the available
alternatives, and playback continues from the last spoken position rather than
the document start, whether the reader chooses within 3 seconds or lets the
window expire, and zero trials leave an unresolved failure presenting as
silence.

**Setup**: playback running on `standin_a`; `standin_b` registered;
fault-injection fixture.

**Procedure — per trial (run 10×)**:
1. Play to a mid-document position P on `standin_a`; record P.
2. Inject a mid-document failure into `standin_a`.
3. Confirm the **named failure** is presented with the available alternative
   engines, in a 3-second offer.
4. **Choice half** (odd trials): choose the alternative within 3 s.
   **Expiry half** (even trials): let the window expire.
5. In both halves: confirm playback continues from the last spoken position P
   (not the document start) on the other engine, and the failure and the
   engine change are recorded in the diagnostic record.
6. Assert zero silence: the failure never resolves into unexplained silence.

**Observe**: failure presentation; offer contents; resume position vs P;
record entries.
**Per-trial pass**: named failure + alternatives offered; continuation from
the last spoken position on both the choice and expiry paths; zero silence.
**10/10 rule**: all ten trials must pass; any failure fails SC-039.

## T-35 · SC-040 — Locked engine failure: 3-second offer resolves without unlock

**Requirement** (SC-040): In 10 out of 10 trials, an engine failing while the
device is locked resolves through the 3-second offer without an unlock, and the
notification has stated both the failure and the fallback; and in 10 out of 10
trials where no alternative engine exists, the session ends paused with a stated
reason rather than continuing or failing silently.

**Setup**: playback running; device locked; fault-injection fixture.

**Procedure — Part A, per trial (run 10×)**:
1. Play on `standin_a`; lock the device.
2. Inject a mid-document failure into `standin_a`.
3. Without unlocking, confirm the notification states both the failure and the
   fallback, and the 3-second offer resolves (auto-fallback to `standin_b`
   after expiry) with playback continuing from the last spoken position.

**Procedure — Part B (no alternative), per trial (run 10×)**:
1. Play on `standin_a` with `standin_b` armed to fail as well (forced
   double failure).
2. Inject the failure; let the 3-second window expire with no working
   alternative.
3. Confirm the session ends **paused with a stated reason** — never
   continuing, never failing silently.

**Observe**: notification content (failure + fallback); unlock count (zero);
end state in Part B.
**Per-trial pass**: Part A — resolved locked, both facts stated; Part B —
paused with a stated reason, zero silence.
**10/10 rule**: all ten trials of each part must pass; any failure fails
SC-040.

## T-36 · SC-042 — Progress determinate only when the total is actually known

**Requirement** (SC-042 · FR-028a): In 10 out of 10 trials, the progress
indication on the lock-screen surface is determinate only when the document's
total spoken length is actually known and is indeterminate otherwise, zero
instances occur of an estimated or projected total being shown as established,
and zero instances occur of the indication switching between determinate and
indeterminate within one session.

**Setup**: corpus doc (d) (pre-computed total spoken length) and a
synthesised-as-it-plays document (total not knowable in advance).

**Procedure — per trial (run 10×)**:
1. Play the unknown-total document: confirm the lock-screen progress
   indication is **indeterminate** for the whole session and never adopts a
   projected total.
2. Play doc (d) (total actually known): confirm the indication is
   **determinate** from the start (bar/percentage against the known total).
3. In both cases, observe the whole session: the indication must never switch
   determinacy mid-session (a bar whose denominator changes while the reader
   is looking at it is a defect).

**Observe**: determinacy form per document; any mid-session switch; any
estimated total.
**Per-trial pass**: indeterminate when the total is unknown; determinate only
when actually known; zero estimated totals; zero mid-session switches.
**10/10 rule**: all ten trials must pass; any failure fails SC-042.

## T-37 · SC-043 — Residency after pause: one component, one notification, live Play

**Requirement** (SC-043 · FR-009a): In 10 out of 10 trials, after playback
has been paused or interrupted and a resumable position remains, exactly one
resident component and one ongoing notification exist and no second lock-screen
surface competes with them, the resident notification shows the paused state
and a resume control while asserting no elapsed time, remaining time, or
section title, and a Play press from that notification still produces audio
without the application being opened.

**Setup**: playback paused mid-document (resident state active).

**Procedure — per trial (run 10×)**:
1. Pause playback mid-document (also exercise an interrupted session in half
   the trials: kill the app UI / simulate interruption, leaving a resumable
   position).
2. Count resident components (`adb shell dumpsys activity services <pkg>`)
   and ongoing notifications (`adb shell dumpsys notification`): exactly
   **one** of each, no second lock-screen surface.
3. Inspect the resident notification: paused state + resume control present;
   **no elapsed time, no remaining time, no section title** on the
   resident-paused surface (FR-009a).
4. With the app not open, press Play from the resident notification: confirm
   audible audio (the liveness verdict — SC-049 carries the count/teardown
   verdict).

**Observe**: component and notification counts; resident-surface content;
audio from the locked Play press.
**Per-trial pass**: exactly one resident component + one ongoing notification;
paused state + resume control with no elapsed/remaining/section title; Play
press produces audio without opening the app.
**10/10 rule**: all ten trials must pass; any failure fails SC-043.

## T-38 · SC-045 — Document end while locked: teardown, no resume anywhere

**Requirement** (SC-045 · FR-032e, FR-032f): In 10 out of 10 trials, a
document reaching its end while the device is locked results in the resident
notification being dismissed and the resident component being torn down, with
zero instances of a surface remaining that offers to play or resume the
completed document, zero instances of audio produced at the moment of
completion, and zero instances of the next launch presenting a paused state or
a resume offer for that document.

**Setup**: playback near the end of a short document; device locked.

**Procedure — per trial (run 10×)**:
1. Start a short document; lock the device; let it play to its end.
2. At completion, confirm: the resident notification is dismissed and the
   resident component is torn down (counts via `dumpsys`).
3. Confirm zero audio at the moment of completion and zero remaining surfaces
   offering play/resume of the completed document.
4. Reopen the application and open the same document: confirm **no** paused
   state and **no** resume offer for it (completion cleared the stored
   position, FR-032b); playback starts from the first spoken position.
5. Confirm the teardown is not recorded as an interruption, a service kill, or
   a failure in the diagnostic record.

**Observe**: notification/service counts at end; audio at completion; next
launch presentation; record entries.
**Per-trial pass**: dismissal + teardown at end; zero play/resume surfaces;
zero completion audio; zero paused-state/resume offer on next launch.
**10/10 rule**: all ten trials must pass; any failure fails SC-045.

## T-39 · SC-046 — Locked surface shows title + section title, no redaction

**Requirement** (SC-046 · FR-028b): In 10 out of 10 trials on a locked
device with no phone-wide notification-content redaction enabled, the
lock-screen surface shows the document title and the section title rather than
a generic "new notification" placeholder, and zero instances occur of the
application redacting that content, hiding it, or presenting a control that
would.

**Setup**: device locked; phone-wide "hide notification contents" setting
**off** (verified in system settings); playback running with the current
position after at least one heading.

**Procedure — per trial (run 10×)**:
1. Confirm the phone-wide hide-contents preference is disabled.
2. With the device locked, wake to the lock screen; observe the
   notification.
3. Confirm the document **title** and the current **section title** are shown
   — not a generic "new notification" placeholder.
4. Inspect the application's settings and surfaces: zero redaction, zero
   hiding, zero control that would redact or hide (the app adds no
   hide-on-lock mode of its own, FR-028b).

**Observe**: lock-screen content vs placeholder; app-side redaction surfaces.
**Per-trial pass**: title + section title visible locked; zero app-side
redaction/hiding controls.
**10/10 rule**: all ten trials must pass; any failure fails SC-046.

## T-40 · SC-047 — Section title tracks every state change that can alter it

**Requirement** (SC-047 · FR-028d): In 10 out of 10 trials, the section title
on the lock-screen surface is correct immediately after a boundary crossing, a
resume, a skip in either direction, and a re-anchor following an engine
change, and zero instances occur of a title naming a section the position has
already left or a section it has not yet entered.

**Setup**: doc (a) with ≥ 2 headings; playback running; device locked.

**Procedure — per trial (run 10×)**:
1. Cross a section boundary during playback: confirm the section title updates
   immediately to the newly entered section.
2. Resume a paused session: confirm the title matches the resumed position.
3. Skip forward and backward across a heading: confirm the title follows the
   skip (a stale title across a 5 s skip is a defect).
4. Re-anchor after an engine change (standin_a → standin_b): confirm the
   title is re-resolved from the document currently being read.
5. At every step, assert the title names neither a section already left nor a
   section not yet entered (most recent preceding heading per FR-028c).

**Observe**: section title after each of the four event classes.
**Per-trial pass**: correct title immediately after every event class; zero
stale or forward-looking titles.
**10/10 rule**: all ten trials must pass; any failure fails SC-047.

## T-41 · SC-048 — Genuine failure reason persists in the app, no expiry

**Requirement** (SC-048 · FR-037, FR-037a): In 10 out of 10 trials, a
genuine failure arising while the device is locked — an engine failure, an
audio-device failure, or a refused start — still shows its full reason in the
application when it is opened at any later time, with no expiry window
applied, so that the persistence FR-037 requires is distinguishable in testing
from the focus-loss expiry of SC-031 rather than the two surfaces agreeing on
everything.

**Setup**: fault-injection fixture; device locked.

**Procedure — per trial (run 10×)**:
1. Trigger a genuine failure while locked (inject `engine_unreachable`,
   `device_refused`, or a refused start).
2. Wait well beyond 3 s (use 10 minutes in half the trials).
3. Open the application: confirm the **full reason** is still shown.
4. Contrast check: a focus-loss pause reason in the same trial set must
   already have expired (SC-031) — the two reason classes must behave
   differently.

**Observe**: failure reason presence at > 3 s (and at 10 min); contrast with
the expired focus-loss reason.
**Per-trial pass**: full failure reason shown at any later time, zero expiry;
focus-loss reason expired in the contrast check.
**10/10 rule**: all ten trials must pass; any failure fails SC-048.

## T-42 · SC-049 — Residency counts: one while resumable, zero after end

**Requirement** (SC-049 · FR-009a, FR-032f): In 10 out of 10 trials,
after a paused session has been playing and has been paused, exactly one
resident component and one ongoing notification exist, and the same holds
after an interrupted session; and in 10 out of 10 trials where the document
reached its end, zero resident components and zero ongoing notifications
remain, so that residency is observable as tracking whether there is
something to resume rather than tracking that playback has ever been started.

**Setup**: playback states reachable on demand; `dumpsys` counters.

**Procedure — per trial (run 10×)**:
1. Play, then pause: count resident components and ongoing notifications —
   exactly **one** of each.
2. Simulate an interrupted session (kill the UI / interruption leaving a
   resumable position): counts remain exactly **one** of each.
3. Play a document to its end: counts must be **zero** resident components
   and **zero** ongoing notifications.
4. (The liveness half — a Play press from the resident notification
   producing audio — is the SC-043 verdict; this section is the
   count/teardown verdict.)

**Observe**: component/notification counts after pause, after interruption,
after end.
**Per-trial pass**: 1/1 after pause and after interruption; 0/0 after end.
**10/10 rule**: all ten trials must pass; any failure fails SC-049.

## T-43 · SC-050 — One-hour idle pause: zero synthesis, then ≤ 3 s start

**Requirement** (SC-050 · FR-015a, FR-015c): In 10 out of 10 trials, a
document left paused for at least one hour produces zero synthesis and holds
an empty pre-roll buffer throughout the idle period, and zero instances
occur of a per-character engine being billed for audio during that idle
period; and in 10 out of 10 trials, pressing play after that idle period
begins pre-roll and produces audible audio within the 3-second bound of
FR-027.

**Setup**: **normative reference device** (README §2); playback paused
mid-document; stand-in billed-bytes counter readable
(`engine.billed_bytes` trace line).

**Procedure — per trial (run 10×)**:
1. Pause the document; record the stand-in's billed-bytes counter C0 and
   confirm the pre-roll buffer is empty (`preroll.buffered_secs = 0`).
2. Leave the session paused, untouched, for **one hour**, sampling
   `preroll.buffered_secs` and the billed-bytes counter at 10-minute
   intervals.
3. Assert throughout: zero synthesis, empty pre-roll buffer, billed-bytes
   counter unchanged (zero billing for the idle period).
4. Press Play; confirm pre-roll begins and measure press-to-first-audio:
   **≤ 3 s**.

**Observe**: pre-roll level and billed bytes across the hour; post-idle
press-to-audio interval.
**Per-trial pass**: zero synthesis / empty buffer / zero billing for the
whole idle hour; post-idle Play produces audible audio ≤ 3 s.
**10/10 rule**: all ten trials must pass; any failure fails SC-050. (The
one-hour hold makes this the longest section; run it first in a session of
trials or let it run while other sections' paperwork is completed.)

## T-44 · SC-050b — Discarded pre-roll bills zero characters beyond one budget

**Requirement** (SC-050b · FR-015b): In 10 out of 10 trials, discarded
pre-roll (audio prepared but never played) bills zero characters on a
per-character engine beyond the smallest documented per-request byte budget,
so pre-roll never spends the reader's free quota on minutes that are
discarded.

**Setup**: per-character (cloud) engine path simulated by the stand-in's
billed-bytes counter (harness fixture); smallest documented per-request byte
budget from the versioned provider table (constitution Principle X; the
recorded Google Cloud TTS limit is 5,000 bytes).

**Procedure — per trial (run 10×)**:
1. Record the stand-in's billed-bytes counter C0.
2. Press Play (pre-roll prepares), then discard the session **without
   playing** (Stop, or kill the session) before any audio is heard.
3. Read the counter C1 after the discard.
4. Assert `C1 − C0 = 0` beyond the smallest documented per-request byte
   budget — i.e. no more than one minimum request chunk was ever synthesised
   for the discarded pre-roll, and zero characters beyond it.

**Observe**: billed-bytes delta across the discarded pre-roll.
**Per-trial pass**: billed delta within one smallest per-request budget,
zero beyond it.
**10/10 rule**: all ten trials must pass; any failure fails SC-050b.

## T-45 · SC-051 — Two-stand-in failover drill (first-class procedure)

**Requirement** (SC-051 · FR-023b, FR-024): In 10 out of 10 trials within
this feature, using two stand-in engines that honour the same engine
contract (`standin_a`, `standin_b`, both `engine_tier: test_double`), an
engine failing mid-document is followed by the other engine taking over with
no document restart, and forcing the second engine to fail as well leaves
the session paused with a stated reason rather than in silence. This
criterion verifies the recovery mechanism only; SC-044 remains the evidence
that the mechanism behaves the same way against real engines.

**Setup**: both stand-ins registered (admissible only while no
non-`test_double` adapter is registered); fault-injection fixture; playback
on `standin_a`.

**Procedure — per trial (run 10×)**:
1. Start playback on `standin_a`; play to a mid-document position P; record P.
2. Inject a **mid-document failure** into `standin_a` (the injectable
   failure both stand-ins honour per the engine contract).
3. Confirm the named failure is presented and the 3-second offer names
   `standin_b`; let the window expire without choosing.
4. Confirm `standin_b` takes over **from the last spoken position P** —
   zero document restart (resume position == P, not 0).
5. Immediately force `standin_b` to fail as well (second injection).
6. Confirm the session ends **paused with a stated reason** — not silence,
   not an endless offer loop — and both failures are recorded as named
   failures in the diagnostic record.

**Observe**: failure presentation; offer contents; takeover position vs P;
double-failure end state; record entries.
**Per-trial pass**: takeover with no restart; double failure → paused with
a stated reason; zero silence.
**10/10 rule**: all ten trials must pass; any failure fails SC-051.

## T-46 · SC-052 — Expired fallback follows the declared order, consent-bounded

**Requirement** (SC-052 · FR-023c): In 10 out of 10 trials, a failure
resolved through the expired 3-second fallback selects an engine the
declared order places after the failed one rather than restarting the walk at
the top of the sequence, the engine selected is one the offer named, and
zero instances occur of an engine the reader has not consented to being
selected by the fallback.

**Setup**: `standin_a` active, `standin_b` registered; no cloud engine
consented (consent state in application-private storage); fault-injection
fixture.

**Procedure — per trial (run 10×)**:
1. Play on `standin_a`; inject a mid-document failure; record the offer's
   named engines.
2. Let the 3-second window expire without choosing.
3. Confirm the engine selected is the one the declared order places **after**
   `standin_a` (i.e. `standin_b`) — the walk does not restart at the top
   of the sequence, and the selected engine is one the offer named.
4. Assert zero selections of any engine the reader has not consented to
   (with no cloud engine consented, the fallback never selects a cloud or
   evaluated-fallback engine; the record shows the selection and its tier).

**Observe**: selected engine vs declared order; offer contents; consent
state; record entries.
**Per-trial pass**: next-in-order selection, named in the offer; zero
non-consented selections.
**10/10 rule**: all ten trials must pass; any failure fails SC-052.

## T-47 · SC-053 — Playing surface: no dismissal; pause is the route

**Requirement** (SC-053 · FR-028e): In 10 out of 10 trials, a surface
carrying playing audio offers no dismissal affordance and presents a working
pause control, and pressing pause on a transient reason surface dismisses
that surface within 3 seconds while pressing pause on the persistent
resident surface leaves it present and dismissable with its paused state and
resume control intact, so that a Play press from it still produces audio.

**Setup**: playback running (locked); a transient reason surface reachable
(failure or focus-loss pause); the resident surface reachable (paused
session).

**Procedure — per trial (run 10×)**:
1. With audio playing, attempt to swipe the notification away: confirm the
   surface offers **no dismissal affordance** and the gesture does not
   dismiss it (ongoing flag, `setOngoing(true)`); the pause control is
   present and works.
   (Where a platform or OEM build permits dismissal regardless, the
   Edge-Case rule governs: the document pauses rather than going on speaking
   with no control surface, and the stored position is preserved either
   way.)
2. Produce a transient reason surface (failure or focus-loss pause); press
   pause; confirm that surface is **dismissed within 3 seconds**.
3. Produce the persistent resident surface (paused session); press pause /
   observe post-pause: confirm the surface **remains present and
   dismissable** with its paused state and resume control intact.
4. Press Play from that resident surface: confirm audio still produces
   (the control survived).

**Observe**: dismissal behaviour while playing; transient-surface dismissal
latency; resident-surface persistence; Play liveness.
**Per-trial pass**: no dismissal while playing with a working pause control;
transient surface dismissed ≤ 3 s after pause; resident surface present and
dismissable with state + control intact; Play from it produces audio.
**10/10 rule**: all ten trials must pass; any failure fails SC-053.

## T-48 · SC-054 — Rate press takes audible effect within 2 seconds

**Requirement** (SC-054 · FR-019c): In 10 out of 10 trials, a rate control
press takes audible effect within 2 seconds on a mid-range arm64 device with
the document already parsed.

**Setup**: **normative reference device** (README §2); playback running;
document already parsed.

**Procedure — per trial (run 10×)**:
1. Press a rate control (up or down one rung); the harness timestamps the
   press.
2. Measure the interval to the audible speed change (the speech rate
   change is itself the audible confirmation, FR-018a).
3. Assert the interval is **≤ 2 s** and the rate moved exactly one rung
   (cross-check against SC-026).

**Observe**: press-to-audible-effect interval per trial.
**Per-trial pass**: audible effect ≤ 2 s; exactly one rung.
**10/10 rule**: all ten trials must pass; any failure fails SC-054.

---

# Special procedures (not plain on-device trials)

## S-1 · SC-037 — Static-scan + manual-absence procedure (record never surfaced)

**Requirement** (SC-037 · FR-039a): Zero instances of the application
presenting, listing, or exporting the diagnostic record, so that FR-039a is
verified as an absence rather than left to inspection.

**This is a static-scan + manual-absence procedure, not an on-device
trial.** Refs: T074 (diagnostic reader CLI + scanner — the only reader),
T081 (shell-purity gate).

**Procedure**:
1. **Static scan** — from the repository root, grep every application
   surface for record-read/export strings:
   `grep -rniE 'diag(record)?|session.record|export|share|read.*record|record.*(read|view|list)' android-shell/ src-tauri/`
   Assert **zero hits** that present, list, read, or export the record. The
   only sanctioned reader is the developer tooling in `crates/diag` (T074)
   — never an app surface. Any hit is a failure unless it is inside the
   T081 allowlist and is a type/getter/delegation position (the T086
   human-review item).
2. **Manual-absence walk** — on the device, open every application screen
   (all Tauri views), the settings surface, and every lock-screen /
   notification-shade surface; confirm no menu item, button, screen, or
   affordance presents, lists, or offers to export the diagnostic record.
3. **Cross-reference** — confirm the record is reachable only through
   developer tooling (`crates/diag` CLI, `adb shell run-as`), and that
   T081's `check-shell-purity.sh` allowlist contains no record-read or
   export path.

**Pass**: zero static-scan hits outside the allowlist; zero in-app or
lock-screen affordances in the manual walk.
**Rule**: the criterion passes only when both legs return zero; any hit
fails SC-037. Evidence: the grep output (zero hits) and the signed manual
walk checklist go to the recording sheet.

## S-2 · SC-010 — Surface procedure: zero body text on any notification surface

**Requirement** (SC-010): Zero instances of document body text appearing in
the notification, on the lock screen, or in the notification shade, with the
only document-derived content permitted there being the document title and
the current section title.

**This is a three-leg surface procedure, not a plain on-device trial.**
All three legs must pass.

**Procedure**:
1. **Host leg** — `cargo nextest run -p player prop_surface` (T051,
   `crates/player/tests/prop_surface.rs`) is green: the property test
   asserts the surface is title + section + progress + controls ONLY —
   never body, elapsed, remaining, rate, or voice; section = most-recent-
   passed else absent; determinacy fixed at session start; title from
   metadata else the filename-stable placeholder, never the first sentence.
2. **Static leg** — grep the platform surface for body-text render paths:
   `grep -rniE 'BigTextStyle|InboxStyle|setBigText|setBigContentTitle|bodyText|documentText|docBody|prose|spokenText|currentSentence|paragraphText' android-shell/ src-tauri/`
   Assert **zero hits** (outside the T081 allowlist's type/getter
   positions). The notification render path may carry title, section title,
   and progress only.
3. **Device leg (locked visual check, 10/10)** — with the device locked and
   playing, in 10 trials spanning a boundary crossing, a resume, a skip in
   each direction, and an engine change, capture the lock screen and the
   notification shade (screenshot or direct inspection) and assert the
   content is exactly: **document title + current section title + progress
   indication + transport controls** — zero body text, zero elapsed or
   remaining time, zero rate value, zero voice name (the FR-028 series,
   exactly).

**Pass**: host leg green; static leg zero hits; locked visual check 10/10.
**Rule**: any body-text instance in any leg fails SC-010. Evidence: T051
test output, grep output (zero hits), and the 10-row locked-visual sheet.

## S-3 · FR-026 — Offline-start procedure (airplane mode)

**Requirement** (FR-026): The system MUST NOT depend on network
connectivity to start or continue playback.

**Procedure — per trial (run 10 trials, harness convention)**:
1. Enable airplane mode (`adb shell cmd connectivity airplane-mode enable`);
   confirm no active default network (`adb shell dumpsys connectivity`).
2. Force-stop the app; cold-start it; open a local document; press Play.
3. Continue playback for **60 s** without touching the device.
4. Confirm audible audio throughout, the position advancing, and zero
   network activity for the app uid (uid network stats zero bytes; the
   record carries no network-dependent entries).

**Observe**: audio continuity for 60 s; position advance; egress bytes.
**Per-trial pass**: playback starts and continues 60 s with no network;
zero egress.
**10/10 rule**: all ten trials must pass; any failure fails the FR-026
procedure.

## S-4 · SC-044 — CARRIED-FORWARD (not runnable here)

**Requirement** (SC-044 · FR-023b): On a device carrying at least two working
engines, one offline and one cloud, an engine failing mid-document is followed
by the other engine taking over with no document restart, and forcing the
second engine to fail as well leaves the session paused with a stated reason
rather than in silence.

**Status: CARRIED-FORWARD — not runnable in this feature.** FR-024 places
synthesis in feature `002-fr-024-synthesis-adapter`; real engines do not
exist yet, so the hardware evidence cannot be produced here. Owner: the
synthesis feature, which re-verifies SC-044 once a real offline engine and a
real cloud engine are delivered. The recovery mechanism itself is discharged
now against the two contract-honouring stand-ins by **SC-051** (§T-45), which
is explicitly NOT a discharge of SC-044. T087 records the SC-044 verdict as
DEFERRED with owner = synthesis feature, and logs it as carried-forward
hardware debt (per T086's task text).

## S-5 · Zero-occurrence criteria — record scans routed to T087 (NOT trials)

These are zero-occurrence criteria verified by scans over the retained
diagnostic record. They are **not** on-device trials: T087 executes the record
scans as `cargo nextest run -p diag` and cites the scan target by name, with
T068b's canary results (`crates/diag/tests/prop_scanner_canary.rs`) as the
evidence that those scans can fail at all — a scan that has never been shown
to reject a planted violation is not evidence of zero occurrences. The device
procedure is only to generate and pull the record:
`adb shell run-as <pkg> cat files/diag.jsonl` (or the `crates/diag` reader
CLI), exercised by the T-34/T-35/T-45/T-46 trials above.

- **SC-005** (FR-005, FR-039) — *Record-scan procedure referenced*: the
  playing-advance rule scan over the retained record (implemented in
  `crates/diag/src/scan.rs`, T074; property-gated by T068
  `prop_no_prose.rs`): zero `transport_change` entries with
  `transport: "playing"` that are not followed, in the same session, by a
  later entry carrying a strictly greater `position_secs` (the window being
  the interval to the next entry of the same session), except where the
  intervening entries are a completion, a stop, a pause, a device loss, or a
  named failure. T087 cites the scan target by name plus the T068b canary
  (planted `transport:"playing"` with no greater `position_secs` follow-up
  must report non-zero).
- **SC-012** (FR-031, FR-032c, FR-023a) — *Record-scan procedure
  referenced*: over the retained record, zero `session_start` entries without
  a user action. Every session start must be attributable to a user-initiated
  play press, except the two permitted auto-starts: the FR-032c
  deliberate-pause reopen after its 3-second offer expires with the
  application in the foreground, and the FR-023a engine-fallback continuation
  after its 3-second offer expires (a continuation of an already-started
  session, not a session start). T087's scan pairs each `session_start` with
  its initiating event chain.
- **SC-019** (FR-032b) — *Record-scan procedure referenced*: over the
  retained record, zero resume offers for Stopped or completed documents.
  T087's scan pairs each reopen/offer event with the prior `session_end`
  entry for the same content fingerprint and asserts zero offers where
  `session_end_reason` ∈ {`stopped`, `completed`}.

## S-6 · Host-decided criteria — N/A on-device (gate references)

Decided by host gates; no device procedure exists and none is required. T086
records "N/A on-device — see gate ref" for each:

- **SC-006** (FR-007) → **T081** `scripts/check-manifest.sh`: asserts
  `<service android:foregroundServiceType="mediaPlayback">` +
  `FOREGROUND_SERVICE` + `FOREGROUND_SERVICE_MEDIA_PLAYBACK` +
  `minSdk(Version) 29` in the built manifest.
- **SC-015** (FR-019a) → **T055** `crates/engine-core/tests/prop_pitch.rs`:
  deterministic host measurement — fundamental-frequency contour and
  spectral-peak positions of identical PCM at all seven rungs match within
  ±50 cents of F0, including 2.0x→4.0x.
- **SC-035** (FR-042) → **T081** `scripts/check-shell-purity.sh`: static
  scan of the platform surface for transport, rate, position, buffering, and
  focus-response logic (forbidden call shapes fixed in T013); any match
  outside the declared allowlist fails; the residue of allowlisted
  `transport|position_secs|rate|gain|skip` occurrences is the T086
  human-review item (read each flagged occurrence and confirm it is a type,
  getter, or delegation, not a computation — recorded in VERIFICATION.md,
  never automated).
- **SC-056** (FR-045) → **T068** `crates/diag/tests/prop_no_prose.rs`:
  scanner asserting zero entries containing document text, `detail` absent
  or ≤ 200 characters matching the non-prose pattern, and every `failure`
  and `refusal` carrying `error`.
- **SC-057** (FR-023c, FR-044) → **T081** closed-set/registry-reference
  check: every governed closed set (rate rungs, focus outcomes, transport
  states, error kinds, session-end reasons) contains no catch-all,
  "other", "internal", or "unknown" arm; every `engine` identifier with
  `engine_tier != test_double` resolves to a constitution registry
  name+version; every `engine_tier == test_double` identifier is
  `standin_a` or `standin_b`; and `test_double` appears in any selectable
  set only while no non-`test_double` adapter is registered.

---

*End of procedure set. 48 trial-runnable sections (T-01–T-48) + 6 special
procedures (S-1–S-6). Execution and attestation: T086 (device) / T087
(VERIFICATION.md) only.*
