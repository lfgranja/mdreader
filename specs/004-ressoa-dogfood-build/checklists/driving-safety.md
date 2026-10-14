# Driving Safety & Failure Handling Checklist: Ressoa Dogfood Build

**Purpose**: Validate that every driving-safety and failure-handling requirement is testable, complete, and consistent across the spec, contracts, and data model — enforcing Constitution Principle VIII (Driving-First, NON-NEGOTIABLE) and Principle VII (Closed Sets)
**Created**: 2026-10-13
**Feature**: [spec.md](../spec.md), [data-model.md](../data-model.md), [contracts/surface.md](../contracts/surface.md), [contracts/earcon.md](../contracts/earcon.md), [contracts/ingest.md](../contracts/ingest.md)
**Constitution**: [constitution.md](../../../.specify/memory/constitution.md) — Principle VIII (Driving-First), Principle VII (Closed Sets), Driving-Safety Requirements section

**Review Ownership**: This checklist is a reviewer-owned requirements-quality review artifact. Mark an item `[x]` only when the reviewer determines the driving-safety criterion is satisfied.
**Marker Semantics**: `[x]` means the criterion has been reviewed and satisfied. It does not mean implementation work is complete.

---

## 1. Screen-Off Continuity (FR-001, SC-001)

- [x] CHK001 - Does the spec require playback to continue with the screen off and device locked until the user pauses or the document ends? [Completeness, Spec §FR-001]
- [x] CHK002 - Is there a testable acceptance criterion for screen-off continuity (SC-001: 30-minute commute-length session, screen off and locked, zero unlocks)? [Measurability, Spec §SC-001]
- [x] CHK003 - Does User Story 1 have explicit acceptance scenarios for screen-off playback and lock-screen control functionality? [Coverage, Spec §US-1]
- [x] CHK004 - Is the screen-off requirement stated as a MUST (not a SHOULD or preference)? [Normativity, Spec §FR-001]

## 2. Lock-Screen Control Coverage (FR-002, FR-011, FR-012)

- [x] CHK005 - Are all driving-time transport controls (play, pause, skip-forward/back) required on the lock-screen surface? [Completeness, Spec §FR-002]
- [x] CHK006 - Is the lock-screen rate-step action required (reachable without unlock, no value display)? [Completeness, Spec §FR-012]
- [x] CHK007 - Is the Stop control explicitly forbidden on the lock-screen surface (in-app only)? [Completeness, Spec §FR-002, contracts/surface.md]
- [x] CHK008 - Does the spec define skip behavior (section boundary with ±10 s fallback for zero-section documents)? [Clarity, Spec §FR-002]
- [x] CHK009 - Are lock-screen content fields bounded to at most title + section + progress? [Completeness, Spec §FR-011]
- [x] CHK010 - Are prohibited lock-screen fields explicitly enumerated (prose, elapsed/remaining time, rate, voice identity)? [Completeness, Spec §FR-011, contracts/surface.md]
- [x] CHK011 - Is the 7-rung speed ladder specified with default 1.0x and saturation at ends (never wrap or fail)? [Clarity, Spec §FR-012]
- [x] CHK012 - Is pitch preservation required at every speed step? [Completeness, Spec §FR-012, SC-007]

## 3. No-Look Confirmation (FR-016, SC-004)

- [x] CHK013 - Does the spec require every driving-time control to confirm without looking via earcon + state change + haptic? [Completeness, Spec §FR-016]
- [x] CHK014 - Are all 6 confirmation types enumerated and required (play, pause, skip, rate-step, warning, failure)? [Completeness, Spec §FR-016, contracts/earcon.md]
- [x] CHK015 - Is there a testable acceptance criterion for no-look confirmation (SC-004: 10/10 correct per confirmation type)? [Measurability, Spec §SC-004]
- [x] CHK016 - Is the interface required to remain fully operable when confirmation sounds are disabled or DND/muted? [Completeness, Spec §FR-016, contracts/earcon.md]
- [x] CHK017 - Are earcon properties specified (≤500 ms, 300–2500 Hz, distinct timbre + rhythm + contour)? [Clarity, contracts/earcon.md]
- [x] CHK018 - Is the haptic path specified to ride outside STREAM_MUSIC (works muted/DND)? [Clarity, contracts/earcon.md]
- [x] CHK019 - Is the duck-exempt short-mix bypass specified so nav-duck never quiets the confirmation? [Clarity, contracts/earcon.md]

