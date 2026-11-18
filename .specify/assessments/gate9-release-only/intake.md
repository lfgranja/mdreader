# Idea Intake: Gate 9 release-only

- **Slug**: gate9-release-only
- **Created**: 2026-11-18
- **Source**: Approved governance decision recorded in `.specify/memory/STATE.md` (2026-11-18) + repository path `.github/workflows/ci.yml`
- **Type**: compliance

## Idea (as captured)

> "Gate 9 release-only (confirmado): gates Host 1–8 permanecem nos PRs; Gate 9 roda manualmente (`workflow_dispatch`) no commit candidato de release; tags `v*` devem ser protegidas por ruleset GitHub exigindo sucesso de Host e Android antes da liberação. A implementação e a configuração remota ainda estão pendentes de execução e verificação." — `.specify/memory/STATE.md`

## Restated

Make the Android build (Gate 9) a release-time gate only: keep Host gates on pull requests, run Gate 9 on manual dispatch and on `v*` tag pushes, and protect `v*` tags with a GitHub ruleset requiring both Host and Android checks.

## Origin & Context

- **Raised by**: Software Engineer (decision recorded in STATE.md, 2026-11-18)
- **Trigger**: Gate 9 currently runs on every non-draft PR and branch push (`.github/workflows/ci.yml:121`), making it mandatory during ordinary development; the approved release policy requires it only for release candidates.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: exact `v*` tag naming already in use — verified: `v0.1.0-alpha.1`, `v0.1.0-rc1`, `v0.1.0-rc2` exist]
- [NEEDS CLARIFICATION: whether the ruleset can be created with current token permissions — to verify live via `gh api`]
