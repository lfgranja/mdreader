# Tasks: 022 — Gate 9 Release-Only

**Input**: `specs/022-gate9-release-only/{spec,plan,analyze}.md`
**Assess decision**: `.specify/assessments/gate9-release-only/decision.md` = `go` (Option A).
**Tests**: deterministic inspection commands + target host gates.

## Phase 1: Workflow trigger scoping

- [x] T001 Record baseline evidence: `ci.yml:116-121` Android job runs on push/dispatch/non-draft PR; `gh api .../rulesets` empty. (done in research.md)
- [x] T002 Edit `.github/workflows/ci.yml`: add `push.tags: ['v*']` under `on.push`.
- [x] T003 Change `android-build.if` to `github.event_name == 'workflow_dispatch' || startsWith(github.ref, 'refs/tags/v')`.
- [x] T004 Verify YAML parses and the diff touches only the intended lines.

## Phase 2: Remote tag ruleset

- [x] T005 Create a GitHub ruleset targeting `v*` tags with required checks = Host + Android contexts.
- [x] T006 Verify live via `gh api repos/lfgranja/mdreader/rulesets` (record [NV] if unavailable).

## Phase 3: Verification & delivery

- [x] T007 Re-run target gates (fmt, nextest census, clippy) — unchanged Rust surface.
- [x] T008 Validate artifact (`.specify/validations/gate9-release-only/validate.md`).
- [x] T009 Release artifact (`.specify/releases/gate9-release-only/release.md`) with SemVer, changelog IDs, rollback.
- [ ] T010 Commit, push, open PR, observe CI green, deliberate merge to `dev`, cleanup.

## Dependencies

T002–T004 before T005; T005–T006 before T007; T008 before T009; T009 before T010.