## 4. Interruption Safety (FR-004, FR-005, FR-006, SC-003)

- [x] CHK020 - Does the spec require ducking to ~20% gain for navigation-style transient interruptions (AUDIOFOCUS_LOSS_TRANSIENT_MAY_DUCK)? [Completeness, Spec §FR-004]
- [x] CHK021 - Is full-volume restore after duckable transient required without losing the listener's place? [Completeness, Spec §FR-004]
- [x] CHK022 - Does the spec require pause within 1 s for silence-demanding interruptions (call, alarm)? [Completeness, Spec §FR-005]
- [x] CHK023 - Does the spec require pause within 1 s for permanent audio loss? [Completeness, Spec §FR-005]
- [x] CHK024 - Does the spec require pause within 1 s for device removal (headphones unplugged / wireless route lost)? [Completeness, Spec §FR-006]
- [x] CHK025 - Is the 3-second reason lifetime on the lock-screen surface required for pause-demanding interruptions? [Completeness, Spec §FR-005, contracts/surface.md]
- [x] CHK026 - Is "never auto-resume" required for all pause-demanding outcomes? [Completeness, Spec §FR-005]
- [x] CHK027 - Is there a testable acceptance criterion for interruption safety (SC-003: 10/10 per interruption class)? [Measurability, Spec §SC-003]
- [x] CHK028 - Are all interruption classes covered by acceptance scenarios (navigation transient, call/alarm, headphone removal, permanent loss)? [Coverage, Spec §US-2]
- [x] CHK029 - Is the latest-wins rule specified for overlapping interruptions (transient followed by permanent, and permanent followed by transient)? [Clarity, Spec §Edge Cases]
- [x] CHK030 - Is the duck-gain restore after MAY_DUCK specified as automatic (not requiring explicit Play)? [Clarity, Spec §FR-005, data-model.md]
- [x] CHK031 - Is device-reconnect specified to only latch the route, never change transport? [Clarity, Spec §FR-005]

## 5. Resume Safety (FR-007, FR-009, SC-005)

- [x] CHK032 - Does the spec require resume at or just before the point of interruption (containing-sentence-start snap + 5–10 s cue)? [Completeness, Spec §FR-007]
- [x] CHK033 - Are the replay caps specified (≤30 s interruption, ≤60 s kill/power-loss)? [Clarity, Spec §FR-007]
- [x] CHK034 - Is "never mid-word" required for resume position? [Completeness, Spec §FR-007]
- [x] CHK035 - Is the 3-second visual dialog required for deliberate-pause reopen (continue vs. start-over, silence = continue)? [Completeness, Spec §FR-009]
- [x] CHK036 - Is the lock-screen resume control required after kill/power-loss (no visual dialog)? [Completeness, Spec §FR-009, contracts/surface.md]
- [x] CHK037 - Is there a testable acceptance criterion for resume safety (SC-005: 10/10 for move/rename, content-change, explicit-stop)? [Measurability, Spec §SC-005]
- [x] CHK038 - Is the distinction between deliberate-pause (3 s dialog) and kill/power-loss (lock-screen resume) explicitly defined? [Clarity, Spec §FR-009, data-model.md]
- [x] CHK039 - Is the persisted cursor required to be the last-spoken position (never buffered-ahead)? [Clarity, Spec §FR-007, data-model.md]
- [x] CHK040 - Is the durable write cadence specified (every 20 s fixed within 15–30 s envelope + on every interruption/lifecycle transition)? [Clarity, Spec §FR-007]

## 6. Lock-Screen Purity (FR-011, SC-006)

- [x] CHK041 - Does the spec require the lock-screen surface to show at most title + section + progress? [Completeness, Spec §FR-011]
- [x] CHK042 - Are all prohibited display fields explicitly enumerated (prose, elapsed/remaining time, rate, voice identity)? [Completeness, Spec §FR-011, contracts/surface.md]
- [x] CHK043 - Is there a testable acceptance criterion for lock-screen purity (SC-006: 10/10 inspections, zero leaks)? [Measurability, Spec §SC-006]
- [x] CHK044 - Is the display-vs-payload distinction specified (purity applies to display fields only; transport/reason/dismissal are state/behavior, not display leaks)? [Clarity, contracts/surface.md]
- [x] CHK045 - Is the rate-step affordance permitted without value display (value-blind on both surfaces)? [Clarity, Spec §FR-012, contracts/surface.md]
- [x] CHK046 - Are all user-visible lock-screen strings required to be pt-BR (document title exempt as proper noun)? [Completeness, Spec §FR-011]

