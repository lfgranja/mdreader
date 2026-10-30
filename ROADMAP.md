# Roadmap — mdreader / Spec-Driven Development

> **Constituição:** `.specify/memory/constitution.md` v1.12.0  
> **Fluxos:** `/speckit.assess.*` (discover) → `/speckit.specify` → `/speckit.clarify` → `/speckit.plan` → `/speckit.tasks` → `/speckit.implement` → `/speckit.converge` (deliver) → `/speckit.bug.assess` → `/speckit.bug.fix` → `/speckit.bug.test` (triage)  
> **Objetivo:** `OBJETIVO.md` — leitor de documentos Markdown com áudio/síntese de voz, Rust como única autoridade de posição/transporte/buffering/rate, Kotlin apenas espelho estrutural (SC-035), Tauri 2.12.1, CPAL 0.18.2 + AAudio, mono f32 @ 22050 Hz, sherpa-onnx 1.13.8 + Piper pt-BR (`offline_default`).

---

## 1. Mapa do caminho percorrido (evidência direta)

### 1.1 Discovery — `.specify/assessments/`

| Slug | Estágios | Veredito | Status | Evidência |
|---|---|---|---|---|
| `directory-reading` | `intake.md` apenas | — | **Bloqueado** (intake) | `.specify/assessments/directory-reading/intake.md` |
| `design-son-adoption` | 5/5 (`intake→research→problem→concept→decision`) | `go` → `005` | **Completo / handoff ativo** | `decision.md` (0 clarifications pendentes) |
| `fr-024-synthesis-adapter` | 5/5 | `go` condicional | **Completo; clarificações transportadas para `/speckit.clarify`** | `decision.md`: 5 `[NEEDS CLARIFICATION]` (`SC-009`, fronteiras `Piper`) |
| `ressoa-mobile-app` | 5/5 | `go` (Option A narrow) | **Completo / handoff para `004`** | `decision.md` |
| `ref-device-emulator` | 5/5 | `go` (após 6 gaps G1–G6 resolvidos) | **Completo / handoff para `008`** | `decision.md` (veredito atualizado 2026-10-29) |

**Percorrido:** 4 de 5 ideias com `go`; 1 bloqueado em intake (`directory-reading`). Nenhum assessment pendente de decisão.

### 1.2 Delivery — `specs/`

| # | Spec | spec | plan | tasks | checklists | contracts | Código / testes | Verif. | Estado |
|---|---|---|---|---|---|---|---|---|---|
| 001 | `android-media-shell` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ `tests/`; `VERIFICATION.md` | `gate-8-shell PASS`; `gate-8-full DEFERRED (owner=synthesis)` | **Converge / verificado** |
| 002 | `fr-024-synthesis-adapter` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ `crates/adapter-synthesis/`; V3 attestation (110 pass, `clippy` zero) | `SC-044 DEFERRED`; `V3_ATTESTATION_2026-10-10.md` | **Implementado / converge parcial (defer constitucional)** |
| 003 | `reference-device` | ✓ | ✓ | ✓ (140l) | ✓ | ✓ | apenas refs; emulador Android SDK (API 29+, local/free) viável para verif. estrutural (manifest/service/notific/focus/diag/rate); **físico obrigatório** para `gate-8-full` (áudio real, SC-009, Bluetooth/car, telephony/alarm, 10/10) | `VERIF-EMULATOR` + `gate-8-full` DEFERRED | **Plan/Tasks presentes; implementação pendente** |
| 004 | `ressoa-dogfood-build` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ `verification/` | `verify.sh` GREEN `9501bba` (Phase 17, 2026-10-22) | **Converge / verificado** |
| 005 | `adopt-design-sona` | ✓ | ✓ (post-clarify) | **✗ pendente** | ✓ (16/16) | ✓ | fonte reflete design-dir; **sem** `tests/` próprio do spec | — | **Clarify→Plan completo; `/speckit.tasks` pendente** (`STATE.md` 2026-10-24: "NEXT: `/speckit.tasks`") |
| 006 | `lente-sinestesica-completa` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | converge completo | **Concluído** |
| 007 | `son-a-complete-design` | ✓ | ✓ | **✗ não encontrado** | ✓ | ✓ | — | — | **Specify/plan feito; `/speckit.tasks` e implementação pendentes** |
| 008 | `ref-device-emulator` | ✓ | ✓ | ✓ | ✓ | ✓ | apenas docs/estrutural | converge completo | **Concluído (camada estrutural, sem código)** |

