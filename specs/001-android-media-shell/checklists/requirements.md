# Specification Quality Checklist: Android Media Shell

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-10-01
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs)
      Checked. FR-002/FR-003 name "the Rust playback module (the player crate)" as the
      defined owner — a deliberate, recorded exception (clarify 2026-10-02, option A):
      naming the constitution-mandated owner is a binding, not a tool choice, and FR-002
      defines "the engine" as that module. No versions, SDK calls, or build tools appear. The
      Architecture Decision section below is the one place where technology *choices* appear,
      and it is explicitly marked as not being part of the specification body. Iteration 6
      recorded a second, narrower class of name that does appear in requirement text: a
      requirement may name the platform signal or record field it binds to — FR-010 already
      named the foreground-service refusal exception, FR-016a now names
      `ACTION_AUDIO_BECOMING_NOISY` and `AudioDeviceCallback.onAudioDevicesRemoved`, and
      FR-045/SC-005/SC-056 name the record's fields. These bind a requirement to a protocol
      contract rather than choosing a tool, they are what makes the requirements checkable
      rather than descriptive, and no technology choice is made by them.
- [x] Focused on user value and business needs
      Checked. Every user story is written from Luís's situation (commute, locked phone,
      navigation prompt, headphone removal), not from a subsystem's perspective.
- [x] Written for non-technical stakeholders
      Checked. Requirements use "the system", "the user", "the audio device", with two
      recorded deliberate exceptions: the owner names above (clarify 2026-10-02, option A —
      FR-002/FR-003/FR-042 name the constitution-mandated owner rather than describing it
      vaguely), and the exception below: a requirement that binds to a platform signal (FR-016a) or a record
      field (FR-045) names that signal or field rather than describing it vaguely, because
      "the platform's device-removal notification" is the vagueness this item exists to
      exclude. Every such name was already an established convention in this specification
      (FR-010, FR-028e) rather than a new exception to it.
- [x] All mandatory sections completed
      Checked. User Scenarios, Requirements, Success Criteria, Assumptions are all present
      and populated. Key Entities present and populated.

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
      Checked. Zero markers, re-verified after the 2026-10-02 clarification session
      (Iteration 6). Decisions that could have warranted clarification were resolved with
      documented assumptions or requirements instead: notification-permission flow (FR-021
      states the consequence rather than designing a prompt), sleep timer (out of scope),
      playlist/queue behaviour (out of scope, recorded in Assumptions), deferred and refused
      audio-focus answers (FR-013b), the declared sample-rate value (FR-033 fixes 22050 Hz),
      and the free-text field in the record schema (FR-045 forbids it outright).
- [x] Requirements are testable and unambiguous
      Checked. FR-013/FR-014/FR-041 separate ducking, transient pause, and permanent loss into
      three outcomes with different observable results. FR-016 states a one-second bound.
      FR-019 states a closed seven-step rate ladder and its two-second effect bound.
      FR-023 forbids silence as a degradation path. Iteration 5 removed the last vague
      quantities: "reduce playback volume" now carries the ~20% target of FR-013, and "a
      progress indication" plus the rate ladder of FR-019 are enumerated rather than
      described. Iteration 6 closed the remaining soft edges: FR-013b enumerates the three
      focus-request answers and what each does, FR-016a names both device-removal mechanisms
      and holds each to the one-second bound, FR-033 fixes the declared rate to a number
      rather than to "a single fixed rate", FR-044 enumerates the governed closed sets, and
      FR-045 states the record's admissible content as a positive list rather than as a
      prohibition without a boundary.
- [x] Success criteria are measurable
      Checked. Every SC carries a number, an exact-zero condition, or a deterministic
      automated check read as an exact-zero condition (clarify 2026-10-02, option A:
      SC-006 = zero missing declarations, SC-035 = zero shell-logic instances, SC-057 =
      zero catch-all arms). Iteration 6 added
      SC-055 (focus deferred-grant and refusal trials), SC-056 (a scanner reporting zero
      entries carrying document text), and SC-057 (an automated check enumerating the closed
      sets), and converted SC-005 from an assertion into a rule a scan can evaluate: a
      `transport: "playing"` entry conforms only if `position_secs` advances within the
      window between that entry and the next entry of the same session.
