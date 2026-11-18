# Decision: Gate 9 release-only

- **Slug**: gate9-release-only
- **Decided**: 2026-11-18
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | Approved governance decision + verified ci.yml trigger (`.github/workflows/ci.yml:121`) |
| Evidence strength | strong | Live `gh api` protection/rulesets, CI timings from PR #222, tag list — all measured this session |
| Value vs. inaction | strong | Prevents unverified `v*` tags and removes ~27 min Android cost from every PR |
| Feasibility / appetite | strong | Option A is small: one workflow file + one ruleset |
| Strategic fit | strong | Matches constitution/CONTRIBUTING authority split and the 80/20 choice in STATE.md |
| Risk posture | strong | Reversible config change; rollback = revert workflow + delete ruleset |

## Verdict & Rationale

`go`. The problem is real and approved, evidence is measured (not assumed), and Option A is a small, reversible change with clear deterministic verification. Proceed to specification.

## If go — Handoff to `/speckit-specify`

- **Problem**: Gate 9 is entangled with PR CI and `v*` tags are unprotected.
- **Chosen approach**: Option A — scope the `android-build` job to `workflow_dispatch` + `v*` tags, add `push.tags: ['v*']`, create a `v*` ruleset requiring Host + Android check contexts.
- **In scope**: `.github/workflows/ci.yml`, remote `v*` ruleset, spec artifacts.
- **Out of scope**: Host gates, branch protection for dev/main, signing/publishing, tag renaming.
- **Success metrics**: PR triggers Host only; dispatch/tag triggers both; ruleset exists with exact contexts.
- **Carried-forward open questions**: verify token can create rulesets (else `[NV]` + escalate); exact check-context strings.
