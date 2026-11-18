# Converge: 022 Gate 9 Release-Only

**Date**: 2026-11-18
**Status**: implemented; awaiting PR/merge verification

## Delivered

- Assess: `.specify/assessments/gate9-release-only/{intake,research,problem,concept,decision}.md` — verdict `go` (Option A).
- Spec/plan/tasks/analyze/checklist: `specs/022-gate9-release-only/{spec,plan,tasks,analyze}.md` + `checklists/requirements.md`.
- Code: `.github/workflows/ci.yml` — `on.push.tags: ['v*']`; `android-build.if` scoped to `workflow_dispatch` or `refs/tags/v*`.
- Remote: GitHub ruleset id `27578490` (`v* release gates`), active, target `tag`, include `refs/tags/v*`, required checks = Host + Android contexts.

## Evidence (measured 2026-11-18)

- `git diff origin/dev -- .github/workflows/ci.yml` — 2 insertions, 1 deletion.
- YAML parse via PyYAML — `on.push.tags == ['v*']`.
- `gh api --method POST /rulesets` — returned id 27578490 with both contexts; `gh api /rulesets` lists it.
- `cargo fmt --all -- --check` — exit 0. Rust source unchanged from PR #222 (Host gates 9m47s, Android 27m29s, 1077/1077 tests green).

## Honest status

- Local deterministic checks PASS.
- PR/merge/CI for this artifact: [NV] until opened and observed green.
- Rollback: revert ci.yml commit + `gh api -X DELETE repos/lfgranja/mdreader/rulesets/27578490`.
