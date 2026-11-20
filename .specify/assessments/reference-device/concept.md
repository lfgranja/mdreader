# Concept: Reference Device Realignment

- **Slug**: reference-device
- **Created**: 2026-11-20 (backfilled)
- **Recommended option**: Option A

## Options

### Option A — Amend 001 to match the already-amended 002 (recommended)
- **Sketch**: Site A rewrite of the 001 Reference-device paragraph to the Redmi identity; Site B reconcile both ≤3 s sites to ≤10 s cold-start; Site D amendment commit with acceptance/date/rationale/history pointer; Site C follow-up closure note appended under the intact 002 divergence sentence. RED→GREEN doc checks per T003/T007/T008 (A78 hits → 0, 2201117TY → ≥1, both bounds read ≤10 s).
- **Appetite**: small (docs-only, 4 contracted sites).
- **Trade-offs**: Single authority restored, history preserved, fully grep-verifiable. No functional risk.

### Option B — Revert 002 to the A78 class
- **Sketch**: Undo `d5d6b45` so both specs say A78.
- **Appetite**: small.
- **Trade-offs**: Rewrites an accepted maintainer decision and keeps a device no one owns; rejected.

### Option C — Define a third "generic" reference class
- **Sketch**: Abstract both specs to a performance envelope rather than a device.
- **Appetite**: medium.
- **Trade-offs**: Reintroduces the hypothetical class problem and requires re-baselining every gate; rejected.

## Recommendation

Option A. It follows the already-accepted maintainer decision, touches only contracted sites, and is verifiable by grep alone.

## Out of Scope (recommended option)

- No code/gate changes; no rewriting of old measurement rows; no new device class.

## Assumptions to Validate

- All four contract sites still match current text before any edit (T002; STOP-and-update if drifted).
- The 002 amendment shape (`d5d6b45`) is the accepted precedent (verified: `spec.md` FR-003).