- [x] Success criteria are technology-agnostic (no implementation details)
      Checked. SC-001 through SC-008 describe trials and observable outcomes. SC-006 asks
      for an automated check of a manifest declaration, which is a verifiable artefact and
      not a tool or framework choice. Iteration 6 recorded the same narrower exception as
      above: SC-002, SC-005, SC-056, and SC-057 name a platform signal or a record field in
      order to be checkable at all, and SC-057 names the constitution's engine registry as the
      single place engine identifiers live rather than introducing a list of engines into the
      specification. No language, framework, or build tool is named in any criterion, with one
      recorded deliberate exception (clarify 2026-10-02, option A): SC-035 names the
      "Android and Tauri surface" as the check's scope — the constitution-mandated owner,
      not a tool choice.
- [x] All acceptance scenarios are defined
      Checked. Five user stories carrying 26 acceptance scenarios in total, each in
      Given/When/Then form. The count in earlier iterations of this checklist was stale; it is
      corrected here against the file as it stands.
- [x] Edge cases are identified
      Checked. 48 edge cases, including kernel-level device loss, rapid repeated play
      presses, focus loss mid-sentence, focus loss coinciding with a skip press,
      system-killed service, denied notification permission, permanent focus loss while
      screen-off, device power loss during playback, and a declined or expired resume offer.
      Iteration 6 tied the kernel-level/Bluetooth-dropout case to FR-016a, so that the case
      now names both removal mechanisms rather than asserting a single one is enough.
- [x] Scope is clearly bounded
      Checked. FR-024 explicitly places parsing, segmentation, synthesis, and voice
      selection outside this feature. FR-025 excludes Android Auto. Assumptions exclude
      sleep timer, presets, and playlists.
