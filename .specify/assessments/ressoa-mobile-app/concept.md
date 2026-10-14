# Concept: Real Ressoa mobile app on DESIGN-SONA

- **Slug**: ressoa-mobile-app
- **Created**: 2026-10-12T12:50Z
- **Recommended option**: Option A — Dogfoodable single-voice sideload build

## Options

### Option A — Dogfoodable single-voice sideload build
- **Sketch**: The maintainer installs a signed sideload build on the reference phone, opens a long document, and hears it read aloud with the screen off. The lock-screen surface shows only the document title, current section, and progress in the DESIGN-SONA appearance; interruptions from navigation, calls, and headphone changes resolve into states graspable without looking; reopening the document offers to continue where it left off. Nothing is published anywhere — the artifact exists to close the loop for one driver and to produce the missing on-device measurements.
- **Appetite**: medium (weeks) — budget, not estimate; uncertainty marked plainly below.
- **Trade-offs**: Wins the fastest closed loop from paper to audible on real hardware, and turns every unattested success metric (first-audio bound, eyes-free confirmation, OEM behavior, position survival) into a measured number. Sacrifices distribution (no store listing, no new users), full DESIGN-SONA breadth (notification-first, remaining surfaces deferred), and engine failover (one offline voice only). Key risk: on-device measurements could fail (latency, intelligibility, OEM relayout) and send the work back into the deferred proofs — which is the point of doing it before any release.
- **Rabbit holes**: chasing full DESIGN-SONA breadth (envelope strip, gesture extensions, full earcon grammar) before the notification-first loop is measured; expanding from one voice to failover Cloud/Edge consent and credential handling; turning sideload signing into release plumbing (changelog, versioning, support channel).

### Option B — Full distributable Ressoa v1
- **Sketch**: A public-facing Ressoa release: the complete DESIGN-SONA surface (notification plus in-app reading view, rate control, and confirmation sounds), real-voice failover when an engine fails, green on-device trials across the required phone set, and release-ready distribution metadata. A stranger could discover, install, and rely on it for a commute.
- **Appetite**: large (months) — budget, not estimate.
- **Trade-offs**: Wins an actual product and the first chance at external validation. Sacrifices speed and focus: it commits to distribution, support, naming/availability, and failover breadth before any on-device number exists, and it takes on the documented platform risks (OEM relayout, compact-slot limits, Live Update rules) as release blockers rather than measured constraints. Key risk: shipping breadth nobody asked for — there is no evidenced external user — while the core audible loop is still unmeasured.
- **Rabbit holes**: store/F-Droid submission mechanics and review cycles; second-engine provisioning, consent surfaces, and credential handling; cross-OEM visual conformance; release support (updates, migrations, failure reports from strangers).

### Option C — Hold the app, close the deferred proofs first
- **Sketch**: Build nothing distributable now. Keep dogfooding through host/CLI paths while the deferred evidence is closed in place: the open shell gaps, the real-audio proof, and the on-device trial set. Revisit any app artifact only when first-audio latency, eyes-free distinctiveness, and position survival are measured numbers rather than specified bounds. (The "buy instead of build" form of this option: keep using built-in TTS / screen readers / produced audio for actual commuting needs in the meantime.)
- **Appetite**: small (days) of new app scope — the deferred-proof work itself is separately budgeted elsewhere; this option adds no app artifact.
- **Trade-offs**: Wins discipline and avoids every premature commitment (signing, listing, support, brand exposure). Sacrifices momentum and learning: without any device artifact, the deferred proofs stay host-bound, DESIGN-SONA stays untested against real constraints, and the question "does anyone beyond the maintainer want this" stays permanently unanswered. Key risk: indefinite deferral — the loop never closes because the artifact that would produce the measurements is the thing being withheld.
- **Rabbit holes**: perfecting host-side mechanism while on-device reality drifts; re-litigating specified bounds instead of measuring them; letting "not yet" become the standing answer without a re-entry trigger.

## Recommendation

Recommend **Option A**. It is the only option that serves the problem's goals in order: it makes the driving loop audible for the sole evidenced user (the maintainer), converts every unattested success metric into a measurement on the reference phone, and respects the recorded non-goals and the standing "do not launch yet" strategy by withholding distribution entirely. Option B clears no bar that Option A does not clear first — it buys breadth and support burden with no external demander. Option C is the safe second choice but fails the cost-of-inaction test: the measurements it wants are most cheaply produced by the sideload artifact it refuses to build. Appetite honesty: Option A's "weeks" budget assumes the deferred audio and shell-gap work lands inside this effort; if on-device numbers fail, the follow-on is bounded rework of those proofs, not scope expansion — that boundary is what keeps A small-relative-to-B rather than a stalking horse for B.

## Out of Scope (for the recommended option)

- Any public distribution (F-Droid submission, store listing, landing/changelog/support channel).
- Engine failover breadth (second engine, cloud consent/credential handling, unattended fallback).
- DESIGN-SONA surfaces beyond the notification-first readable loop (in-app reading view breadth, envelope strip, gesture extensions, full earcon grammar — notification-critical confirmations only).
- Car-head-unit integration, light mode, in-app diagnostics, monetization/paywalls, non-Android platforms, cloud-first operation (inherited non-goals, unchanged).

## Assumptions to Validate

- A single offline voice is an acceptable v1 listening experience for the maintainer's commute (intelligibility under cabin noise to be measured, not assumed).
- Sideload installation on the reference phone is an acceptable distribution form for a maintainer-only artifact (no reviewer/gatekeeper involved).
- The reference-phone measurements (first-audio bound, eyes-free confirmation, OEM behavior, position survival) will actually gate any later distribution decision — poor numbers stop the line rather than renegotiating it.
- The public/technical naming boundary (Ressoa externally, mdreader technically) holds for the artifact's visible strings.
- No external user or launch commitment arrives mid-effort that would force Option B scope onto Option A.
