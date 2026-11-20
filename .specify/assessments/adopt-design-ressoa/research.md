# Idea Research: Adotar o DESIGN-RESSOA

- **Slug**: adopt-design-ressoa
- **Created**: 2026-11-20 (backfilled)
- **Evidence confidence (overall)**: high for host census/contracts; medium for device legs (deferred)

## Users & Demand

- A driver needs a glanceable, value-blind notification and in-app surface with distinct earcon confirmations; visual identity must not add prose, rate, voice or brand to the driving surfaces. [source: `spec.md` US1–US4] (confidence: high)

## Prior Art

- `004-ressoa-dogfood-build` already implements the earcon grammar (300–2500 Hz closed band, ≤500 ms); 005 specifies the name→action mapping and distinctiveness, not new DSP parameters. [source: `spec.md` FR-003] (confidence: high)
- DESIGN-SONA adoption assessment and `DESIGN-RESSOA/` dir provide the token ledger and brand assets (Concha Harmônica R, Ressonador Canônico). [source: `spec.md` Input] (confidence: high)
- Surface-audit infrastructure (C-H1..C-H8) from 004 contracts is the deterministic host oracle. [source: `plan.md`, `tasks.md` T004–T011] (confidence: high)

## Market & Context

- Cost of inaction: inconsistent palette, unpinned brand placement and untested earcon distinctiveness weaken the driving-first identity and accessibility story.

## Data & Constraints

- Closed set of exactly 10 hex tokens: 8 `--ressoa-*` + 2 signal pins (`--signal-caution #FFD400`, `--signal-failure #FF4D63`); jade `#5AD6A8`/`#5FD0A0` and celeste `#B9A6F5` are reserved/unpainted (FR-001/FR-005, SC-001/SC-006).
- Notification carries only title/section/progress + one accent from the closed set; no body/rate/voice/brand/logo/waves (FR-002/FR-005/FR-007, SC-003).
- Earcons synthesized in Rust at 22050 Hz mono f32, 7 confirmations / 6 names; additive short duck-exempt channel; transport earcons dropped-and-re-fired on pause, system earcons immediate; ≤100 ms trigger-to-first-sample (FR-003/FR-009/FR-010, SC-002/SC-005).
- Double signature: Concha R header in-app only; Ressonador Canônico only in host CLI stderr + repo artifacts, never on driving surfaces (FR-004/SC-006).
- Operable with sound off/mute/DND — earcon is redundancy, never a barrier (FR-006/SC-004).
- Single-column geometry preserved; no `display: grid` second column (FR-008); surface count = 2.
- SC-007 TalkBack: controls announce labels, rate value via visually hidden live region (never painted), progress state, reason/failure lines; reduced-motion freezes the pill.

## Evidence Against the Idea

- Chromatic-only restyle plus earcon mapping may look incomplete versus the full proposal; mitigated by explicit Option A scope and spec 006 covering the complete visual separately. [inferred] (confidence: high)
- Device legs deferred → host evidence must not be presented as device PASS; mitigated by [NV] markers (SC-001/003).

## Gaps & Open Questions

- None blocking. Device trials owned by `device-build`/gate 9; host census is green at backfill.

## Sources

- `specs/005-adopt-design-ressoa/{spec,plan,tasks,research,data-model,verification}.md`, `contracts/`, `checklists/`
- `DESIGN-RESSOA/` tokens/assets, `specs/004-ressoa-dogfood-build/contracts/earcon-mapping.md`
