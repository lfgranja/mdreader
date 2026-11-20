# Decision: Ressoa Dogfood Build

- **Slug**: ressoa-dogfood-build
- **Decided**: 2026-11-20 (backfilled; lineage `ressoa-mobile-app` verdict `go` for Option A)
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | Missing on-device measurements block credible driving-safety claims |
| Evidence strength | medium | Host/contract evidence strong; device trials [NV] named, not asserted |
| Value vs. inaction | strong | Installable artifact + real commute evidence |
| Feasibility / appetite | strong | Option A consumes existing host-verified 001/002/003; bounded scope |
| Strategic fit | strong | Single authority, offline-only, closed sets, pt-BR discipline match constitution |
| Risk posture | medium | Device trials and SAF grant behavior require device confirmation |

## Verdict & Rationale

`go` for Option A only. The dogfood slice delivers the missing measurements without publication risk; residual risk is device-only and explicitly marked [NV].

## If go — Handoff to `/speckit-specify`

- **Problem**: no installable offline build producing on-device evidence.
- **Chosen approach**: single-voice sideload consuming 001/002/003; closed-surface lock-screen + in-app; hash resume; earcon/haptic confirmations; private diagnostics.
- **In scope**: FR-001..019, SC-001..010, T001..T173 (all `[X]` at backfill).
- **Out of scope**: publication, multi-voice, network, consent surface, Android Auto.
- **Success metrics**: SC-001 30-min zero-unlock; SC-002/003/004 10/10; SC-005/009 resume; SC-006/010 surface purity; SC-008 diagnostics.
- **Carried-forward questions**: device/gate-9 trials [NV]; pt-BR reason word-count verification on device.
