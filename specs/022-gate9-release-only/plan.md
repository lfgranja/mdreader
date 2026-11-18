# Implementation Plan: Gate 9 Release-Only

**Branch**: `feature/022-gate9-release-only` | **Date**: 2026-11-18 | **Spec**: [`spec.md`](./spec.md)

## Summary

Restrict the Android build (Gate 9) in `.github/workflows/ci.yml` to `workflow_dispatch` and `v*` tag pushes, add `push.tags: ['v*']`, then create a GitHub ruleset for `v*` tags requiring the exact Host and Android check contexts.

## Technical Context

- **Language/Version**: YAML workflow + GitHub REST API (no Rust source changes)
- **Primary Dependencies**: `gh` CLI (authenticated), GitHub Actions
- **Testing**: workflow inspection (`gh api`), ruleset inspection (`gh api`), target gates (`cargo fmt`, `nextest`, `clippy`) unchanged
- **Target Platform**: `lfgranja/mdreader` GitHub repository
- **Constraints**: no auto-merge, no force-push, no changes to Host gates or `dev`/`main` protection, factory read-only

## Constitution Check

| Gate / Principle | Status | Notes |
|---|---|---|
| Authority = target CI/PR (`gh`) | PASS | GitHub Actions only; no factory import |
| Host gates unchanged | PASS | Only `android-build` job trigger + `push.tags` change |
| Reversible | PASS | Revert workflow + delete ruleset |
| No LLM final validation | PASS | Deterministic `gh api` + CI decide |
| Concurrency/worktree ownership | PASS | Own branch + worktree, ledger active |

## Project Structure

```
.github/workflows/ci.yml     # trigger scoping (only source file changed)
specs/022-gate9-release-only/{spec,plan,tasks,analyze,converge}.md
.specify/assessments/gate9-release-only/*   # assess artifacts
.specify/validations/gate9-release-only/validate.md
.specify/releases/gate9-release-only/release.md
```

## Implementation Order

1. Edit `ci.yml`: add `push.tags: ['v*']`; change `android-build.if` to dispatch-or-v-tag.
2. Validate YAML (`ruby -e YAML.load_file` or `python3` with PyYAML if present; else `gh` actionlint not available — use structural diff + CI).
3. Create remote `v*` ruleset via `gh api` with both required check contexts; verify live.
4. Run target gates (fmt/nextest/clippy unaffected — still green from PR #222) and open PR.

## Rollback

Revert the `ci.yml` commit and delete the `v*` ruleset via `gh api -X DELETE /rulesets/{id}`.

## Complexity Tracking

| Risk | Why needed | Rejected alternative |
|---|---|---|
| Ruleset payload may be rejected by API | Required for tag protection | Document-only policy (no enforcement) |
