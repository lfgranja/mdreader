# Validation (R4): 022 Gate 9 Release-Only

- **Date**: 2026-11-18
- **Spec**: `specs/022-gate9-release-only/spec.md`
- **Status**: PASS (local deterministic evidence; remote PR/CI marked [NV])

## Gate results

| Check | Command | Result |
|---|---|---|
| Spec quality | `checklists/requirements.md` | all items checked |
| Workflow diff | `git diff origin/dev -- .github/workflows/ci.yml` | 2+/1-, intended lines only |
| YAML parse | `python3 -c yaml.safe_load(...)` | OK, `push.tags == ['v*']` |
| Trigger semantics | inspect job `if` | `workflow_dispatch` or `refs/tags/v*` |
| Host gates unchanged | branch protection via `gh api` | dev/main untouched (Host context still required) |
| Tag ruleset | `gh api repos/lfgranja/mdreader/rulesets` | id 27578490, active, both contexts |
| Format | `cargo fmt --all -- --check` | exit 0 |
| Rust surface | `git diff origin/dev --name-status` | only ci.yml + STATE.md tracked |

## Success criteria mapping

- SC-G9-001: satisfied by job `if` (no PR branch condition) — verified by inspection.
- SC-G9-002: satisfied by `push.tags` + `workflow_dispatch` — verified by inspection; live dispatch/tag run [NV] until executed.
- SC-G9-003: verified live via `gh api` rulesets.
- SC-G9-004: rollback documented in converge/release.

## Notes

- No LLM final validation used; only deterministic commands.
- Live behavior of a real `v*` tag push is [NV] (no tag push performed; would publish a release, out of scope).