## 7. Dismissal Safety (FR-014)

- [x] CHK047 - Does the spec require lock-screen dismissal to be refused while audio plays? [Completeness, Spec §FR-014]
- [x] CHK048 - Is pausing specified as the route to dismissal? [Completeness, Spec §FR-014, contracts/surface.md]
- [x] CHK049 - Is there a testable acceptance criterion for dismissal refusal (SC-006: 10/10 attempts refused while playing)? [Measurability, Spec §SC-006]
- [x] CHK050 - Is the platform lock-screen privacy setting obeyed (no workaround)? [Completeness, Spec §Edge Cases, contracts/surface.md]

## 8. Failure Closed-Set (FR-005)

- [x] CHK051 - Are all 8 failure reasons enumerated in the closed set (FocusDenied, FocusDelayed, PermanentLoss, TransientPaused, DeviceRemoved, VoiceUnavailable, FormatUndeliverable, DecodeFailed)? [Completeness, Spec §FR-005, data-model.md]
- [x] CHK052 - Is "no Other/Unknown" explicitly required (Principle VII compliance)? [Completeness, Spec §FR-005]
- [x] CHK053 - Does each failure reason map to a ≤8-word pt-BR lock-screen line (soft target) + ≤40-char hard limit? [Completeness, contracts/surface.md reason-line table]
- [x] CHK054 - Is the word count verified for each reason line (all ≤8 words including any appended title, with 40-char hard limit)? [Measurability, contracts/surface.md]
- [x] CHK055 - Does each failure reason map to a distinct earcon (warning vs. failure pair)? [Completeness, contracts/earcon.md]
- [x] CHK056 - Is a logged failure name required for each failure reason? [Completeness, Spec §FR-005]
- [x] CHK057 - Is the FocusDelayed/Deferred zero-audio hold specified (no reason line, no earcon)? [Clarity, contracts/surface.md, data-model.md]
- [x] CHK058 - Is the TransientPaused dual-outcome mapping specified (MAY_DUCK → duck, non-duckable/call/alarm → pause + reason)? [Clarity, Spec §FR-005, data-model.md]
- [x] CHK059 - Are undeliverable failures (VoiceUnavailable, FormatUndeliverable, DecodeFailed) specified as persistent (reason_ttl_ms=null)? [Clarity, contracts/surface.md]

## 9. Offline Guarantee (FR-015)

- [x] CHK060 - Does the spec require audible speech with no network connectivity? [Completeness, Spec §FR-015]
- [x] CHK061 - Is "zero network-dependent voice code" explicitly required for this build? [Completeness, Spec §FR-015]
- [x] CHK062 - Is the offline voice specified as a single CC0 Piper voice (faber, cadu, or jeff — fixed at plan time)? [Clarity, Spec §Assumptions]
- [x] CHK063 - Is any future network voice explicitly deferred as a later opt-in (never the sole path)? [Clarity, Spec §FR-015]

## 10. Diagnostic Privacy (FR-017, SC-008)

- [x] CHK064 - Does the spec require a private on-device diagnostic record? [Completeness, Spec §FR-017]
- [x] CHK065 - Is "never transmitted anywhere" explicitly required? [Completeness, Spec §FR-017]
- [x] CHK066 - Is "no document prose in any entry" explicitly required? [Completeness, Spec §FR-017]
- [x] CHK067 - Is there a testable acceptance criterion for diagnostic privacy (SC-008: zero entries containing document prose)? [Measurability, Spec §SC-008]
- [x] CHK068 - Are the retention bounds specified (30-day window + 10 MB cap, oldest-first trim)? [Clarity, Spec §FR-017]
- [x] CHK069 - Is retrieval specified as maintainer-only via adb pull from app-private storage? [Clarity, Spec §FR-017]
- [x] CHK070 - Is "no in-app surface" for diagnostics explicitly required? [Completeness, Spec §FR-017]
- [x] CHK071 - Are the required diagnostic fields enumerated (session, transport, per-section progress, per-rate changes, buffer states, interruption class+outcome, failure names, timestamps, content-hash refs)? [Completeness, Spec §FR-017, data-model.md]
- [x] CHK072 - Is the detail field constrained (≤200 chars, key=value identifier-pattern or typed fields only)? [Clarity, data-model.md]

## 11. Constitution Compliance (Principle VIII + Driving-Safety Requirements)

