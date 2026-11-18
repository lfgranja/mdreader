# Concept: Gate 9 release-only

- **Slug**: gate9-release-only
- **Created**: 2026-11-18
- **Recommended option**: Option A

## Options

### Option A — Scope the workflow trigger, then add a tag ruleset
- **Sketch**: Restrict the `android-build` job to `workflow_dispatch` and `v*` tag pushes; add `push.tags: ['v*']` so the workflow runs on release tags; create a GitHub ruleset for `v*` tags requiring the Host and Android check contexts.
- **Appetite**: small
- **Trade-offs**: Wins exact policy compliance and removes Android cost from PRs. Sacrifices early Android compile feedback on PRs. Risk: a malformed ruleset payload; mitigated by live `gh api` verification.
- **Rabbit holes**: trying to make the ruleset require a check that never runs on tags (both jobs must trigger on `v*`); check-context names must match job `name:` values byte-for-byte.

### Option B — Keep Gate 9 on PRs but add a tag ruleset only
- **Sketch**: Leave `ci.yml` unchanged; only add the `v*` ruleset.
- **Appetite**: small
- **Trade-offs**: Enforces tag safety but does not satisfy the approved "release-only" policy and keeps the ~27-minute PR cost.
- **Rabbit holes**: none, but it is a policy mismatch.

### Option C — Do nothing
- **Sketch**: Document the policy without implementing it.
- **Appetite**: none
- **Trade-offs**: Zero risk, zero compliance; tags remain unprotected.
- **Rabbit holes**: n/a.

## Recommendation

Option A. It is the only option that satisfies the approved governance decision, is small, and is fully verifiable with deterministic commands (`gh api`, workflow inspection).

## Out of Scope (for the recommended option)

- Signing, store publication, tag renaming, changes to `dev`/`main` protection.

## Assumptions to Validate

- The token can create rulesets (verify via `gh api`); if not, record `[NV]` and escalate.
- Check contexts are exactly `Host gates (verify.sh 1-8 subset + supply chain)` and `Android build (gate 9, aarch64 debug)`.
