# Problem Definition: Real Ressoa mobile app on DESIGN-SONA

- **Slug**: ressoa-mobile-app
- **Created**: 2026-10-12T12:49Z
- **Inputs used**: intake.md, research.md

## Problem Statement

A reader of long technical documents who is driving (or otherwise unable to look at a screen) cannot today listen to those documents on a phone with position that survives, interruptions that stay safe, and a lock-screen surface that respects attention — the existing engine/shell work is host-verified mechanism and the DESIGN-SONA surface is paper, so there is no distributable app that closes this loop.

## Affected Users & Stakeholders

- **Users**: the maintainer as dogfooding driver-reader — loses reading position and safe continuity while commuting; this is the only evidenced user. — [grounded in research: Users & Demand]
- **Users**: readers of long technical content (developers, engineering students) consuming while driving/working — affected the same way when they cannot look at the screen. — [grounded in research: ICP primario]
- **Users**: people with low vision or audio preference — affected where eyes-free operation and non-visual confirmation matter. — [grounded in research: ICP secundario]
- **Users**: [NEEDS CLARIFICATION: first non-author reader/driver — no external user evidenced; identity and needs unknown]
- **Stakeholders**: the maintainer (sole user, decider, funder) — decides scope, timing, and distribution commitments. — [grounded in research: single-maintainer governance]
- **Stakeholders**: [NEEDS CLARIFICATION: distribution gatekeepers and support burden — e.g. F-Droid reviewers / future readers — interest and decision power unknown until "real" is defined]

## Goals

- A driver can follow a long document across a whole commute without looking at the phone and without losing their place.
- Interruptions from the road and the phone (navigation, calls, alarms, headphone changes) resolve into states the driver can grasp without visual confirmation.
- The glanceable surface never exposes document prose or demands reading while driving.
- A stored position stays meaningful when the file is moved or renamed, and never offers a resume that points nowhere.
- There exists a distributable artifact whose behavior on the reference phone matches what was verified on the host. [NEEDS CLARIFICATION: exact distribution form — signed release, F-Droid listing, or device-installable build]

## Non-Goals

- Car-head-unit integration (Android Auto has no applicable category — recorded deferral, not oversight).
- A light-mode appearance (only the dark surface is specified; light needs full re-verification).
- An in-app diagnostic screen or reader-initiated diagnostic export (developer-tooling-only record in this scope).
- Monetization, pricing tiers, or upgrade gating ($0 now; freemium only on real demand).
- Non-Android platforms ([NEEDS CLARIFICATION: iOS/other scope unconfirmed — treated as out until stated]).
- Cloud-first operation (offline audibility is the standing requirement).

## Success Metrics

- Playback continues with the screen off and device locked for the duration of a commute-length session (baseline: host-verified only; on-device 10/10 trials unattested).
- First audible audio arrives within the agreed bound after Play on the reference phone (baseline: bound agreed on paper; no measured on-device value).
- Every driving-time control is reachable and confirmable without unlocking or visual confirmation (baseline: qualitative — blind-test distinctiveness unattested).
- Lock-screen surface shows only title, current section, and progress — never body text, elapsed/remaining, rate, or voice (baseline: specified; OEM behavior across Pixel/Samsung/Xiaomi unattested).
- Reopening a moved or renamed document resumes at the same content position; an edited document never offers a stale resume (baseline: specified behavior; cross-engine re-anchor measured only against stand-ins).
- Interruption outcomes (duck-and-continue vs. pause-with-reason vs. never-auto-resume) match the classified causes on device (baseline: host-verified mechanism; on-device focus/routing trials unattested).
- The on-device diagnostic record is retained per its window and cap and contains no document prose (baseline: host-verified shape; device retention unattested).

## Cost of Inaction

If this is never built, the playback shell stays a host-verified mechanism with no audible output on real hardware, DESIGN-SONA stays an unimplemented paper system, and reading stays a CLI/host activity — the driving reader keeps losing position, keeps facing invasive or silent audio behavior from existing tools, and no distributable artifact ever tests whether anyone beyond the maintainer wants this.

## Open Questions

- [NEEDS CLARIFICATION: What does "Real" concretely mean — signed release APK/AAB, F-Droid listing (own repo vs. public), versioned release with changelog, or only a device-installable debug artifact?]
- [NEEDS CLARIFICATION: Are the open 001 Phase 13 gaps plus real-audio proof (SC-044) prerequisites for "real," or is a silent-but-faithful DESIGN-SONA shell acceptable as v1?]
- [NEEDS CLARIFICATION: Which DESIGN-SONA surfaces are in v1 scope — notification only, or also StatusView, rate control, earcon grammar, envelope strip, gestures?]
- [NEEDS CLARIFICATION: Measured first-audio latency and earcon blind-test distinctiveness on the reference device — what are the numbers?]
- [NEEDS CLARIFICATION: Who is the first non-author user and what do they need beyond the maintainer's dogfooding cases?]
- [NEEDS CLARIFICATION: Naming/distribution readiness — `ressona.dev` state, F-Droid metadata/screenshots state, package label vs. technical name?]
- [NEEDS CLARIFICATION: Target platform scope — Android only, or also iOS/other?]
- [NEEDS CLARIFICATION: What prompted this now — what makes a distributable app the right next step versus closing the deferred synthesis and on-device trials first?]
