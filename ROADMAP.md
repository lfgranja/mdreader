# Roadmap — mdreader / Spec-Driven Development

> **Constituição:** `.specify/memory/constitution.md` v1.12.0  
> **Fluxos:** `/speckit.assess.*` (discover) → `/speckit.specify` → `/speckit.clarify` → `/speckit.plan` → `/speckit.tasks` → `/speckit.implement` → `/speckit.converge` (deliver) → `/speckit.bug.assess` → `/speckit.bug.fix` → `/speckit.bug.test` (triage)

---

## 1. Mapa do caminho percorrido (evidência direta)

### 1.1 Discovery — `.specify/assessments/`

| Slug | Estágios | Veredito | Status | Evidência |
|---|---|---|---|---|
| `directory-reading` | `intake.md` apenas | — | **Bloqueado** (intake) | `.specify/assessments/directory-reading/intake.md` |
| `design-son-adoption` | 5/5 (`intake→research→problem→concept→decision`) | `go` → `005` | **Completo / handoff ativo** | `.specify/assessments/design-son-adoption/decision.md` (0 clarifications) |
| `fr-024-synthesis-adapter` | 5/5 | `go` condicional | **Completo; clarificações transportadas para `/speckit.clarify`** | `decision.md`: 5 `[NEEDS CLARIFICATION]` (`SC-009`, fronteiras `Piper`) |
| `ressoa-mobile-app` | 5/5 | `go` (Option A narrow) | **Completo / handoff para `004`** | `.specify/assessments/ressoa-mobile-app/decision.md` |

**Percorrido:** 3 de 4 ideias com `go`; 1 sem progresso além de intake.

### 1.2 Delivery — `specs/`

| # | Spec | `spec` | `plan` | `tasks` | `checklists` | `contracts` | Código / testes | Verif. | Estado |
|---|---|---|---|---|---|---|---|---|---|
| 001 | `android-media-shell` | ✓ | ✓ | ✓ | ✓ | ✓ | `tests/`; `VERIFICATION.md` | `gate-8-shell PASS`; `gate-8-full DEFERRED (owner=synthesis)` | **Converge / verificado** |
| 002 | `fr-024-synthesis-adapter` | ✓ | ✓ | ✓ (276l) | ✓ (8/8, batch resolvido) | ✓ | `crates/adapter-synthesis/`; V3 attestation (110 pass, `clippy` zero) | `SC-044 DEFERRED`; `V3_ATTESTATION_2026-10-10.md` | **Implementado / converge parcial (defer constitucional)** |
| 003 | `reference-device` | ✓ | ✓ | ✓ (140l) | ✓ | ✓ | apenas referências em `crates/player`; **sem** `tests/` próprio | — | **Plan/Tasks presentes; implementação pendente** |
| 004 | `ressoa-dogfood-build` | ✓ | ✓ | ✓ | ✓ | ✓ | `verification/` | `verify.sh` **GREEN** `9501bba` (Phase 17, 2026-10-22) | **Converge / verificado** |
| 005 | `adopt-design-sona` | ✓ | ✓ (post-clarify 4/5) | ✓ (276l) | ✓ | ✓ | fonte reflete design-dir; **sem** `tests/` próprio do spec | — | **Clarify→Plan completo; `tasks` pendente** (`STATE.md` 2026-10-24) |

**Percorrido:**
- 2 specs totalmente convergidos com evidência de verificação (`001`, `004`).
- 1 spec implementado com attestation V3 e batched clarifications resolvidas, mas com gate constitucional aberto (`002` → `SC-044`).
- 1 spec em ponto de transição exato (`005` → próximo é `/speckit.tasks`).
- 1 spec em tarefas sem implementação (`003`).

### 1.3 Triage — `.specify/bugs/`

- **Estado:** diretório **ausente**; extensão instalada (`extension.yml` v1.0.0).
- **Percorrido:** 0 bugs registrados; fluxo `assess → fix → test` inativo.

---

## 2. O que ainda falta (por fluxo e por spec)

### Assessment (discovery)

- [`directory-reading`]: completar `research` → `define` → `shape` → `decide`, ou documentar `kill` com razão.
- [`fr-024`]: fechar `SC-009` benchmark latência e validar fronteiras `Piper`; depois `SC-044` (gate-8-full) só passa quando 002 confirmar.
- [`design-son-adoption` / `ressoa-mobile-app`]: já entregues; nenhum esforço adicional necessário até novos requisitos surgirem.

### Spec / Delivery

- **005 (ativo):** `/speckit.tasks` → implementação das tarefas do plano → `/speckit.implement` → testes do spec → `/speckit.converge`.
- **002 (deferido):** fechar `SC-009` + `SC-044`; depois atualizar attestation para V4 com `gate-8-full PASS`.
- **003:** executado `/speckit.tasks`; implementar referências de referência-device; gerar `tests/` próprio; attestation / verificação.
- **001:** manter `gate-8-full` como `DEFERRED` até 002; nenhuma ação até então.

