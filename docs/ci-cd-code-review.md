# CI/CD e Code Review Rigoroso (mdmv-dark-factory)

Data: 2026-11-07
Status: implementado (base) — requer configuração de branch protection no host Git (GitHub/Forgejo)

## Princípios (determinism-first + revisão adversarial)
- **Determinismo decide merge**: build/test/lint/scan/gates são hardcoded no pipeline; falha = bloqueio. (mdmv-dark-factory `20-determinism-first`, `04-camada-verificacao`)
- **Checker separado**: revisão por agente/LLM é informativa; aprovador humano/automação de policy != autor; nunca LLM-juiz como gate único. (`21-llm-as-judge-adversarial`, `verification-layer.md` I1/I2)
- **Holdout/contratos**: testes e contratos versionados são o oráculo determinístico; cenários sensíveis devem permanecer fora do alcance de “auto-fix” cego.
- **Execution record**: evidência de comandos/gates/artefatos no PR e no CI.

## Pipeline implementado
- `.github/workflows/ci.yml`
  - **host-gates** (PR/push): fmt, nextest (`--profile ci`, `PROPTEST_CASES=32`), two-way diff `scripts/expected-tests.txt`, docs, clippy `-D warnings`, `cargo deny` + `cargo audit`, deny-list scan, closed-set (`no _ =>` em `crates/*/src`), gate 8 feature-scoped, `check-manifest.sh`/`check-shell-purity.sh`, e execução autoritativa de `scripts/verify.sh`.
  - **android-build** (gate 9, aarch64 debug): roda após host-gates; publica APK debug como artefato (14 dias). Requer Android SDK/NDK/JDK 17/tauri CLI no runner.

## Code review
- `.github/CODEOWNERS`: roteamento por blast radius (crates/src-tauri/vendor/CI/contracts/specs).
- `.github/pull_request_template.md`: checklist de verificação determinística, contratos/constituição, risco/rollback, evidência.
- `.github/CONTRIBUTING.md`: política local e passos de branch protection.

## Branch protection (ação manual obrigatória no host)
Em `lfgranja/mdreader` (GitHub) e/ou mirror Forgejo, proteger `main` e `dev`:
- PR obrigatório, >=1 aprovação, CODEOWNERS review, stale approvals descartadas.
- Status checks obrigatórios: `Host gates (verify.sh 1-8 subset + supply chain)` (+ Android build quando aplicável).
- Resolver conversas; sem force-push/delete; opcional merge queue/squash.

## Observação
O repositório atual tem diff local extenso e trabalho em andamento de spec 010 (T232/T233). CI/code-review **não substitui** validação device/contratos; apenas garante que merges passem por gates determinísticos e revisão rigorosa.
