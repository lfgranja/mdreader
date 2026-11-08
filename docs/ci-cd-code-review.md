# CI/CD e Code Review Rigoroso (mdmv-dark-factory)

Data: 2026-11-07
Status: implementado e com branch protection aplicado via API/gh (2026-11-07)

## Princípios (determinism-first + revisão adversarial)
- **Determinismo decide merge**: build/test/lint/scan/gates hardcoded no pipeline; falha = bloqueio. (mdmv-dark-factory `20-determinism-first`, `04-camada-verificacao`)
- **Checker separado**: revisão por agente/LLM é informativa; aprovador humano/policy != autor; nunca LLM-juiz como gate único. (`21-llm-as-judge-adversarial`, `verification-layer.md` I1/I2)
- **Holdout/contratos**: testes e contratos versionados como oráculo determinístico.
- **Execution record**: evidência de comandos/gates/artefatos no PR/CI.

## Pipeline
- `.github/workflows/ci.yml`
  - **host-gates** (PR/push): fmt, nextest (`--profile ci`, `PROPTEST_CASES=32`), two-way diff `scripts/expected-tests.txt`, docs, clippy `-D warnings`, `cargo deny` + `cargo audit`, deny-list, closed-set, gate 8 feature-scoped, `check-manifest.sh`/`check-shell-purity.sh`, `scripts/verify.sh`.
  - **android-build** (gate 9, aarch64 debug): após host-gates; publica APK debug (14 dias).

## Code review
- `.github/CODEOWNERS`, `.github/pull_request_template.md`, `.github/CONTRIBUTING.md`.

## Branch protection (aplicado em `lfgranja/mdreader`)
Aplicado 2026-11-07 via `gh api` (token autenticado, `viewerPermission=ADMIN`) em `dev` e `main`:
- Required status checks (strict/up-to-date): `Host gates (verify.sh 1-8 subset + supply chain)`.
- PR obrigatório, `required_approving_review_count=1`, CODEOWNERS review, dismiss stale approvals.
- Required conversation resolution: enabled.
- Force-push: disabled; deletions: disabled.
- `enforce_admins=false` (admins não forçados pelo gate; pode ser elevado para `true` se desejado).

Observação: check `Android build (gate 9, aarch64 debug)` não exigido como required (custo/tempo; roda condicionalmente). Pode ser adicionado quando o runner Android estiver estável.
