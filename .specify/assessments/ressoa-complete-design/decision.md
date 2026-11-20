# Decision: Ressoa Complete Design

- **Slug**: ressoa-complete-design
- **Decided**: 2026-11-20
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | 006 explicitly excluded the proposal elements; user asked for the complete design; amendment permits it (spec.md, research.md). |
| Evidence strength | strong | FR-001..011/SC-001..006 with file/line detail, plan constraints, 29/29 tasks and ROADMAP CONVERGED attestation. |
| Value vs. inaction | strong | Delivers the flagship visual contract without architectural change. |
| Feasibility / appetite | strong | Option A is bounded to one surface, RMS/IPC/canvas plus token/census updates; already implemented and attested. |
| Strategic fit | strong | Rust authority, single surface, offline-only and §VII/§VIII amendments all preserved. |
| Risk posture | adequate | A11y/token/census regressions mitigated by structural host checks; device visual trial remains [NV]. |

## Verdict & Rationale

`go` for Option A. Evidence is measured, scope is bounded, and residual risk is covered by deterministic host checks. This assessment documents the already-sanctioned design completion; it does not authorize new surfaces, engine changes or required interactions tied to decoration.

## If go — Handoff to `/speckit-specify`

- **Problem**: The StatusView omits the approved proposal's defining visuals under the old constitution.
- **Chosen approach**: Option A — full proposal on the single surface, RMS-reactive canvas with bounded IPC/frame budget, expanded tokens under §VII.
- **In scope**: FR-001..FR-011; jade/celeste tokens; canvas wave + repeat + numeric rate + glow; contracts/inventory/census updates; T001–T029 verification.
- **Out of scope**: new surfaces, Kotlin playback logic, engine/focus/rate-ladder changes, network assets, required decorative interactions.
- **Success metrics**: SC-001..006; host gates 1–8 green; 29/29 tasks; device screenshot parity (device cycles).
- **Carried-forward questions**: device visual trial (SC-006) — non-blocking for local deterministic validation.
