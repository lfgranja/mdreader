# Progresso — Estado Verificado

> Registro **append-only**: cada entrada registra o estado verificado em uma data.
> Entradas antigas nunca são reescritas nem removidas; correções são novas entradas.

## 2026-11-20 — estado verificado

### Âncora de trabalho
- `feature.json` = `specs/010-directory-reading` (conteúdo: `{"feature_directory":"specs/010-directory-reading"}`).
- HEAD local: `f88579f3`.
- `origin/dev`: `bc14f859` (24 commits atrás (1 commit local à frente; merge-base b3e808ad; `git rev-list --left-right --count HEAD...origin/dev` = 1 24)).
- Factory HEAD: `b1deab1` — apenas fonte de ideias; nunca modificada.

### Contagens reais (via `ls`, não prosa)
- `specs/`: **22** diretórios (`ls -d specs/*/ | wc -l`).
- `.specify/assessments/`: **22** entradas.
- `.specify/bugs/`: **33** entradas.
- `.specify/validations/`: **2** entradas (`device-webview-a11y-gates/`, `v0.1.0-rc2/`).
- `.specify/releases/`: **2** entradas (`v0.1.0-rc1/`, `v0.1.0-rc2/`).
- `docs/research/`: **0** — diretório ausente antes desta escrita; criado em 2026-11-20 para abrigar `docs/research/index.md`.

### Gates determinísticos
- `scripts/verify.sh`: **PASS (exit 0)** somente com
  `SHERPA_ONNX_LIB_DIR=/tmp/sherpa-cache/sherpa-onnx-v1.13.8-linux-x64-static-lib/lib`
  (biblioteca vendored extraída do archive; o download do GitHub dá timeout).
- Sem essa variável, o **gate 2 fica RED** por timeout de download.

### Checks SDD RED
- `check_sdd_completeness.py`: **RED** — 22 gaps (baselined=12; 12 `STALE_BASELINE`).
- `check_work_order_dag.py`: **RED** — 13 `duplicate_task` em `specs/002`, `005`, `009`, `010`.
- `check_task_id_uniqueness.py`: **RED** — 4 specs com IDs duplicados.
- `check_converge_append_only.py`: **RED** — 21 `converge.md` ausentes no baseline + 21 unbaselined.
- Baselines em `docs/governance/` são untracked e foram preservados.

### Afirmações remotas — não verificadas localmente
- [NV] CI: status/execuções de workflows remotos não verificado.
- [NV] PRs abertos: não verificados.
- [NV] Proteções de branch: não verificadas.
- [NV] Gate 9 (Android): não executado nem verificado.
- [NV] Dispositivo/emulador: não verificado.
