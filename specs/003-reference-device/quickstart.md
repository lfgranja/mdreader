# Quickstart: Reference Device Realignment

**Feature**: `003-reference-device` | **Date**: 2026-10-11

Validation guide for the amendment. All checks run on host; no device, no build, no `verify.sh` run required
(docs edits cannot affect gates 1–9; `/speckit.tasks` will still carry a gate-sanity item).

## Prerequisites

- Repo at the implementing commit (amendment landed).
- `002/spec.md:115-117` (reconciled SC-009 + amendment precedent) readable for comparison.

## Scenario 1 — One reference device (SC-001, US2)

1. `grep -rn "2201117TY" specs/001-android-media-shell/plan.md specs/002-fr-024-synthesis-adapter/spec.md`
2. **Expected**: hits in both files, same phone characteristics (Snapdragon 680, 4× A73 + 4× A53, ~5.8 GB, API 33).
3. `grep -n "A78" specs/001-android-media-shell/plan.md`
4. **Expected**: no reference-device meaning remains (SC-003).

## Scenario 2 — Reconciled bound (FR-002, SC-002 input)

1. `grep -n "10 s" specs/001-android-media-shell/plan.md`
2. **Expected**: Testing latency site and Performance Goals site both read ≤10 s cold-start; device-independent bounds (≤1 s headphone-pull, ±5 s skip, ≤2 s rate-change) unchanged.
3. Confirm the 9 s × 5 measurement has an explicit un-asterisked re-judgment statement (recorded at implementation time, old rows untouched).

## Scenario 3 — Divergence closed (FR-004, US2/AC1)

1. Open `specs/002-fr-024-synthesis-adapter/spec.md` around line 117.
2. **Expected**: original "Divergence noted" sentence intact, with a dated follow-up note below naming the 001 amendment commit.

## Scenario 4 — Amendment discipline (FR-003)

1. `git log --oneline -3` and `git show --stat <amendment-commit>`
2. **Expected**: commit message carries maintainer acceptance, date, rationale, preserved-history pointer; diff touches only the contracted sites (plus this feature's own artifacts).
