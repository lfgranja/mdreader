# Problem Definition: Gate 9 release-only

- **Slug**: gate9-release-only
- **Created**: 2026-11-18
- **Inputs used**: intake.md | research.md

## Problem Statement

The Android build (Gate 9) currently runs on every non-draft PR and branch push, so release verification is entangled with ordinary development; maintainers cannot prevent a `v*` tag from being published without any Android check, despite an approved policy that makes Gate 9 release-only.

## Affected Users & Stakeholders

- **Users**: release engineer — needs a trusted Android artifact and a tag that cannot be published unverified.
- **Stakeholders**: maintainer / repo admin — owns branch protection and the release policy; approves the rule change.

## Goals

- Keep Host gates 1–8 mandatory on PRs to `dev`/`main`.
- Run Gate 9 only via manual `workflow_dispatch` on a release candidate commit and on pushes of tags matching `v*`.
- Protect `v*` tags with a GitHub ruleset requiring both the Host and Android checks.

## Non-Goals

- Changing Host gates, their contexts, or branch-protection settings for `dev`/`main`.
- Signing builds, publishing to stores, or changing `v*` tag naming.
- Importing any Forgejo/factory workflow into the target.

## Success Metrics

- A normal PR to `dev` triggers only the Host-gates job (baseline: both jobs today).
- A `workflow_dispatch` and a pushed `v*` tag trigger both jobs.
- `gh api repos/lfgranja/mdreader/rulesets` shows a `v*` tag ruleset requiring the two exact check contexts (baseline: empty).

## Cost of Inaction

`v*` tags can be published with no Android verification, and every PR continues to pay the ~27-minute Android job.

## Open Questions

- None blocking; the governance decision is explicit and the remote API result is validated during implementation.
