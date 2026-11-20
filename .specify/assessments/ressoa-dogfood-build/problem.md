# Problem Definition: Ressoa Dogfood Build

- **Slug**: ressoa-dogfood-build
- **Created**: 2026-11-20 (backfilled)
- **Inputs used**: intake.md | research.md

## Problem Statement

Specs 001/002 are host-verified but there is no installable, offline build that produces the missing on-device measurements (TTFA, device-removal pause latency, replay seconds, blind confirmation rates). Without a maintainer-only dogfood artifact, the driving-first claims remain untested and no real-session evidence exists.

## Affected Users & Stakeholders

- **Users**: the maintainer on the reference phone (Redmi 2201117TY), screen off, commuting.
- **Stakeholders**: downstream design specs (005/006) and release records (v0.1.0-rc*) that consume the device evidence.

## Goals

- FR-001/FR-003: read aloud screen-off/locked; first speech ≤3 s from Play (measured on the reference phone).
- FR-002/FR-011/FR-019: lock-screen and single in-app screen show only title/section/progress (+ in-app speed/transport), no prose/time/rate/voice.
- FR-004–FR-006: duck under navigation-transient; pause ≤1 s on silence-demanding/permanent loss and on device removal, with ≤8-word pt-BR reason for 3 s, never auto-resume.
- FR-007–FR-010: content-hash resume, last-spoken atomic cursor, 3 s pause offer, no resume after edit/stop/end.
- FR-012/FR-016: seven saturating rate steps + distinct earcon/haptic/visual confirmation per control.
- FR-015/FR-017/FR-018: offline only, private on-device diagnostics (30 d/10 MB), SAF file pick with hash snapshot.

## Non-Goals

- Publication, store listing, full release scope (explicitly excluded).
- Multi-voice catalog (single voice only), network voices, Android Auto.
- Any user-visible English string (identifiers are diag-only).

## Success Metrics

- SC-001: 30-min commute session, screen off, zero unlocks, valid at 4.0×.
- SC-002/SC-003/SC-004: 10/10 for cold-start ≤3 s, interruption outcomes, blind confirmations.
- SC-005/SC-009: 10/10 resume on moved/renamed; none on edited/stopped; 15-min airplane-mode session.
- SC-006/SC-010: 10/10 lock-screen and in-app surface purity; Stop clears position.
- SC-008: diagnostics retained within 30 d/10 MB, zero prose.

## Cost of Inaction

No device evidence; driving-safety and resume claims remain unverifiable, blocking credible release.

## Open Questions

- Device/gate-9 trials [NV] at backfill; T001–T173 all `[X]` (docs/host scope complete).
