# Feature Specification: Gate 9 Release-Only

**Feature Branch**: `feature/022-gate9-release-only`
**Created**: 2026-11-18
**Status**: Draft (assess verdict `go`)
**Input**: Approved governance decision in `.specify/memory/STATE.md` (2026-11-18) + `specs/` audit + live CI/API evidence.

## Problem / Evidence Summary

The Android build (Gate 9) currently runs on every non-draft pull request and every branch push to `dev`/`main` (`.github/workflows/ci.yml:116-121`). Measured on PR #222: Host gates took 9m47s, Android build 27m29s. Meanwhile `v0.1.0-*` tags exist and no GitHub ruleset protects them (`gh api .../rulesets` returned empty on 2026-11-18). The approved policy requires Host gates on PRs, Gate 9 only on release candidates/manual dispatch/`v*` tags, and a `v*` ruleset requiring both checks.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Ordinary PR stays fast (Priority: P1)

As a contributor, when I open a normal PR to `dev`, only the Host verification runs so I get feedback without paying the full Android build cost.

**Independent Test**: Open a non-draft PR to `dev`; observe the check list contains `Host gates (verify.sh 1-8 subset + supply chain)` and does not require `Android build (gate 9, aarch64 debug)`.

**Acceptance Scenarios**:
1. **Given** a non-draft PR to `dev`, **When** CI runs, **Then** only the Host check is required for merge.
2. **Given** a draft PR, **When** CI runs, **Then** no Android job is executed.

### User Story 2 — Release candidate builds Android on demand (Priority: P1)

As a release engineer, when I manually dispatch the workflow on a candidate commit, the Android build runs and produces a verifiable artifact.

**Independent Test**: Dispatch the CI workflow on a commit; observe both Host and Android jobs execute and the APK artifact is produced.

### User Story 3 — `v*` tags cannot be published unverified (Priority: P1)

As a maintainer, when a `v*` tag is pushed, both Host and Android checks must succeed before the tag can be created.

**Independent Test**: Inspect the live `v*` ruleset via `gh api`; observe required status checks are exactly the Host and Android check contexts. Push a `v*` tag and observe both jobs run.

## Functional Requirements

- **FR-G9-001**: The Host gate job MUST remain triggered on pushes and PRs to `dev`/`main`.
- **FR-G9-002**: The Android job MUST NOT run on ordinary PRs or branch pushes; it MUST run on `workflow_dispatch` and on pushed tags matching `v*`.
- **FR-G9-003**: The workflow MUST be triggered by pushes of tags matching `v*` so both checks exist for tag protection.
- **FR-G9-004**: A GitHub ruleset MUST target `v*` tags and require both the Host and Android check contexts (exact names from job `name:` fields).
- **FR-G9-005**: The change MUST be reversible: reverting the workflow and deleting the ruleset restores the prior state.
- **FR-G9-006**: No Forgejo/factory workflow artifact MUST be imported into the target.

## Key Entities

- **Host check context**: `Host gates (verify.sh 1-8 subset + supply chain)`.
- **Android check context**: `Android build (gate 9, aarch64 debug)`.
- **Release tag pattern**: `v*` (existing tags: `v0.1.0-alpha.1`, `v0.1.0-rc1`, `v0.1.0-rc2`).

## Edge Cases

- A PR that otherwise passes Host but is about Android-only code gets no early Android signal (accepted trade-off).
- A manually dispatched workflow on a non-`v*` ref still runs Gate 9 (intended: candidate commit verification).
- Check-context name drift (`name:` changed) would break the ruleset; validation compares exact strings.

## Success Criteria

- **SC-G9-001**: A normal PR to `dev` shows only the Host check as required (baseline: both).
- **SC-G9-002**: `workflow_dispatch` and a pushed `v*` tag both trigger Host + Android jobs (verified live).
- **SC-G9-003**: `gh api repos/lfgranja/mdreader/rulesets` shows a `v*` ruleset with both exact check contexts (baseline: empty).
- **SC-G9-004**: Rollback verified by inspection (workflow diff + ruleset deletion path documented).

## Assumptions

- The authenticated `gh` token can create rulesets; if not, mark `[NV]` and escalate to the Software Engineer.
- Check contexts are taken verbatim from `.github/workflows/ci.yml` `name:` fields.

## Out of Scope

- Signing, store publishing, tag renaming, changes to `dev`/`main` branch protection, importing Forgejo workflows.
