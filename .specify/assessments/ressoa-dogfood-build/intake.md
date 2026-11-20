# Idea Intake: Ressoa Dogfood Build

- **Slug**: ressoa-dogfood-build
- **Created**: 2026-11-20 (backfilled from `specs/004-ressoa-dogfood-build/`)
- **Source**: `specs/004-ressoa-dogfood-build/spec.md` (Created 2026-10-12) + assessment lineage `.specify/assessments/ressoa-mobile-app/`
- **Type**: new feature (maintainer-only dogfood release slice)

## Idea (as captured)

> "assessment verdict `go` for Option A only (dogfoodable single-voice sideload build): a maintainer-only installable app on the reference phone that reads long documents aloud with the screen off, shows only title + current section + progress on the lock-screen surface in the DESIGN-SONA appearance, resolves interruptions into states graspable without looking, resumes by content-derived position, and produces the missing on-device measurements. Nothing is published; full release scope is explicitly excluded." — `spec.md:7`

## Restated

Produce a sideloaded, offline, single-voice build the maintainer can use on their own phone for long commutes: eyes-free transport with earcon/haptic confirmation, closed-set interruption states, content-hash resume, determinate-when-possible progress, and on-device diagnostics — nothing published.

## Origin & Context

- **Raised by**: maintainer, 2026-10-12; inherits `go` for Option A from `ressoa-mobile-app` assessment.
- **Trigger**: specs 001/002 were host-verified but no installable build produced the missing device measurements (TTFA, removal latency, replay seconds).
- **Clarifications captured (2026-10-14)**: second-document pick → auto-save + switch, never auto-start; rate steps saturate at 0.75×/4.0×; only MAY_DUCK restores automatically — all other gains require explicit Play; all user-visible strings pt-BR.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: how to keep lock-screen minimal yet informative — resolved in research: FR-011 title/section/progress only, rate-step affordance without value]
- [NEEDS CLARIFICATION: replay/progress honesty on partial parse — resolved: FR-013 indeterminate for the whole session, never flips mid-session]
