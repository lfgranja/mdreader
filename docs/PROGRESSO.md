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

## 2026-11-20 — Rodada 2: merges, dispositivo e coerência documental (append-only)

### Âncora atualizada
- `feature.json` = `specs/024-directory-resume-offer` (ciclo #024 concluído e mesclado).
- HEAD local `dev` = `44841427` (após `origin/dev` @ `29cb287f`, merges dos PRs #227 e #228).
- Factory HEAD `b1deab1` — somente leitura.

### Contagens reais (via `ls`)
- `specs/`: **25**; `.specify/assessments/`: **28**; `.specify/bugs/`: **33**; `.specify/validations/`: **28**; `.specify/releases/`: **28**; `docs/research/`: **1** (`index.md`).

### Evidência local
- `scripts/verify.sh` gate 1–8 em execução nesta rodada; execução anterior `VERIFY_EXIT=0`.
- 4 checks SDD PASS: completeness (25/0), DAG (855 tasks/0 cycles), uniqueness (0 dup), converge (24 baselined).
- PRs #227 e #228 **MERGED** via `gh` (Host gates pass: 10m38s e 7m54s). Nenhum PR aberto.
- Dispositivo 2201117TY (Android 13/SDK 33) conectado: `app.mdreader` v0.1.0, INTERNET ausente, sem crashes (logcat -t 200). Escrita no dispositivo não executada.

### Correções documentais desta rodada
- `specs/010-directory-reading/tasks.md:144`: T233 marcado `[x]` (implementado no ciclo 024/PR #228).
- `.specify/bugs/fastlane-distribution-artifacts/fix.md`: nota de reconciliação do guard `test_noscope` (`fastlane/metadata` intencional para F-Droid rc2).

### Afirmações remotas — não verificadas localmente
- [NV] CI após os merges; aplicação efetiva de ruleset/proteções; gate 9 em artifact release.
- [NV] Janela visual de 3s, TalkBack/foco, TTFA fim-a-fim e `play_to_audio_ms` em dispositivo (exigem escrita no dispositivo/build Android, não autorizados nesta rodada).

### Nota sobre a spec 020
- Interjeição do SE às 2026-11-20: **desconsiderar** a instrução anterior sobre "spec 020 do gate de allowlist" (mensagem enviada por engano). Nenhuma ação adicional foi tomada sobre a spec 020 nesta rodada.

## 2026-11-21 — Roteiro de testes funcional (append-only)

- Criado `docs/roteiro-testes.md` com 9 seções: pré-condições, gates host, engine/vozes (CLI), transporte/offer/bookmark, UI/WebView/SAF, device ponta a ponta, visual/escuta, bugs com atenção especial e limites conhecidos.
- Base: 4 agentes de pesquisa paralelos (engine/síntese, transporte/sessão, UI/Android, QA/gates), sintetizados em um único arquivo. Comandos ADB/CLI acionáveis e sinais no `diagnostic-record.jsonl` por cenário.
- [NV] nenhuma medição remota foi executada para produzir este roteiro; cenários de dispositivo listados como passos a executar, não como resultados.
