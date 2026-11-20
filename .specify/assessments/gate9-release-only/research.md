# Idea Research: Gate 9 release-only

- **Slug**: gate9-release-only
- **Created**: 2026-11-18
- **Evidence confidence (overall)**: high

## Users & Demand

- Release engineer needs a trusted, reproducible Android artifact for a `v*` release; today the same build runs on every PR, so its signal is diluted and CI time is spent on every change — [source: `.github/workflows/ci.yml:116-121`] (confidence: high)
- Maintainer requires that a `v*` tag cannot be published before Host and Android checks succeed — [source: STATE.md governance decision, 2026-11-18] (confidence: high)

## Prior Art

- PR #222 (2026-11-18) demonstrated both checks green on a feature branch: Host gates 9m47s, Android build 27m29s — [source: `gh pr checks 222` measured in this session] (confidence: high)
- Existing tags are `v0.1.0-alpha.1`, `v0.1.0-rc1`, `v0.1.0-rc2` — [source: `git ls-remote --tags origin`] (confidence: high)

## Market & Context

- Cost of doing nothing: every PR pays the ~27-minute Android job; a `v*` tag can be created without any Android verification. — [source: ci.yml + live CI timings] (confidence: high)

## Data & Constraints

- `dev` branch protection requires only `Host gates (verify.sh 1-8 subset + supply chain)`, 0 approvals; `main` requires 1 approval + CODEOWNERS — [source: `gh api repos/lfgranja/mdreader/branches/{dev,main}/protection`, measured 2026-11-18] (confidence: high)
- No rulesets currently configured — [source: `gh api repos/lfgranja/mdreader/rulesets` returned empty, measured 2026-11-18] (confidence: high, remote state [NV] after change until re-verified)
- CI provider is GitHub Actions; PR CLI is `gh` (factory Forgejo apparatus is reference only) — [source: AGENTS.md + CONTRIBUTING.md] (confidence: high)

## Evidence Against the Idea

- Restricting Gate 9 from PRs removes early Android compile feedback; contributors could break Android builds until release time. Mitigation: keep the job available via `workflow_dispatch` on any ref, and rely on the fact that Android code is a thin mirror of Rust authority. — [source: inferred from ci.yml structure] (confidence: medium)

## Gaps & Open Questions

- The exact ruleset payload accepted by the GitHub API (check context names must match job `name:` values exactly) — to be validated live.
- [NEEDS CLARIFICATION: none blocking — governance decision is explicit]

## Sources

- `.github/workflows/ci.yml` (host: local repo, policy: allowlisted path)
- `.specify/memory/STATE.md` (host: local repo, policy: allowlisted path)
- `gh api repos/lfgranja/mdreader/branches/dev/protection`, `.../main/protection`, `.../rulesets` (host: api.github.com via gh, measured 2026-11-18)
- `gh pr checks 222 --repo lfgranja/mdreader` (host: api.github.com via gh, measured 2026-11-18)
