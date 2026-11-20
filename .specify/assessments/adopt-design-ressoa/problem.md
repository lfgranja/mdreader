# Problem Definition: Adotar o DESIGN-RESSOA

- **Slug**: adopt-design-ressoa
- **Created**: 2026-11-20 (backfilled)
- **Inputs used**: intake.md | research.md

## Problem Statement

The app surfaces (MediaStyle notification + StatusView) lack a coherent RESSOA visual/sonic identity: token usage is unpinned, brand placement is undefined, earcon→action mapping is unspecified, and notification purity/accessibility are not enforced by deterministic checks. The driving-first design system therefore cannot be audited or trusted.

## Affected Users & Stakeholders

- **Users**: driver/listener relying on glanceable, value-blind surfaces and distinct earcons.
- **Stakeholders**: maintainer (host census owner), `device-build` (deferred device legs), spec 006 (consumes the reduced Lente + contracts).

## Goals

- FR-001/FR-005/FR-008: apply the closed 10-hex token set to both surfaces, chromatic only, single-column geometry preserved.
- FR-002/FR-007: notification shows title/section/progress only, no body/rate/voice/brand/logo/waves.
- FR-003/FR-009/FR-010: map 7 confirmations to 6 Ressoa earcon names, additive duck-exempt channel, ≤100 ms, mute/DND redundancy.
- FR-004: double brand signature — Concha R in-app, Ressonador Canônico in host diagnostics/repo only.
- FR-006: operable without sound.
- Reduced-motion + TalkBack support per SC-007.

## Non-Goals

- Full Svelte UI / new surfaces; new token members; numeric rate on any surface; brand or waves on notification/lock-screen.
- Device visual/blind trials in this feature (deferred to `device-build`/gate 9).

## Success Metrics

- SC-001: deterministic census of exactly 10 hex tokens, fail-closed; no new token.
- SC-002/SC-005: 7 distinct confirmations in blind protocol; ≤100 ms action→earcon.
- SC-003: 10/10 device inspections, zero leaks [NV].
- SC-004/SC-006/SC-007: mute operability, no off-set/vacancy paint, TalkBack announcements + reduced-motion freeze.
- Host gates 1–8 green; expected-tests census in sync.

## Cost of Inaction

Unverifiable design-system compliance; inconsistent driving surfaces and untested accessibility.

## Open Questions

- Device legs remain DEFERRED with named owner; no silent pass.
