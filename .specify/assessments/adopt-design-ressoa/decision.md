# Decision: Adotar o DESIGN-RESSOA

- **Slug**: adopt-design-ressoa
- **Decided**: 2026-11-20 (backfilled; accepted Option A handoff 2026-10-22)
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | Surface identity/purity/earcon mapping unauditable today |
| Evidence strength | strong | Closed token ledger, surface-audit C-H1..C-H8, 004 earcon grammar, pt-BR/closed-set contracts |
| Value vs. inaction | strong | Auditable driving-first identity + accessibility |
| Feasibility / appetite | strong | Option A reuses existing surfaces and 004 DSP; host-verifiable |
| Strategic fit | strong | Closed sets, single authority, no new surfaces match constitution |
| Risk posture | medium | Device legs DEFERRED (owner `device-build`), named not silently passed |

## Verdict & Rationale

`go` for Option A. Host evidence is deterministic; device deferral is explicit and owned.

## If go — Handoff to `/speckit-specify`

- **Problem**: RESSOA identity unpinned on the two driving surfaces.
- **Chosen approach**: closed 10-token chromatic restyle + double brand + earcon mapping + notification purity + reduced-motion/TalkBack.
- **In scope**: FR-001..010, SC-001..007, T001..T055 (after duplicate-ID cleanup), surface audits C-H1..C-H8.
- **Out of scope**: Svelte UI, new tokens, numeric rate, brand/waves on driving surfaces, device trials.
- **Success metrics**: SC-001 census 10 hex fail-closed; SC-002/SC-005 7 distinct confirmations + ≤100 ms; SC-003 zero leaks [NV]; SC-004/SC-006/SC-007 accessibility.
- **Carried-forward questions**: device legs owned by `device-build`/gate 9; complete visual in spec 006.