### Bug (triage)

- Nenhum bug registrado; se aparecer (ex.: falha de `verify.sh` em novo branch, regressão de `SC-051`, erro de `clippy` after upgrade), iniciar com `/speckit.bug.assess` e criar `.specify/bugs/<slug>/`.

---

## 3. Atualização baseada em pesquisa (fontes externas)

- **Spec Kit docs (github.github.io):** ciclo SDD confirmado como `specify → clarify → plan → tasks → analyze → implement → converge`; assess como discovery anterior; bug como triagem separada.
- **Constituição v1.12.0:** gate-8 splitado (`shell` vs `full`) documentado; `SC-035` (Kotlin puro espelho); `SC-051` preservado no adapter; `SC-007` (freeze de animações) integrado no design-sona.
- **Estado do repo (2026-10-26):** `005` é o único branch ativo com `clarify` realizado; `004` é baseline de conclusão; `002` tem `clippy` zero e `V3` attestation; `001` tem `VERIFICATION.md`; `003` ainda sem testes próprios.

---

## 4. Próximo passo recomendado (80/20 — maior avanço, menor esforço)

> **Resposta direta:** Nem `/speckit.assess.intake` (baixo impacto, bloqueado em idea não ativa), nem `/speckit.specify` (já feito para 005; novos specs não são prioridade), nem `/speckit.bug.assess` (zero bugs, sem sinal de regressão ativa).

### 80/20: `/speckit.tasks` para `005-adopt-design-sona`

**Por que:**
1. `STATE.md` (2026-10-24) diz explicitamente “NEXT: `/speckit.tasks`” — é o passo designado pelo próprio processo.
2. `005` é o branch ativo (`branch 005-adopt-design-sona`, `226c06c`); todos os clarificações (E1–E3, D1–D5) já foram aceitos pelo mantenedor.
3. O plano (`plan.md`) já existe com `data-model.md`, contratos (`status-surface`, `earcon-mapping`, `surface-audit`), `checklists/` (16/16 validado), e `quickstart.md`; não há bloqueio de especificação.
4. A implementação de tarefas do design-sona afeta o driving surface (`status_view.html`, `MediaMirror.kt`, tokens) — impacto amplo, esforço moderado (já há código de referência em `DESIGN-SONA/`, `tokens.md`, `components.md`).
5. Fechar 005 libera recursos para fechar `SC-044` em 002 e depois `003`, criando uma cadeia de convergência.

**Execução sugerida:**
- Executar `/speckit.tasks` com input = resumé de `005-adopt-design-sona/plan.md` + `tasks.md` existente.
- Priorizar tarefas que entregam `SC-001` (papel de focus), `SC-007` (freeze de animações) e `SC-002` (disposição haptic) — já validadas no clarify.
- Após tarefas, ir para `/speckit.implement` e gerar `tests/` próprio do spec (hoje ausente), depois `/speckit.converge` com attestation.

### Alternativa de alto impacto / baixo esforço (se 005 estiver bloqueado por outro razão):
- **Fechamento de `SC-009` + `SC-044` via `/speckit.clarify` / benchmark** para `002`. Mas `002` já tem batches resolvidos; isso é mais esforço de pesquisa de latência do que de espec.
- **Kill ou finalize `directory-reading`** se não for necessário: baixa prioridade.

---

## 5. Marcas de progresso (checklist de convergência)

- [x] Constituição v1.12.0 ratificada; gates 1–8 definidos; `SC-035`/`SC-051`/`SC-009`/`SC-044` documentados.
- [x] `004-ressoa-dogfood-build`: verificado verde (`9501bba`).
- [x] `001-android-media-shell`: `VERIFICATION.md` + `gate-8-shell PASS`.
- [x] `002-fr-024-synthesis-adapter`: `V3_ATTESTATION` + `CLARIFICATION_BATCH_RESOLVED` + `crates/adapter-synthesis/` com 110 pass.
- [x] `005-adopt-design-sona`: clarificações E1–E3/D1–D5 resolvidas; plano pronto; `NEXT: tasks`.
- [x] `003-reference-device`: artigos presentes; implementação pendente.
- [ ] `005`: `/speckit.tasks` executado (próximo passo 80/20).
- [ ] `002`: `SC-044` fechado → `gate-8-full PASS`.
- [ ] `003`: `/speckit.tasks` executado; `tests/` gerado; attestation.
- [ ] `.specify/bugs/`: criado se necessário (atualmente não há demanda).
- [ ] `directory-reading`: `decision.md` (kill / go / clarify). **Baixa prioridade.**

---

*Atualizado: 2026-10-26 · Base: `.specify/memory/STATE.md` (resume points 005 run 4/5, 004 Phase 17), `specs/*/VERIFICATION.md`, `V3_ATTESTATION_2026-10-10.md`, `constitution.md` §1309.*
