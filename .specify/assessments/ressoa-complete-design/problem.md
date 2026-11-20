# Problem Definition: Ressoa Complete Design

- **Slug**: ressoa-complete-design
- **Created**: 2026-11-20
- **Inputs used**: intake.md | research.md

## Problem Statement

The shipped StatusView implements the 006 Lente Sinestésica but omits the proposal's defining visual elements — teal→violet wave ribbons, glassmorphism title card, reactive waveform, numeric rate, repeat control and glow/neon accents — because the pre-amendment constitution forbade them. Users see an incomplete design; the project has an approved proposal and an amended constitution that now permit the complete design.

## Affected Users & Stakeholders

- **Users**: Ressoa listeners in driving/peripheral-gaze contexts — expect the approved visual language and complete transport controls.
- **Stakeholders**: maintainer (surface/a11y invariants, token governance), design owner (proposal fidelity).

## Goals

- Apply all proposal elements on the single StatusView (FR-001, FR-009, FR-010, FR-011).
- Make the waveform genuinely reactive to RMS audio with bounded IPC/frame budgets (FR-008).
- Paint the numeric rate only in the speed-control area without breaking value-blind accessibility semantics (FR-003).
- Expand the token set with jade/celeste under §VII and keep notification accent-only (FR-005..FR-007).

## Non-Goals

- No second surface, no native/Kotlin playback logic, no new required interaction depending on supplementary visuals (§VIII).
- No network assets, scripts or `foreignObject`; no new continuous animation loops beyond S5 budget.
- No changes to rate ladder, focus outcomes or `EngineError`/`EventKind` closed sets.

## Success Metrics

- SC-001: every proposal element present; SC-002: zero accessibility regressions; SC-003: notification purity unchanged.
- SC-004: reduced-motion respected; SC-005: surface inventory valid, no dead corpses; SC-006: device trial matches proposal.
- Host census/gates 1–8 unchanged or extended, never weakened; 29/29 tasks attested.

## Cost of Inaction

The flagship design promise stays unfulfilled and the constitution amendment yields no user-visible benefit.

## Open Questions

- Device visual/interaction trial (SC-006) is [NV] in local deterministic validation; deferred to device cycles.
