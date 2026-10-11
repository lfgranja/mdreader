# Research: Reference Device Realignment

**Feature**: `003-reference-device` | **Date**: 2026-10-11

All Technical Context unknowns were already resolved by the spec (Q1:B, 2026-10-11) and by the
accepted 002 precedent (`d5d6b45`). No open NEEDS CLARIFICATION remains. This note records the
decisions so they are not re-litigated.

## R1: Reference device identity

- **Decision**: Redmi 2201117TY — Snapdragon 680 (`bengal`), 4× Cortex-A73 + 4× Cortex-A53, ~5.8 GB RAM, Android 13 / API 33.
- **Rationale**: Single-user project; the only gate device the maintainer possesses. Verdicts normed to a possessed phone are runnable; verdicts normed to the A78 class are not (spec US1).
- **Alternatives considered**: Keep the A78 class (rejected — nobody on the project possesses it; the "slower device failing is not evidence of failure" rule made gates decorative); name a hypothetical newer phone (rejected — same defect, plus no hardware to run trials on).
- **Source**: `spec.md` FR-001; `002/spec.md:117` amendment (same phone, accepted precedent, not under re-review).

## R2: Reconciled cold-start bound

- **Decision**: ≤10 s cold-start press→first-audible at transducer on the reference phone, replacing the ≤3 s bound at the two device-coupled sites in `001/plan.md` (Testing 10/10-trial latency; Performance Goals "Audible audio after Play").
- **Rationale**: Maintainer choice Q1:B (2026-10-11); consistent with amended 002 SC-009 (`002/spec.md:115`); measured 9 s × 5 feels fine in real use on this phone. Pre-roll becomes a future improvement, not a gate obligation.
- **Alternatives considered**: Keep ≤3 s (rejected by maintainer — would FAIL the possessed phone and mandate adaptation work for a single-user project); relax further / drop the bound (rejected — a bounded start is the tunnel-usability guarantee, Driving-Safety-adjacent).
- **Source**: `spec.md` FR-002 + Assumptions (device-independent bounds ≤1 s / ±5 s / ≤2 s explicitly out of scope).

## R3: Amendment discipline (how, not what)

- **Decision**: Same shape as the 002 amendment `d5d6b45`: maintainer acceptance + date + rationale recorded in the implementing commit; prior text superseded by note, never rewritten; past measurements preserved, re-judged only by explicit statement.
- **Rationale**: 002's discipline is the accepted template (`spec.md:47`); consistency keeps the audit trail uniform.
- **Alternatives considered**: In-place rewrite of old verdicts (rejected — destroys history; forbidden by spec Edge Cases).
- **Source**: `spec.md` FR-003, FR-004, Edge Cases.

## R4: Divergence closure site

- **Decision**: Follow-up note appended at/near `002/spec.md:117` stating the condition (001 still says A78) no longer holds, with pointer to the 001 amendment commit. Original annotation left intact above the note.
- **Rationale**: FR-004 requires closure "by a follow-up note (not rewritten history)".
- **Alternatives considered**: Editing the 002 annotation in place (rejected — same history-rewrite defect as R3).
- **Source**: `spec.md` FR-004, US2 acceptance scenario.

## Open items

None. Ready for Phase 1 design.
