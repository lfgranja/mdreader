# Release (R7): 022 Gate 9 Release-Only

- **Date**: 2026-11-18
- **Version**: v0.1.0 (infrastructure-only change; no SemVer bump of the app artifact — workflow/ruleset config only)
- **Validation**: R4 PASS (see `.specify/validations/gate9-release-only/validate.md`)

## Summary

Makes the Android build a release-only gate. Host gates remain mandatory on PRs; Gate 9 runs on manual dispatch and `v*` tag pushes; a `v*` tag ruleset enforces both checks.

## Changes

- FR-G9-002/003: `.github/workflows/ci.yml` — `push.tags: ['v*']`; `android-build.if` scoped to dispatch or `refs/tags/v*`.
- FR-G9-004: remote ruleset `v* release gates` (id 27578490) requiring the Host and Android check contexts.
- FR-G9-006: no factory workflow imported.

## Changelog (IDs)

- Added: FR-G9-001..006, SC-G9-001..004.
- Changed: CI trigger policy for Gate 9 (governance decision 2026-11-18).
- Removed: Gate 9 on every non-draft PR / branch push.

## Rollback

1. Revert the `ci.yml` commit on `dev`.
2. `gh api -X DELETE repos/lfgranja/mdreader/rulesets/27578490`.
3. Re-run CI on a sample PR to confirm both jobs return (prior state).

## Residual risk / [NV]

- A real `v*` tag push was not performed; ruleset enforcement on tag creation is [NV] until a tag release is executed.
- PR/merge CI for this change: [NV] until observed green.
