# Decision: Real Ressoa mobile app on DESIGN-SONA

- **Slug**: ressoa-mobile-app
- **Decided**: 2026-10-12T12:51Z
- **Verdict**: go
- **Artifacts reviewed**: intake.md, research.md, problem.md, concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | adequate | Real unclosed driving loop for the evidenced maintainer-driver; no external user evidenced, so validity holds at dogfood scope, not release scope. |
| Evidence strength | adequate | Strong internal (host-green shell, specified design, recorded constraints) with explicitly-marked assumption gaps on demand/market and unmeasured on-device numbers — medium overall, never over-claimed. |
| Value vs. inaction | adequate | Closes the audible loop and produces the gating on-device measurements; inaction leaves a silent mechanism and paper design serving one user via CLI only. |
| Feasibility / appetite | adequate | Option A credible within a medium (weeks) budget on top of the built shell and in-progress adapter; on-device failure bouncing back to deferred proofs is the stated risk. |
| Strategic fit | strong | Driving-first, offline-first, closed sets, F-Droid-not-Play, do-not-launch-yet, and $0-now all align with the constitution and product-marketing record. |
| Risk posture | adequate | Major risks identified with documented mitigations (platform R1–R4, design R9–R12, Phase 13 gaps, SC-044 deferral); residual on-device unknowns are the measured risk of Option A, not hidden. |

## Verdict & Rationale

**Go — for Option A scope only (dogfoodable single-voice sideload build).** Problem validity and evidence strength both clear `adequate`, a recommended concept option exists, and strategic fit is strong; the scorecard supports specifying the narrow artifact that makes the driving loop audible for the sole evidenced user and turns every unattested metric into a reference-phone measurement, while withholding all distribution. A `go` for the full release (Option B) would over-claim thin external evidence and is explicitly not granted.

## If go — Handoff to `/speckit.specify`

- **Problem**: A driving reader cannot listen to long documents on a phone with surviving position, safe interruptions, and an attention-respecting lock-screen surface — the shell is host-only mechanism and DESIGN-SONA is paper.
- **Chosen approach**: Option A — Dogfoodable single-voice sideload build (maintainer-only install on the reference phone; notification-first DESIGN-SONA readable loop; one offline voice; nothing published).
- **In scope / out of scope**: In scope: audible commute-length loop with screen off, classified interruption states graspable without looking, title+section+progress lock-screen surface, content-keyed resume, reference-phone measurement of every success metric. Out of scope: any public distribution, engine failover breadth, DESIGN-SONA surfaces beyond the notification-first loop, car-head-unit, light mode, in-app diagnostics, monetization, non-Android platforms, cloud-first operation.
- **Success metrics**: Screen-off playback for a commute session; first audio within the agreed bound on the reference phone; eyes-free reachability/confirmation of driving controls; lock-screen showing only title/section/progress across the OEM set; move/rename-safe resume with no stale offers; classified interruption outcomes matching causes; diagnostic retention window/cap with zero document prose.
- **Carried-forward open questions**: exact sideload-vs-release meaning of "real" (resolved to sideload for this scope); whether Phase 13 gaps + real-audio proof land inside this effort (yes — bounded, no scope expansion on failure); notification-first vs. broader DESIGN-SONA surfaces (notification-first only); measured latency/distinctiveness numbers (to be produced, then gate); first non-author user (explicitly deferred); naming/distribution readiness (deferred with the distribution); platform scope (Android only); timing trigger (measurements now gate any later release).
