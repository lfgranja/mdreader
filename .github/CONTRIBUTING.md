# Contributing / Code Review & CI Gate (mdmv-dark-factory aligned)

This repository follows **determinism-first** verification and **adversarial/separated review** principles (see `~/development/MDMV/projetos/mdmv-dark-factory/docs/research/20-determinism-first.md`, `21-llm-as-judge-adversarial.md`, `04-camada-verificacao.md`).

## Non-negotiable gates (deterministic)
1. CI must be green before merge. Required workflow: `.github/workflows/ci.yml` (`host-gates`; Android build job when applicable).
2. `scripts/verify.sh` gates 1-8 are the authoritative host-checkable contract (format, nextest + expected-tests two-way diff, docs, clippy, supply chain, deny-list, closed-set, feature-scoped pipeline checks, named checks). Gate 9 (Android build/device artifact) is required for Android-impacting changes.
3. **LLM/agent review is informative, never a merge gate.** Deterministic checks decide merge eligibility.

## Code review (mandatory, rigorous)
- All changes land via Pull Request to protected branches (`dev`/`main` as configured).
- Require at least one approving review from a qualified reviewer; CODEOWNERS review required for owned paths (`.github/CODEOWNERS`).
- **Checker != author**: the approving reviewer must not be the change author (configure branch protection accordingly).
- For high-blast-radius changes (playback authority, session/store/bookmarks, JNI/Android shell, contracts, supply chain, CI), require explicit risk/rollback notes and device or targeted integration evidence when host checks are insufficient.
- Prefer small, atomic commits/PRs; no unrelated refactors. Squash/merge strategy follows repository policy.

## Branch protection (configure in GitHub/Forgejo settings)
Enable on `main` and `dev`:
- Require pull request before merging.
- Require approvals (>=1) and CODEOWNERS review.
- Dismiss stale approvals on new commits.
- Require status checks to pass: `Host gates (verify.sh 1-8 subset + supply chain)` and, when relevant, `Android build (gate 9, aarch64 debug)`.
- Require conversation resolution.
- Restrict force-push and branch deletion.

## Local preflight
```sh
cargo fmt --all -- --check
cargo nextest run --workspace
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/verify.sh
```
