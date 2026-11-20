# Idea Intake: Adotar o DESIGN-RESSOA

- **Slug**: adopt-design-ressoa
- **Created**: 2026-11-20 (backfilled from `specs/005-adopt-design-ressoa/`)
- **Source**: `specs/005-adopt-design-ressoa/spec.md` (Created 2026-10-22) + assessment handoff (`go` for Option A scope)
- **Type**: feature (design-system adoption, host-verifiable slice)

## Idea (as captured)

> "Adotar o DESIGN SYSTEM definido em @DESIGN-RESSOA/ para o app." (assessment handoff: `go` para o escopo da Option A — tokens, marca dupla e earcons Ressoa nas superfícies existentes, notificação-first, closed set of 10 hex tokens, sem UI Svelte completa) — `spec.md:7`

## Restated

Apply the RESSOA design system to the two existing surfaces (MediaStyle notification + single in-app StatusView): a closed 10-hex token set, double brand signature (Concha Harmônica R in-app, Ressonador Canônico in host diagnostics only), mapped Ressoa earcons, notification purity, reduced-motion respect and a reduced Lente Sinestésica — without introducing a new UI framework or extra surfaces.

## Origin & Context

- **Raised by**: maintainer, 2026-10-22; handoff from the design-system assessment.
- **Trigger**: app shipped with DSN appearance but no coherent RESSOA token/brand/earcon identity; design dir `DESIGN-RESSOA/` defined the target.
- **Clarifications captured (2026-10-23/24)**: chromatic restyle only; `--ressoa-voice` is the single fill token for both progress branches; reserved celeste/jade never painted; 7 confirmation types across 6 acoustic names; no numeric rate on any surface; device legs deferred (owner `device-build`, gate 9).

## First-Glance Unknowns

- [NEEDS CLARIFICATION: whether "2-field grid" implies a two-column CSS layout — resolved in research: descriptive alias only, single-column geometry preserved (FR-008)]
- [NEEDS CLARIFICATION: TalkBack rate announcement vs value-blind — resolved: live `role="status"` announces value, never painted (SC-007)]
