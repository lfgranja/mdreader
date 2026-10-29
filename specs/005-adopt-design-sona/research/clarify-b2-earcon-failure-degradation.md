# Clarification B2 — Earcon failure / degradation behavior

> Research note for `specs/005-adopt-design-sona/`. Question: when an expected
> confirmation/warning earcon cannot be produced (synthesis error, audio-channel
> allocation failure), what does human-factors literature and safety-critical
> interface practice recommend — given the spec already prohibits *silent
> suppression* as policy ("every triggered signal sounds its earcon")? Scope:
> the failure case only. Claims cited inline (URL + authors + year).

## 1. Lost alerts → mode error → eroded trust

The danger is not the missing sound but the *unobserved absence*: the driver
acts on a presumed state the system never confirmed, then compounds the error.

- **Sarter & Woods (1995)**, *Human Factors* 37:5–19, "How in the World Did
  We Get into That Mode? Mode Error and Awareness in Supervisory Control"
  (https://doi.org/10.1518/001872095779049461). Poor feedback about
  automation state is a principal contributor to "automation surprises" and
  mode error; the recurring operator questions are "What is it doing? Why?
  What next? How did we get into that mode?" They recommend *auditory/kinetic
  clues* and "displaying recent data" as mode-awareness aids because the visual
  field is already occupied — i.e., the audible channel carries state the
  driver otherwise cannot see. A silently failing earcon removes one of the few
  eyes-free state cues.
- **Parasuraman & Riley (1997)**, *Human Factors* 39(2):230–253, "Humans and
  Automation: Use, Misuse, Disuse, Abuse"
  (https://doi.org/10.1518/001872097778543886). Inadequate feedback about an
  automation's actions/intentions drives *misuse* (over-reliance) and *disuse*
  (under-utilization); trust, workload, and risk shape whether the operator
  uses the automation at all. A confirmation channel that sometimes produces
  nothing and never says so is exactly that failure mode.
- **Bliss & Acton (2003)**, *Applied Ergonomics* 34(6):507–516, "Alarm
  mistrust in automobiles: how collision alarm reliability affects driving"
  (https://doi.org/10.1016/j.apergo.2003.07.003). With 70 simulator drivers,
  alarm and swerving reactions improved monotonically with reliability
  (50/75/100%); design imperative: **maximize alarm reliability while
  minimizing invasiveness**. The Bliss/cry-wolf line (Breznitz 1983;
  probability-matching ≈90% of subjects) shows operators *calibrate* to
  perceived reliability — a channel that sometimes silently fails trains the
  driver to discount it.

Direct studies of "missing alert → mode error" in-vehicle are sparse, but the
chain is supported: silent omission of an expected alert is functionally a
low-reliability alert, and Bliss & Acton (2003) + Parasuraman & Riley (1997)
document the resulting mistrust/disuse.

## 2. Standards — presentation reliability and channel redundancy

- **ISO 15006:2011** (confirmed current 2024), in-vehicle auditory
  presentation (https://www.iso.org/standard/55322.html). Classifies signals
  by temporal response class: short-term 0–3 s, medium 3–10 s, long >10 s;
  time-critical safety warnings have priority; short-term signals "should be
  sent immediately after a critical event is detected." Mandates audibility
  (SNR ≥ 1.3 masked specific loudness) and onset-to-full-loudness ≤30 ms for
  critical/warning sounds — i.e., the sound must be *present and timely* —
  but does **not** mandate a redundant non-auditory channel or a fallback
  when audio is unavailable. Redundancy is left to the designer.
- **FMVSS No. 141** (49 CFR 571.141), Minimum Sound Requirements for Hybrid
  and Electric Vehicles
  (https://www.ecfr.gov/current/title-49/subtitle-B/chapter-V/part-571/subpart-B/section-571.141).
  Requires the pedestrian-alert sound to be emitted whenever the propulsion
  system is activated in the relevant conditions — obligation on the *vehicle*
  to produce the sound; no "sound unavailable → substitute visual" provision
  (receiver is not in the vehicle). Analogue for "the alert must sound," not a
  fallback model.
- **UN ECE R138** (https://unece.org/sites/default/files/2025-06/R138r1am4e.pdf).
  §6.2.7: in the mandatory speed range the AVAS sound "shall always be
  emitted"; pause function allowed **only outside** that range. Precedent for
  "system-initiated warnings never queued/dropped." Mandates *continuity of
  emission*, not a fallback channel.

Net: standards require the sound to happen (reliability of emission) and
prioritize it temporally, but **do not mandate redundant channels** for a
failed emission. Redundancy is a design-level decision.

## 3. Multimodal redundancy / equivalent completion

- **Redundant-signals effect** (Miller 1982 race-model inequality; reviewed
  Frontiers in Psychology 2017, https://doi.org/10.3389/fpsyg.2017.01141):
  redundant audio+visual cues yield faster detection than either alone
  (statistical facilitation). On failure: a redundant visual/state channel
  means the operator still perceives the event when audio drops — the task
  does not fail.
- **Partan & Marler (1999/2005)** (cited Partan et al. 2009,
  https://doi.org/10.1016/j.anbehav.2009.03.011): multimodal signals
  classified as *redundant* (same message across channels) vs. non-redundant;
  redundant backup channels "ensure that the signal is transmitted even in
  noise." Supports "state stays observable via display + diagnostics" when
  audio is lost.
- **DTIC redundant-alerts study** (2007,
  https://oai.dtic.mil/oai/oai?identifier=ADA466336): visual-only alerts up to
  63% slower than redundant visual+auditory (or visual+tactile); redundant
  modalities rated more effective at getting attention. Redundancy buys
  attention, not task completion — but its absence on failure must not
  *create* a new task (retry storm).
- **Horvitz et al. (2003)**, *CACM* 46(3):52–59, "Models of Attention in
  Computing and Communications" (https://doi.org/10.1145/602421.602422):
  critical events bypass deferral; bounded deferral waits only for a
  perceptual breakpoint within a deadline. Applied to failure: a failed
  earcon is lost, not deferred — so the system must not queue/retry
  indefinitely; the critical event has passed its window.

## 4. GUI feedback when a feedback sound fails (non-safety-critical)

For an *in-app confirmation earcon* (not a safety alarm), literature and
practice converge on: **do not block the action, do not retry in a loop,
surface the failure through the other channels, and record it.**

- The transport state change has already committed; the earcon is feedback,
  not the operation. Blocking/retrying would violate the action–sound
  attribution window (≤100 ms, FR-010) and add latency to a driving-critical
  control.
- **Redundant-channel practice** (ISO 15006 §5 temporal classes; Bliss &
  Acton 2003 "minimize invasiveness"): the visual state change (transport
  glyph, progress, banner) plus haptic pattern (004 contract) already carry
  the confirmation; the earcon is additive. When audio fails, visual + haptic
  still complete the feedback — "equivalent completion."
- **Diagnostic/observability** (constitution: append-only JSONL, ≤30-day/10 MB
  trim; `detail` ≤200 chars identifier-pattern): a failed emission is a
  *diagnostic event*, not a user-facing alarm. Log it (error kind +
  channel/allocation reason) so the failure is observable to the maintainer
  without a new UI surface or retry storm.
- **Bounded, not infinite, retry**: at most one re-attempt on the next audio
  callback boundary is defensible (Horvitz bounded deferral); unbounded retry
  storms are unsupported by any cited source and would compete with speech
  synthesis for the audio stream.

## 5. Implications for the spec (FR-009 failure case)

Recommendation for FR-009's failure case (synthesis error or audio-channel
allocation failure):

1. **Never silently suppress, never block.** The transport action completes
   and commits normally; narration is not paused/delayed to "make room" for a
   retry (FR-009 forbids this for the success path — keep it for failure).
2. **Equivalent completion via redundant channels.** The visual state change
   (transport glyph + progress + banner) and the haptic pattern (per 004
   contract) carry the confirmation; the earcon is additive redundancy. No
   new UI affordance required.
3. **Bounded retry: at most one re-fire on the next audio callback
   boundary** (Horvitz bounded deferral; ISO 15006 short-term 0–3 s class).
   If that also fails, **stop retrying** — no retry storm.
4. **Fail-observable, not fail-loud.** Emit a diagnostic record (append-only
   JSONL, typed error kind + channel/allocation reason, `detail` ≤200 chars,
   identifier-pattern only, no document text). No in-app error surface, second
   earcon, or toast for a failed confirmation sound.
5. **Trust preservation.** Because operators calibrate to perceived
   reliability (Bliss & Acton 2003; Parasuraman & Riley 1997), the only
   acceptable "failure" is one the system can observe and count — never one
   it hides. Require the failure to be counted in diagnostics, not reported
   to the driver.

This keeps FR-009's "no silent suppression as policy" intact (the policy
concerns *not choosing* to suppress an available sound) while defining a
bounded, observable, redundant-channel degradation when the sound is
physically impossible.