- [x] Dependencies and assumptions identified
      Checked. 15 assumptions recorded, including the dependency on an audio engine from
      another feature and the single-document session constraint. The count in earlier
      iterations of this checklist was stale; it is corrected here against the file as it
      stands. Iteration 6 rewrote the audio-handoff assumption, which previously said the
      declared rate's value was a plan-level decision that no requirement depended on — a
      claim FR-033 no longer allows. The assumption now fixes 22050 Hz and states precisely
      what remains plan-level (the negotiated native rate and the conversion path, neither of
      which may be obtained by adopting the device's default rate).

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
      Checked. Each FR maps to a user story acceptance scenario or to an Edge Case.
      FR-006/FR-007/FR-008 map to SC-006 and User Story 1; FR-009/FR-010/FR-011 map to
      User Story 1, SC-014, and the always-resident-component edge cases; FR-022/FR-023
      map to User Story 5. Iteration 4 closed the iteration-3 gap: FR-033, FR-034, and
      FR-035 (the audio handoff contract) now map to two explicit edge cases covering a
      mismatched sample rate and a mismatched channel layout. Iteration 6 mapped every
      requirement the session added or touched — FR-013b to SC-055, FR-016a to SC-002,
      FR-033 to SC-034, FR-039 to SC-005, FR-044 to SC-057, FR-045 to SC-056 — and the
      convention declared in the Functional Requirements preamble forced six pre-existing
      lettered requirements to be cited by the criterion that already tested them
      (SC-015→FR-019a, SC-017→FR-032a, SC-019→FR-032b, SC-021→FR-032d, SC-033→FR-017a,
      SC-036→FR-019b). A script check over the spec now reports zero lettered requirements
      without a success-criterion link in either direction.
- [x] User scenarios cover primary flows
      Checked. The primary flow (locked playback during a commute) is P1, along with
      interruption handling and headphone removal. Lock-screen controls are P2. Failure
      surfacing is P3 because it is cheap once the rest exists but is non-negotiable in
      intent.
- [x] Feature meets measurable outcomes defined in Success Criteria
      Checked. The shell is the deliverable; synthesis quality, document parsing, and voice
      naturalness are explicitly out of scope and belong to other features.
- [x] No implementation details leak into specification
      Checked, with two exceptions that are recorded deliberately. An "Architecture Decision
      (not part of this specification)" note names the technologies because the maintainer
      chose architecture A explicitly and a future reader needs to know why the shell is
      split this way. That note makes no requirement and states no technology constraint.
      The second exception (clarify 2026-10-02, option A): FR-002/FR-003/FR-042/FR-043 name
      the constitution-mandated owner in the requirement body, with FR-002 defining the
      terminology so the names are bindings rather than choices.

## Notes

- Items marked incomplete require spec updates before `/speckit.clarify` or
  `/speckit.plan`

### Validation iterations

- Iteration 1: 24 items checked. Two failed.
  - "No implementation details leak into specification" — the draft stated Rust ownership
    and the Kotlin mirror rule directly in FR-002/FR-003. Rewritten behaviourally; the
    technology names moved to a clearly marked non-normative note.
  - "Edge cases are identified" — the draft had four. Added three: rapid repeated play
    presses, permanent focus loss while screen-off, and system-killed service recovery.
- Iteration 2: all 24 items pass.
- Iteration 3: 23/24 pass. `/speckit.clarify` added FR-027 to FR-035 plus SC-009 to SC-013.
  One item regressed: "All functional requirements have clear acceptance criteria", because
  the new audio handoff requirements (FR-033, FR-034) have no acceptance scenario. The
  remaining 23 items were re-evaluated and are unaffected by the additions.
- Iteration 4: 16/16 pass. `/speckit.clarify` resolved five decisions — skip granularity
  (fixed 5-second increments, replacing the section-boundary draft), background service
  start (always-resident component), rate change (pitch-preserving), session-end state
  (Pause preserves, Stop discards), and the resume offer (3-second auto-start on
  deliberate pause only). This closed the iteration-3 checklist gap with two audio-handoff
  edge cases. Spec grew from 35 to 41 functional requirements and from 13 to 21 success
  criteria. No item regressed.
- Iteration 5: 16/16 pass. `/speckit.clarify` asked four further questions and resolved:
  the rate control (closed ladder 0.75x–4.0x, one press per step, FR-019), the reporting
  surface for a permanent focus loss while locked (scrolling "Retomando leitura de *&lt;título&gt;*"
  on the lock screen, expiring 3 seconds after the pause, FR-041), the ducking level
  (~20% of normal, FR-013), and the support floor (Android 10 / API 29). Spec grew from 46
  to 47 functional requirements and from 25 to 32 success criteria. No item regressed; the
  one item that could have — "Requirements are testable and unambiguous" — was tightened
  because FR-013 previously said only "reduce playback volume" with no level.
- Iteration 6: 16/16 pass. `/speckit.clarify` asked seven further questions and resolved
  them: focus answers other than an immediate grant (FR-013b names deferred and refused, and
  FR-016a names both device-removal mechanisms against the one-second bound), the declared
  sample rate (FR-033 fixes 22050 Hz, replacing a plan-level decision the Assumptions
  section had claimed no requirement depended on), the identifier convention for derived
  requirements (declared in the Functional Requirements preamble), the governed closed sets
  (FR-044, with focus regained and session-end reconciled), how a "playing" state is shown to
  have been audible (SC-005 became a position-advance rule over the record, and FR-039 now
  requires every transport change to carry its position), and the content of the diagnostic
  record (FR-045 forbids document text and closes the free-text field; SC-056 verifies it by
  scanner). Spec grew from 71 to 75 functional requirements and from 54 to 57 success
  criteria. No checkbox changed state; three justifications were rewritten because they had
  drifted from the file (the assumption count, the acceptance scenario count, and the
  edge-case count), and three were extended to record the deliberate exception created by
  naming a platform signal or a record field in requirement and criterion text. The one item
  that could have regressed — "All functional requirements have clear acceptance criteria" —
  was checked first and every new requirement does map to a criterion; declaring the
  identifier convention then exposed six older lettered requirements whose criteria tested
  them without naming them, and those citations were added.

### Open items for planning

- SC-006 requires an automated manifest check. The mechanism for that check belongs in
  `plan.md`, not here; the requirement is that the check exists and fails when the
  declaration is absent.
- FR-009 and FR-010 (Service bootstrap and Android 14+ foreground-service restrictions) are
  the highest-risk requirements in this feature. Neither can be verified by a host-based
  test, which is why gate 9 in the constitution exists. Planning should sequence an
  on-device verification early rather than last.
- FR-004 and SC-007 assume a position representation that survives engine changes. The
  interface between this feature and the engine feature must define that representation;
  that contract is produced by `/speckit.plan`.
- FR-009 and FR-010 now depend on an always-resident component (FR-009). The choice of what
  that component is, and how it is kept alive without becoming a battery drain, is a
  `plan.md` decision; the observable requirement is a lock-screen Play press that produces
  audio without an app open (SC-014).
- FR-032a through FR-032d distinguish Pause from Stop as separate acts with different storage
  consequences. The storage mechanism belongs in `plan.md`; the observable contract is that
  only Stop and reaching the document end clear the stored position (SC-017, SC-018).
- FR-032c permits audio to start without a fresh user action after a 3-second offer, which is
  a bounded, visible exception to FR-031. Whether the countdown pauses when the app leaves
  the foreground is covered by an edge case; the implementation choice is a `plan.md` matter.
- FR-015a requires pre-roll buffering to be bounded in memory, but no numeric bound is
  stated. The concrete limit belongs in `plan.md`; the requirement is that a bound exists
  and that position does not advance (SC-013).
- FR-041 introduces a scrolling lock-screen statement and a 3-second expiry for it, while
  FR-028 forbids elapsed time, remaining time, rate, and voice name on that same surface.
  Planning should confirm the two coexist — the scrolling statement carries only the document
  title, so it is not one of the four forbidden values, but the interaction is worth stating
  explicitly rather than discovering at implementation time (SC-029, SC-031).
- FR-019 now declares a closed rate ladder whose top rung, 4.0x, is twice the previous one
  (2.0x). The pitch-preservation requirement of FR-019a covers that step explicitly, and
  SC-015 tests it, but planning should confirm that 4.0x remains intelligible for the
  pt-BR voices selected later; if it does not, the rung is removed by amendment rather than
  quietly made unavailable.
- FR-045 requires the record's `detail` field to be either removed or bounded, and requires
  the choice to be declared. Decided in clarify 2026-10-02 (option A): retained and bounded —
  `maxLength: 200` with a key=value non-prose pattern — declared in
  `contracts/diagnostic-record.schema.json`, which now also requires `error` for `refusal`
  and `position_secs` on every entry, so the contract satisfies FR-045 and SC-056 can pass
  against that declaration.
- FR-033 fixes the declared rate at 22050 Hz, so planning owns the negotiation rather than
  the choice: request exactly that rate on the output stream, and treat a device refusal as
  FR-035's named failure rather than as a reason to fall back to the device's default
  configuration (SC-034 asserts the absence of that fallback).
- FR-016a requires each of the two device-removal mechanisms to be sufficient on its own.
  Planning should confirm which one fires first for a wired removal, a Bluetooth route loss,
  and the simultaneous unplug-and-connect case FR-016 already covers, so that the precedence
  rule is stated once rather than discovered per device.
- FR-044 names the closed sets and states that extension is a coordinated edit with a semver
  bump and a documentation update. `plan.md` owns the deterministic check SC-057 describes;
  the constitution's closed-set gate already covers the catch-all case in general, so the new
  requirement narrows that gate to the sets this feature enumerates rather than replacing it.
