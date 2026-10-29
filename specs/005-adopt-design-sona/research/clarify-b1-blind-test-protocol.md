# Clarification B1 — Blind Identification Test Protocol for the Sona Earcons

> **Feature:** `specs/005-adopt-design-sona` (SC-002: "10 of 10 per type without looking at the screen")
> **Scope:** research note for the maintainer-driver single-user gate; not a clinical efficacy claim.
> **Method:** primary-source review (peer-reviewed auditory-display literature + ISO/IEC standards), 2026-10-24.

## 1. Where "10 of 10 per type" sits relative to established practice

**SC-002 as written** (10/10 correct for each of play, pause, skip, rate, warning, failure) is a *single-subject acceptance gate*, not a published psychometric criterion. Established practice:

- **Earcon identification baselines.** Without training, recognition of abstract earcons is ~10%; after 5–10 min of training with feedback it reaches ~80% (Brewster, Wright & Edwards 1993, *INTERCHI '93*, pp. 222–227; Sonification Handbook ch. 14, §14.4; Dingler & Lindsay 2008, ICAD). A cold 100% is therefore unrealistic for any earcon set; a familiarization phase is mandatory.
- **Bonebright & Nees 2009**, *Applied Cognitive Psychology* 23(3):431–445: earcons do not impair spoken-passage comprehension — supports additive (non-ducking) mixing, already adopted in FR-009.
- **Nees & Liebman 2023**, *Auditory Perception & Cognition* (meta-analysis of brief audio alerts): earcons are among the least-interfering alert types, but "best practice … thorough testing of the proposed alerts in their [target] environment" — i.e., the gate must run in-car, not in a quiet lab.
- **Edworthy et al. 2017**, *Ergonomics* (IEC 60601-1-8 alarm sets): recognizability is evaluated with a **confusion matrix** over trained listeners; prototype sets must reach near-perfect recognition with *zero systematic confusions* before standardization. This is the closest analog to SC-002 and it is matrix-based, not a bare count.
- **ISO 7731:2003** (danger signals) Annex C specifies a *subjective listening test* for audibility over noise (masked threshold), not a fixed trial count. **ISO/TS 9241-126:2019** §9 (earcon construction) and §9.5 (earcon duration) give construction guidance but no pass threshold. **ISO 14915** (withdrawn, superseded by ISO 9241-115/171) is general multimedia. None of these standards endorses "10/10" as a criterion.
- **Psychophysics.** With 6 response options, chance = 1/6 ≈ 16.7%; P(10/10 by guessing) = (1/6)^10 ≈ 1.7×10⁻⁸, so 10/10 is a stringent personal gate — but it is a *design-verification* result for N=1, not a population claim. The established method is the **constant-stimulus method with randomized presentation and a confusion matrix** (Clarke 1957, *J. Acoust. Soc. Am.* 29:715–720, constant-ratio rule for confusion matrices).

**Adequacy verdict:** 10 trials/type is the *floor*, not the target. Recommend **≥15–20 trials per type** (6 types → 90–120 trials), randomized, with a confusion matrix, so a single slip does not flip the verdict and confusions become diagnosable.

## 2. Can a 40 ms directional chirp be told apart?

- **Luo et al. 2007**, *Hearing Research* 224(1-2):75–83: FM-sweep direction-identification threshold ≈ 20 ms for English/Mandarin listeners; tone-language speakers are *better* at direction ID; discrimination thresholds ≈ 20 ms. Direction ID is harder than detection.
- **Schouten 1985** (reported in Schouten & Pols 1989, *Perception & Psychophysics* 46(3):235–244): upward sweeps need *shorter* durations/slower rates than downward sweeps; discrimination tracks Δf (initial–final difference), not rate alone. Downward sweeps are the harder direction.
- **PMC7078663 (2020)**, *Frontiers in Neuroscience*: cortical MMNm fires at 10 ms, but psychophysically, when sweep duration exceeds ~40 ms (slower rate) up/down accuracy *equalizes*; at ≤40 ms an up/down asymmetry remains. **40 ms is the boundary**, not a comfortable margin.
- The app's chirp (880→1108 Hz, +228 Hz ≈ +4.1 semitones; 880→660 Hz, −220 Hz ≈ −5.1 semitones, both in 40 ms ≈ 5.5–5.7 kHz/s) sits exactly on that boundary, and the harder direction (down) carries the smaller relative interval. Empirical risk: skip-backward will be the failure mode.

**Verdict:** a 40 ms chirp is *not* reliably direction-identifiable for untrained listeners; the literature supports ≥80 ms (or a full-octave interval) plus a redundant cue (rhythm/timbre) per Brewster's earcon guidelines.

## 3. Presentation channel: loudspeaker vs. headphones (~1 kHz, brief)

- **ISO/TS 9241-126:2019** §7.5.1 (minimum signal duration) and §9.5 (earcon duration), and **ISO 7731:2003** (audibility over ambient noise via masked threshold) all presuppose the signal reaches the ear; the transducer is part of the system under test.
- Smartphone media speakers have a non-flat response with resonances typically around 1–2 kHz and roll-off at the extremes; in a car cabin the road/wind/engine floor sits ~60–70 dBA, so a ~1 kHz, 40 ms tone is at risk of masking. Headphones (flat-ish FR, better SNR, binaural cues) overestimate performance.
- **Recommendation:** the gate must run on the **reference device's built-in media speaker** in a parked car (engine idling or recorded cabin noise), fixed distance (~0.5 m), with a headphone condition recorded only as a secondary comparison. Headphone-only pass is not acceptable for a driving-first app.

## 4. Recommended protocol (single user, N=1)

1. **Familiarization (with feedback, 5–10 min):** labeled examples of all 6 earcons, repeated until self-reported confident; per Brewster/Dingler baselines this is required to reach ~80%+.
2. **Constant-stimulus gate:** seeded RNG order, 6 types × 15–20 trials (90–120 total), no two identical consecutively, **no feedback during the gate**, listener blind to the schedule, responses logged by an independent script (timestamp, presented, responded, correct) → confusion matrix.
3. **Attention traps:** 5–10% catch trials (silence or a novel tone) to detect inattention/guessing.
4. **Session structure:** 2–3 blocks of ~20–40 trials with breaks; retest after ≥24 h for test–retest stability.
5. **Pass criterion:** 100% per type (e.g., 20/20) **and** zero off-diagonal confusions in the matrix; **skip-forward and skip-backward scored as separate types** (distinct stimuli and actions) plus a direction sub-score.
6. **N=1 caveat:** document listener (age, hearing status, driving experience); treat as a design-verification gate, with the confusion matrix retained as the artifact for future redesigns.

## Implications for the spec (concrete recommendations)

1. **Amend SC-002** to encode the protocol above (constant-stimulus, randomized, ≥15–20 trials/type, 6 types with skip-fwd/skip-back separate, confusion matrix, familiarization-with-feedback prerequisite, blind, no in-gate feedback, catch trials, test–retest; pass = 100%/type + zero off-diagonal confusions).
2. **Lengthen the skip chirp** from 40 ms to ≥80 ms (or a full octave) and add a redundant cue (rhythm/timbre), because 40 ms is the documented boundary for reliable direction ID and the downward sweep is the harder direction.
3. **Mandate the in-car media-speaker channel** for the gate (engine-idle noise, fixed distance); headphone pass is secondary only.
4. **Add familiarization as a prerequisite** — untrained listeners top out near 80% even with good earcons.
5. **Retain the confusion matrix as the gate artifact**, not just a pass/fail count, so redesigns target specific confusions.