**Percorrido:**
- 4 specs totalmente convergidos com evidência de verificação (`001`, `004`, `006`, `008`).
- 1 spec implementado com attestation V3 e clarifications batched resolvidas, mas gate constitucional aberto (`002` → `SC-044`).
- 1 spec em ponto de transição exato (`005` → próximo é `/speckit.tasks`).
- 1 spec com plan mas sem tasks/implement (`003`).
- 1 spec com spec/plan mas sem tasks (`007`).

### 1.3 Triage — `.specify/bugs/`

| Slug | assess | fix | test | Status | Resumo |
|---|---|---|---|---|---|
| `corrupt-design-asset` | ✓ | ✗ | ✗ | Assess concluído | Asset `ui-proposta-2-lente-sinestesica.png` contém imagem de Rust tutorial, não mockup SONA. Afeta verificação visual de 005/006/007. |
| `corrupt-ui-proposta-asset` | ✓ | ✗ | ✗ | Assess concluído | Pasta vazia além do assessment. |
| `sona-visual-core-fix` | ✓ | ✓ (parcial) | ✓ (parcial) | Fix parcial | Header quebrado, waveform vazio, botões pequenos persistem (`sona_final_updated.png`). Status: high, visual core ainda quebrado. |
| `sona-visual-fix` | ✓ | ✗ | ✗ | Assess concluído | Pasta vazia além do assessment. |
| `sona-waveform-missing-in-screenshot` | ✓ (re-evaluated) | ✗ | ✗ | Assess concluído | `drawWaveform()` precisa de `SampleQueue` ativo (`total_units > 0`) para ribbon funcional; screenshot com `monkey` não confirma. |

**Percorrido:** 5/5 assessados (valid). Apenas 1 entrou em fix/test, mas com fix parcial. Nenhum bug fechado com `bug.test` green. Nenhuma regressão ativa confirmada (`bg_14` interrompido, não falha verificada).

---

## 2. O que ainda falta (por fluxo e por spec)

### Assessment (discovery)

- [`directory-reading`]: completar `research` → `define` → `shape` → `decide`, ou documentar `kill` com razão. Baixa prioridade; não bloqueia 005.
- [`fr-024`]: fechar `SC-009` benchmark latência e validar fronteiras `Piper`; depois `SC-044` (gate-8-full) só passa quando 002 confirmar.
- [`design-son-adoption` / `ressoa-mobile-app`]: já entregues; nenhum esforço adicional necessário até novos requisitos surgirem.

### Spec / Delivery

- **005 (ativo, 80/20):** `/speckit.tasks` → implementação das tarefas do plano → `/speckit.implement` → testes do spec → `/speckit.converge`. Todos os clarificações (E1–E3, D1–D5) já aceitos; `plan.md`, `checklists/` (16/16), contratos e `data-model.md` presentes.
- **002 (deferido):** fechar `SC-009` + `SC-044`; tarefas T045/T046/T048 pendentes; depois atualizar attestation para V4 com `gate-8-full PASS`.
- **003:** executado `/speckit.tasks`; implementar referências de referência-device; gerar `tests/` próprio; attestation / verificação. Emulador viável para verificação estrutural; físico obrigatório para `gate-8-full`.
- **001:** manter `gate-8-full` como `DEFERRED` até 002/SC-044; nenhuma ação até então.
- **007:** criar `/speckit.tasks`, depois `/speckit.implement` (RMS instrumentation, canvas waveform, repeat button, token expansion jade/celeste, glow/neon).

### Bug (triage)