- [x] CHK073 - Does the spec enforce "no required interaction may demand looking at the screen" for all driving-time operations? [Compliance, Constitution §VIII]
- [x] CHK074 - Are all driving-time controls (play, pause, resume, next/previous section, adjust rate) reachable from the lock screen without unlocking? ("change voice" exempt — single-voice build) [Compliance, Constitution §VIII, Spec §FR-002, FR-012]
- [x] CHK075 - Does the spec require playback to survive an interrupted network connection without stopping mid-sentence? [Compliance, Constitution §VIII, Spec §FR-015]
- [x] CHK076 - Is the audio-start bound specified (≤3 s from Play to first audible speech)? [Compliance, Constitution §Driving-Safety Requirements, Spec §FR-003, SC-002]
- [x] CHK077 - Does the spec obey the single-playback-authority rule (Rust owns playback, Kotlin MediaSession is a mirror)? [Compliance, Constitution §Tauri Platform Contract, contracts/surface.md]
- [x] CHK078 - Is the deliberate deviation from Android ducking best practice (ADR-001) recorded with rationale? [Compliance, Constitution §Tauri Platform Contract, Spec §FR-004]
- [x] CHK079 - Are closed sets enforced throughout (failure reasons, transport states, end kinds, speed ladder, earcon types)? [Compliance, Constitution §VII, data-model.md]

## 12. Cross-Artifact Consistency

- [x] CHK080 - Are failure reason names consistent across spec.md, data-model.md, and contracts/surface.md? [Consistency, Spec §FR-005, data-model.md, contracts/surface.md]
- [x] CHK081 - Are earcon types consistent across spec.md, contracts/earcon.md, and data-model.md? [Consistency, Spec §FR-016, contracts/earcon.md, data-model.md]
- [x] CHK082 - Are interruption outcomes consistent across spec.md, data-model.md, and contracts/surface.md? [Consistency, Spec §FR-004/FR-005/FR-006, data-model.md, contracts/surface.md]
- [x] CHK083 - Are lock-screen content fields consistent across spec.md, contracts/surface.md, and data-model.md? [Consistency, Spec §FR-011, contracts/surface.md, data-model.md]
- [x] CHK084 - Are resume behaviors consistent across spec.md, data-model.md, and contracts/surface.md? [Consistency, Spec §FR-007/FR-009, data-model.md, contracts/surface.md]
- [x] CHK085 - Are speed ladder values consistent across spec.md, data-model.md, and contracts/surface.md? [Consistency, Spec §FR-012, data-model.md, contracts/surface.md]
- [x] CHK086 - Are diagnostic record requirements consistent across spec.md, data-model.md, and SC-008? [Consistency, Spec §FR-017, data-model.md, Spec §SC-008]
- [x] CHK087 - Are timing bounds consistent across spec.md and SC-001 through SC-008? [Consistency, Spec §FR-003/FR-005/FR-006/FR-007, Spec §SC-001–SC-008]

---

## Notes

- **Pass count**: 87 / 87
- **Fail count**: 0 / 87
- **Total items**: 87
- **Constitution principle**: VIII (Driving-First, NON-NEGOTIABLE) — every item in this checklist traces to a MUST-level requirement in the spec
- **Verification method**: Each item was verified by cross-referencing the spec.md functional requirements (FR-001 through FR-019), success criteria (SC-001 through SC-008), contracts (surface.md, earcon.md, ingest.md), data-model.md, and the constitution's Driving-Safety Requirements section
- **Key cross-artifact findings**:
  - The 8-reason failure closed-set is consistently enumerated across spec.md (FR-005), data-model.md (Interruption table), and contracts/surface.md (reason-line table)
  - All 6 earcon confirmation types are consistently specified across spec.md (FR-016), contracts/earcon.md (grammar table), and data-model.md (Earcon section)
  - The TransientPaused dual-outcome mapping (MAY_DUCK → duck, non-duckable → pause + reason) is consistent across spec.md (FR-004/FR-005), data-model.md (Interruption table), and contracts/surface.md (reason-line table)
  - Lock-screen purity fields (title + section + progress only) are consistent across spec.md (FR-011), contracts/surface.md (Fields), and data-model.md (Progress section)
  - Resume behavior distinction (deliberate-pause 3 s dialog vs. kill/power-loss lock-screen resume) is consistent across spec.md (FR-009), data-model.md (Playback Session end_kind), and contracts/surface.md (Resume)
- **No gaps identified**: Every driving-safety and failure-handling requirement in the spec has a testable acceptance criterion and is traceable to a constitution principle
