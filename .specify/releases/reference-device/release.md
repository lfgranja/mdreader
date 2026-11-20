# Release (R7): 003 Reference Device Realignment

- **Date**: 2026-11-20 (artifact backfilled from `specs/003-reference-device/`)
- **SemVer**: v0.2.1 (patch — governance/docs-only amendment, no app binary change)
- **Validation**: R4 PASS for documentary checks (`.specify/validations/reference-device/validate.md`)

## Summary

Restores a single reference-device authority: `001/plan.md` now names the maintainer's real phone (Redmi 2201117TY) and reconciles both device-coupled bounds to ≤10 s cold-start, matching the already-amended `002/spec.md` (`d5d6b45`). The 002 divergence note is closed by an appended follow-up, not by rewriting history.

## Changelog (IDs)

- **Added**: FR-001 (001 plan Reference-device paragraph → Redmi 2201117TY, Snapdragon 680 `bengal`, 4× A73 + 4× A53, ~5.8 GB, API 33, for FR-027/FR-019c and all 10/10 criteria); FR-002 (both ≤3 s sites → ≤10 s cold-start; device-independent bounds untouched); FR-003 (amendment acceptance/date/rationale/preserved history); FR-004 (002 divergence closure note).
- **Added**: SC-001 (one-device grep), SC-002 (9 s × 5 re-judgment against ≤10 s, un-asterisked), SC-003 (no hypothetical class).
- **Changed**: `specs/001-android-media-shell/plan.md` Sites A/B; `specs/002-fr-024-synthesis-adapter/spec.md` Site C; Site D amendment commit message.
- **Preserved**: `002/SC-009-GATE-RECORD.md` measurement rows verbatim; the 002 "Divergence noted" sentence intact with the closure note appended below it.
- **Tasks**: T001..T012 (12/12 complete), including RED→GREEN doc checks T003/T007/T008 and docs-only scope check T012.
- **Bugs**: none; `d5d6b45` recorded divergence is the originating follow-up, now closed.

## Rollback

Revert the amendment commit(s) to restore the A78-class text and the pre-closure 002 note. No crate, gate script, dependency or constitution change — no binary or remote artifact to roll back. Because the amendment is append-only, reverting a closure note must not delete the original annotation.

## Residual risk / [NV]

- Device trials and host gate re-run [NV]; this feature only corrects the authority they are judged against.
- If the maintainer replaces the phone, a new realignment of the same shape is required (spec Edge Cases).