- `corrupt-design-asset`: precisa `fix`/`test` (substituir asset corrupto; adicionar design-asset integrity gate).
- `sona-visual-core-fix`: fix parcial aplicado, mas visual core ainda quebrado — precisa fix completo + test.
- `sona-waveform-missing-in-screenshot`: precisa `fix`/`test` (confirmar `AppState` publica `channel_waveform` com `total_units > 0`; rebuild APK; screenshot com reprodução ativa).
- `corrupt-ui-proposta-asset` / `sona-visual-fix`: vazios, baixa prioridade, nenhuma regressão ativa confirmada.

---

## 3. Atualização baseada em pesquisa (fontes externas)

| Fonte | Confirmação | Relevância para o projeto |
|---|---|---|
| Android Developers — Foreground service types (developer.android.com) | `FOREGROUND_SERVICE` + `FOREGROUND_SERVICE_MEDIA_PLAYBACK` obrigatórios a partir de Android 14 (API 34); `mediaPlayback` tipo requer minSdk 29 | Valida SC-006/SC-032 no `001`/`004`; manifest editado diretamente (Tauri 2 sem `mobile::update_manifest`) |
| Tauri v2.12.x release notes | 2.12.x line ativa; 2.12.3 é o patch mais recente da linha | Valida pin `tauri = "=2.12.1"` na constituição; nenhuma mudança breaking na 2.12.x que afete o build |
| cpal 0.18.2 CHANGELOG (RustAudio/cpal) | Lançado 2026-08-16; AAudio: Xruns reportados como `ErrorKind::Xrun` | Valida backend CPAL 0.18.2 + AAudio no `crates/player/src/backend.rs`; exato-request 22050 Hz/mono/f32 |
| sherpa-onnx 1.13.8 (crates.io, pub.dev, k2-fsa) | Android arm64 build disponível; Piper TTS suportado; Kokrio/Kokoro também | Valida `offline_default` = sherpa-onnx + Piper pt-BR (`faber` CC0) no `crates/adapter-synthesis` |
| sherpa-onnx Android build docs (k2-fsa.github.io) | Build scripts para Linux/macOS; Android arm64 suportado | Valida viabilidade do build Android gate 9 (`cargo tauri android build --target aarch64`) |

---

## 4. Próximo passo recomendado (80/20 — maior avanço, menor esforço)

### 80/20: `/speckit.tasks` para `005-adopt-design-sona`

**Por que este é o 80/20:**

1. **Estado de prontidão máximo:** `spec.md`, `plan.md`, `checklists/` (16/16), contratos (`status-surface`, `earcon-mapping`, `surface-audit`), `data-model.md` e `quickstart.md` — todos presentes e revisados. Não há bloqueio de especificação.
2. **Branch ativo:** `005-adopt-design-sona` é o branch corrente (`0c985ee`), com 51 arquivos dirty já existentes. O STATE.md (2026-10-24) diz explicitamente "NEXT: `/speckit.tasks`".
3. **Clarificações resolvidas:** E1–E3 e D1–D5 já aceitas pelo mantenedor (CLARIFICATION_BATCH_RESOLVED). Zero `NEEDS CLARIFICATION` bloqueando.
4. **Impacto amplo, esforço moderado:** afeta o driving surface completo (`status_view.html`, `MediaMirror.kt`, tokens DESIGN-SONA, earcons, contraste, animações reduzidas). Tokens e assets já existem em `DESIGN-SONA/` (tokens.md, components.md, preview.html).
5. **Cadeia de convergência:** fechar 005 libera recursos para fechar `SC-044` em 002 e depois `003`, desbloqueando `gate-8-full` para `001`.

**Execução sugerida:**

