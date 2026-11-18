# CI/CD e Code Review Rigoroso (mdmv-dark-factory)

Data: 2026-11-18
Status: branch protection conferido via GitHub API; verificação de workflow e Gate 9 release-only ainda pendentes.

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

## Branch protection (`lfgranja/mdreader`)
Conferido em 2026-11-18 com `gh api` nos endpoints `repos/lfgranja/mdreader/branches/dev/protection` e `repos/lfgranja/mdreader/branches/main/protection`:

| Branch | Aprovações | CODEOWNERS | Check requerido |
|---|---:|---|---|
| `dev` | 0 | Não | `Host gates (verify.sh 1-8 subset + supply chain)` |
| `main` | 1 | Sim | `Host gates (verify.sh 1-8 subset + supply chain)` |

Política aprovada: a aprovação em `main` deve ser independente do autor; `dev` aceita merge sem aprovação após os checks exigidos. A configuração observada da branch é autoridade para o estado remoto.

## Gate 9
O workflow Android atual ainda está configurado fora do ciclo release-only. A mudança pendente deve executar Gate 9 via `workflow_dispatch` no commit candidato e exigir os checks Host e Android em um ruleset de tags `v*`; ainda não implementada nem verificada.
