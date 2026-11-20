# Idea Research: Reference Device Realignment

- **Slug**: reference-device
- **Created**: 2026-11-20 (backfilled)
- **Evidence confidence (overall)**: high (text-level, deterministic)

## Users & Demand

- Every 10/10-trial criterion in `001` is normed to a device class no one on the project owns; verdicts are either unmeasurable or dismissed by the "slower device failing is not evidence" rule, making gates decorative. [source: `spec.md` US1] (confidence: high)

## Prior Art

- `002/spec.md:115` was already amended on 2026-10-11 (commit `d5d6b45`) to name the Redmi and set ≤10 s cold-start; it is the template for the 001 edit. [source: local repo] (confidence: high)
- Amendment discipline: maintainer acceptance + date + rationale + preserved history, never rewriting old measurements. [source: `spec.md` FR-003] (confidence: high)

## Market & Context

- Cost of inaction: repo simultaneously asserts A78 (001) and Redmi (002) — a second drifting authority that invalidates cross-spec comparisons.

## Data & Constraints

- Four contracted edit sites in `contracts/amendment-record.md`: Site A/B in `001/plan.md`, Site C in `002/spec.md`, Site D = commit message (T002 verifies they still match before editing).
- Device identity: Redmi 2201117TY — Snapdragon 680 `bengal`, 4× Cortex-A73 + 4× A53, ~5.8 GB RAM, Android 13 / API 33 (FR-001).
- Numeric reconciliation: both ≤3 s sites (Testing latency, Performance Goals) → ≤10 s cold-start; 002 SC-009 amended bound agrees; ≤1 s / ±5 s / ≤2 s device-independent bounds remain untouched (FR-002).
- History preservation: the 9 s × 5 cold-start FAIL-under-≤3 s record stays verbatim in `002/SC-009-GATE-RECORD.md`; re-judgment stated explicitly (FR-003, T006).
- Docs-only scope: no crate, gate script or constitution change; gates 1–8 behavior untouched (T012).

## Evidence Against the Idea

- Realigning to a slower real device could invalidate past PASS verdicts. Mitigation: re-judge only where the amended bound changes the verdict, with explicit notation, never by editing old rows. [source: `spec.md` Edge Cases] (confidence: high)

## Gaps & Open Questions

- None blocking. Device trials themselves remain [NV] — this feature only fixes the authority they are judged against.

## Sources

- `specs/003-reference-device/{spec,plan,tasks,research,data-model}.md`, `contracts/amendment-record.md`
- `specs/001-android-media-shell/plan.md`, `specs/002-fr-024-synthesis-adapter/spec.md` + `SC-009-GATE-RECORD.md`