1. Executar `/speckit.tasks` com input = resumé de `005-adopt-design-sona/plan.md` + `tasks.md` existente (Setup/Foundational/Polish/Verification; TDD RED gate binding; caminhos exatos `status_view.html` `:21-22,:26,:29-30,:400,:748`, `MediaMirror.kt`:728-743, `player/backend.rs` `:511`, `earcon.rs` `:11-25`, `diag/writer.rs` `:69`; scaffold-first + subagent waves + `commit-hook.sh`).
2. Priorizar tarefas que entregam `SC-001` (papel de focus), `SC-007` (freeze de animações sob `prefers-reduced-motion`) e `SC-002` (disposição haptic) — já validadas no clarify.
3. Após tarefas, ir para `/speckit.implement` e gerar `tests/` próprio do spec (hoje ausente), depois `/speckit.converge` com attestation.

### Alternativa de maior esforço (se 005 estiver bloqueado):

- **Fechamento de `SC-009` + `SC-044` via `/speckit.clarify` / benchmark** para `002`. Maior impacto (desbloqueia `gate-8-full` físico), mas esforço muito maior (consent UI, 2º engine real, benchmark de latência no dispositivo, medição SC-009).

---

## 5. Marcas de progresso (checklist de convergência)

- [x] Constituição v1.12.0 ratificada; gates 1–8 definidos; `SC-035`/`SC-051`/`SC-009`/`SC-044` documentados.
- [x] `004-ressoa-dogfood-build`: verificado verde (`9501bba`, Phase 17, 2026-10-22); `cargo check` histórico PASS.
- [x] `001-android-media-shell`: `VERIFICATION.md` + `gate-8-shell PASS`; `gate-8-full DEFERRED` (owner=synthesis, aguarda 002/SC-044).
- [x] `002-fr-024-synthesis-adapter`: `V3_ATTESTATION` (110 pass, `clippy` zero) + `CLARIFICATION_BATCH_RESOLVED` + `adapter-synthesis/` (12 .rs); `SC-044 DEFERRED`; 1 engine real (`offline_default`); `SC-051` preservado. Pendentes: T045, T046, T048.
- [x] `005-adopt-design-sona`: clarificações E1–E3/D1–D5 resolvidas; plano/próximo=`/speckit.tasks`; branch ativo (`005-adopt-design-sona`; `0c985ee`; 51 dirty; `cargo check` interrompido). Pendente: `/speckit.tasks` → implement → tests → converge.
- [x] `006-lente-sinestesica-completa`: converge completo (T001–T055 todos `[x]`; Convergence Phases 7–9, T041–T055 `[x]`).
- [x] `008-ref-device-emulator`: concluído (camada estrutural apenas; `VERIF-EMULATOR.md`; `gate-8-full` físico preservado).
- [ ] `005`: `/speckit.tasks` executado → `/speckit.implement` → `tests/` próprio → `/speckit.converge` (80/20).
- [ ] `002`: `SC-044` fechado → 2º engine → `gate-8-full PASS`; depois attestation V4. Pendentes: T045, T046, T048.
- [ ] `003`: implementação de dispositivo de referência → `tests/` próprio → attestation. Emulador viável para estrutural; físico obrigatório para `gate-8-full`.
- [ ] `007`: `/speckit.tasks` → implementação (RMS, canvas, repeat, token expansion).
- [ ] `.specify/bugs/`: `corrupt-design-asset`, `sona-visual-core-fix`, `sona-waveform-missing-in-screenshot` precisam `fix`/`test`; `corrupt-ui-proposta-asset`/`sona-visual-fix` vazios (baixa prioridade).
- [ ] `directory-reading`: `decision.md` (go/kill/clarify) — baixa prioridade; não bloqueia 005.
- [ ] `cargo check` / `verify.sh` PASS no working tree atual (atualmente interrompido / não confirmado; deve ser verificado após cada etapa).

---

*Atualizado: 2026-10-29 · Base: `AGENTS.md` (Repository Guidelines reescrito neste turno), `.specify/memory/STATE.md`, `specs/*/spec.md`/`plan.md`/`tasks.md`, `.specify/assessments/*/decision.md`, `.specify/bugs/*/assessment.md`, `OBJETIVO.md`, `constitution.md` §1309, pesquisas web (Tauri 2.12.x, CPAL 0.18.2 AAudio, sherpa-onnx 1.13.8 Android).*
