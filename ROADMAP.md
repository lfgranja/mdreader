# Roadmap — mdreader / Spec-Driven Development

> **Constituição:** `.specify/memory/constitution.md` v1.12.0  
> **Fluxos:** `/speckit.assess.*` (discover) → `/speckit.specify` → `/speckit.clarify` → `/speckit.plan` → `/speckit.tasks` → `/speckit.implement` → `/speckit.converge` (deliver) → `/speckit.bug.assess` → `/speckit.bug.fix` → `/speckit.bug.test` (triage)  
> **Objetivo:** `OBJETIVO.md` — leitor de documentos Markdown com áudio/síntese de voz, Rust como única autoridade de posição/transporte/buffering/rate, Kotlin apenas espelho estrutural (SC-035), Tauri 2.12.1, CPAL 0.18.2 + AAudio, mono f32 @ 22050 Hz, sherpa-onnx 1.13.8 + Piper pt-BR (`offline_default`).

*Atualizado: 2026-10-31 · Base: `AGENTS.md`, `.specify/memory/STATE.md` (branch `pr/005-sc001-sc002-sc007-tests`, HEAD `8a4dd32e`), `specs/*/`, `.specify/assessments/*`, `.specify/bugs/*`, `OBJETIVO.md`, `ROADMAP.md` anterior.*

---

## 1. Mapa do caminho percorrido (evidência direta)

### 1.1 Discovery — `.specify/assessments/`

| Slug | Estágios | Veredito | Status | Evidência |
|---|---|---|---|---|
| `directory-reading` | `intake.md` apenas | — | **Bloqueado** (intake) | `.specify/assessments/directory-reading/intake.md` |
| `design-son-adoption` | 5/5 (`intake→research→problem→concept→decision`) | `go` → `005` | **Completo / handoff ativo** | `decision.md` (0 clarifications pendentes) |
| `fr-024-synthesis-adapter` | 5/5 | `go` condicional | **Completo; clarificações transportadas para `/speckit.clarify`** | `decision.md`: 5 `[NEEDS CLARIFICATION]` (`SC-009`, fronteiras `Piper`) |
| `ref-device-emulator` | 5/5 (+ `6-gaps-research`) | `needs-clarification` (`go` permitido após G1–G6 2026-10-29; `VERIF-EMULATOR.md` existente) | **Completo / `VERIF-EMULATOR` definido; `go` após clarificação** | `decision.md` (atualizado 2026-10-29) |

**Percorrido:** 4 de 5 ideias com `go`/condicional; 1 bloqueado em intake (`directory-reading`). Nenhum assessment pendente de decisão.

### 1.2 Delivery — `specs/`

| # | Spec | spec | plan | tasks | checklists | contracts | Código / testes | Verif. | Estado |
|---|---|---|---|---|---|---|---|---|---|
| 001 | `android-media-shell` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ `tests/`, `crates/*` | `gate-8-shell PASS`; `gate-8-full DEFERRED (owner=synthesis)` | **Converge / verificado (núcleo)** |
| 002 | `fr-024-synthesis-adapter` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ `crates/adapter-synthesis/`; V3 attestation (110 pass, `clippy` zero) | `SC-044 DEFERRED`; `V3_ATTESTATION_2026-10-10.md` | **Implementado / converge parcial (defer constitucional)** |
| 003 | `reference-device` | ✓ | ✓ | ✓ (140l) | ✓ | ✓ | Apenas refs/docs; emulador viável p/ estrutural; físico obrigatório p/ `gate-8-full` | `VERIF-EMULATOR` + `gate-8-full` DEFERRED | **Plan/Tasks presentes; implementação pendente** |
| 004 | `ressoa-dogfood-build` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ `verification/` | `verify.sh` GREEN `9501bba` (Phase 17, 2026-10-22) | **Converge / verificado** |
| 005 | `adopt-design-sona` | ✓ | ✓ (post-clarify) | **✗ pendente** | ✓ (16/16) | ✓ | Fonte reflete design-dir; **sem** `tests/` próprio do spec (testes SC-001/SC-002/SC-007 adicionados em branch `pr/005-sc001-sc002-sc007-tests` — `8a4dd32e`) | — | **Clarify→Plan completo; `/speckit.tasks` pendente** (`STATE.md`: "NEXT: `/speckit.tasks"`). Branch atual: `pr/005-sc001-sc002-sc007-tests` com testes property-based adicionados. |
| 006 | `lente-sinestesica-completa` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | converge completo | **Concluído** |
| 007 | `sona-complete-design` | ✓ | ✓ | **Parcial/indefinido** | ✓ | ✓ | — | — | **Specify/plan feito; `/speckit.tasks` e implementação pendentes** |
| 008 | `ref-device-emulator` | ✓ | ✓ | ✓ | ✓ | ✓ | Apenas docs/estrutural | converge completo | **Concluído (camada estrutural)** |

**Percorrido:**
- 4 specs totalmente convergidos com evidência de verificação (`001`, `004`, `006`, `008`).
- 1 spec implementado com attestation V3 e clarifications batched resolvidas, mas gate constitucional aberto (`002` → `SC-044`).
- 1 spec em ponto de transição exato (`005` → próximo é `/speckit.tasks`); testes para SC-001/SC-002/SC-007 foram adicionados recentemente em branch separado.
- 1 spec com plan mas sem tasks/implement claro (`003`).
- 1 spec com spec/plan mas sem tasks (`007`).

### 1.3 Triage — `.specify/bugs/`

| Slug | assess | fix | test | Status | Resumo |
|---|---|---|---|---|---|
| `corrupt-design-asset` | ✓ | ✗ | ✗ | Assess concluído | Asset `ui-proposta-2-lente-sinestesica.png` contém imagem de Rust tutorial, não mockup SONA. Afeta verificação visual de 005/006/007. |
| `corrupt-ui-proposta-asset` | — | — | — | **Pasta NÃO ENCONTRADA / assessment.md ausente** | Verificado: não existe `.specify/bugs/corrupt-ui-proposta-asset/` com assessment.md válido. |
| `sona-visual-core-fix` | ✓ | ✓ (parcial) | ✓ (parcial) | Fix parcial | Header quebrado, waveform vazio, botões pequenos persistem (`sona_final_updated.png`). Visual core ainda quebrado. |
| `sona-visual-fix` | ✓ | ✗ | ✗ | Assess concluído | Pasta vazia além do assessment. |
| `sona-waveform-missing-in-screenshot` | ✓ (re-evaluated) | ✓ (aplicado 2026-10-28) | ✓ (parcial) | Assess + fix aplicado; funcionalidade completa inconclusiva (`AppState` precisa `kind='determinate'` inicial; reprodução ativa não confirmada) |

**Percorrido:** 5/5 assessados (válidos). Apenas 1 entrou em fix/test parcial. Nenhum bug fechado com `bug.test` green. Nenhuma regressão ativa confirmada.

---

## 2. Quanto já percorremos vs quanto falta

### 2.1 Por fluxo

| Fluxo | % percorrido (evidência) | Status | Comentário |
|---|---|---|---|
| Idea Assessment (`/speckit.assess.*`) | ~90% | **Concluído na maior parte** | 4/5 assessments completos com decisão; apenas `directory-reading` em intake. Decisões materializadas na constituição/contratos. |
| Spec-Driven Development (núcleo 001) | ~95% | **Consolidado** | `001` converge com gates 1-7 PASS, `gate-8-shell` PASS. Único gap: `gate-8-full` DEFERRED aguardando síntese. |
| Spec-Driven Development (Design SONA 005/007) | ~40-60% | **Em andamento** | 005 tem spec/plan/checklists prontos, mas falta `/speckit.tasks` + implementação + testes próprios; 007 tem spec/plan, falta tasks/implement. Testes SC-001/SC-002/SC-007 recém-adicionados em branch `pr/005-sc001-sc002-sc007-tests`. |
| Spec-Driven Development (Síntese 002) | ~70-80% | **Implementado com defer** | Código em `crates/adapter-synthesis/`, V3 attestation OK, mas `SC-044`/`gate-8-full` pendentes. Tarefas T045/T046/T048 abertas. |
| Spec-Driven Development (Device/Emulador 003/008) | ~30-50% | **Parcial** | 008 concluído estruturalmente; 003 tem plan/tasks mas implementação pendente; build Android configurado (gate 9). |
| Bug Fixing | ~15-25% | **Ativo/Inicial** | Bugs visuais SONA assessados, fixes parciais apenas em 1 caso; nenhum fechado com `bug.test` green. |

### 2.2 O que falta (detalhado)

#### Assessment
- [`directory-reading`]: Completar `research → define → shape → decide` ou `kill` com justificativa. **Baixa prioridade** (não bloqueia 005).
- [`fr-024`]: Fechar `SC-009` (benchmark latência) e validar fronteiras `Piper` → necessário para habilitar `SC-044`/`gate-8-full`.

#### Specs / Implementação
- **005 (adopt-design-sona) — CRÍTICO PRÓXIMO**: Executar `/speckit.tasks` → `/speckit.implement` → gerar testes próprios do spec → `/speckit.converge`. Branch atual já tem testes SC-001/SC-002/SC-007 em `pr/005-sc001-sc002-sc007-tests` (`8a4dd32e`); integrar/validar com plano completo. Fechar bugs visuais relacionados.
- **002 (fr-024-synthesis-adapter) — BLOQUEIO FUNCIONAL**: Fechar `SC-009`, completar T045/T046/T048 → habilitar `SC-044` → atualizar attestation para V4 → fazer `gate-8-full` PASS. Único gap real entre "núcleo funciona" e "pipeline completo funciona".
- **003 (reference-device)**: Implementar validação device/emulador; executar testes próprios; gerar attestation. Emulador OK para estrutural; físico obrigatório para `gate-8-full`.
- **007 (sona-complete-design)**: Criar `/speckit.tasks` → `/speckit.implement` (RMS instrumentation, canvas waveform, repeat button, expansão tokens jade/celeste, glow/neon).
- **001/004/006/008**: Sem ações pendentes de converge; manter `gate-8-full` DEFERRED até 002.

#### Bugs
- `corrupt-design-asset`: `bug.fix` + `bug.test` (substituir asset corrupto; considerar design-asset integrity gate).
- `sona-visual-core-fix`: Completar fix (header/waveform/botões) + `bug.test`.
- `sona-waveform-missing-in-screenshot`: Investigar/publicar `channel_waveform` com `kind='determinate'` e reprodução ativa; rebuild + `bug.test`.
- `sona-visual-fix`, `corrupt-ui-proposta-asset`: Baixa prioridade / sem artefatos completos.

---

## 3. Próximo passo recomendado (80/20 — maior avanço, menor esforço)

### Recomendação: `/speckit.tasks` para `005-adopt-design-sona` (MAIOR AVANÇO / MENOR ESFORÇO)

**Por que é 80/20:**
1. **Máxima prontidão**: `spec.md`, `plan.md`, `checklists/` (16/16), `contracts/` (status-surface, earcon-mapping, surface-audit), `data-model.md`, `quickstart.md` todos presentes. Zero `NEEDS CLARIFICATION` bloqueando.
2. **Contexto ativo**: Branch `pr/005-sc001-sc002-sc007-tests` já tem testes property-based para SC-001/SC-002/SC-007 (`8a4dd32e`). STATE.md indica "NEXT: `/speckit.tasks`". Muito trabalho já iniciado.
3. **Alto impacto transversal**: Afeta superfície principal (StatusView/MediaMirror/earcons/tokens/contraste/movimento). Fechar 005 libera caminho para convergir Design SONA e reduz ruído dos bugs visuais atuais.
4. **Cadeia destravadora**: 005 está na frente da consolidação visual; ao mesmo tempo 002 depende de validação, mas 005 é mais direto de "tasks → implement" dado o plano já maduro.
5. **Menor risco técnico relativo**: Não requer benchmark físico complexo como SC-009; majoritariamente código/config/tokens/contratos já mapeados.

**Ação exata (comando/fluxo):**
```bash
# Garantir branch correto
git -C /home/luis/development/mdreader switch pr/005-sc001-sc002-sc007-tests  # ou 005-adopt-design-sona

# Gerar/atualizar tasks
/speckit.tasks  # com base em specs/005-adopt-design-sona/{spec,plan,checklists,contracts,data-model,quickstart}.md

# Implementar por waves (TDD RED→GREEN), commitar via scripts/commit-hook.sh
/speckit.implement

# Validar e convergir
/speckit.converge
```

**Foco imediato nas tarefas geradas:** Priorizar SC-001 (papel de focus), SC-002 (haptic side-channel conforme decisão E3), SC-007 (prefers-reduced-motion/freeze). Validar integração com testes já existentes em `pr/005-sc001-sc002-sc007-tests`.

### Alternativa (maior impacto a longo prazo, maior esforço):
**`/speckit.clarify`/benchmark para 002 → fechar `SC-009` e habilitar `gate-8-full`.** Impacto máximo (desbloqueia pipeline completo), mas exige consentimento UI, 2º engine real, benchmark em dispositivo físico e medições precisas. Muito mais custoso que gerar tasks para 005 neste momento.

**Veredito 80/20:** **`/speckit.tasks` em `005-adopt-design-sona` (na branch `pr/005-sc001-sc002-sc07-tests` ou `005-adopt-design-sona`) é o próximo passo com melhor relação avanço/esforço.**

---

## 4. Marcas de progresso (checklist de convergência)

- [x] Constituição v1.12.0 ratificada; gates 1–8 definidos; `SC-035`/`SC-051`/`SC-009`/`SC-044` documentados.
- [x] `004-ressoa-dogfood-build`: verificado verde (`9501bba`, Phase 17, 2026-10-22).
- [x] `001-android-media-shell`: `VERIFICATION.md` + `gate-8-shell PASS`; `gate-8-full DEFERRED` (owner=synthesis).
- [x] `002-fr-024-synthesis-adapter`: `V3_ATTESTATION` (110 pass, `clippy` zero) + `CLARIFICATION_BATCH_RESOLVED`; `SC-044 DEFERRED`. Pendentes: T045, T046, T048.
- [x] `005-adopt-design-sona`: clarificações resolvidas (E1–E3/D1–D5); spec/plan/checklists/contracts prontos; testes SC-001/SC-002/SC-007 adicionados em `pr/005-sc001-sc002-sc007-tests` (`8a4dd32e`). **Pendente: `/speckit.tasks` → `/speckit.implement` → testes próprios → `/speckit.converge`**.
- [x] `006-lente-sinestesica-completa`: converge completo.
- [x] `008-ref-device-emulator`: concluído estruturalmente.
- [ ] `005`: `/speckit.tasks` executado → `/speckit.implement` → `tests/` próprio validados → `/speckit.converge` (**PRIORITÁRIO 80/20**).
- [ ] `002`: fechar `SC-009` → completar T045/T046/T048 → `SC-044` → attestation V4 → `gate-8-full PASS`.
- [ ] `003`: implementação device/emulador → `tests/` próprio → attestation.
- [ ] `007`: `/speckit.tasks` → `/speckit.implement`.
- [ ] `.specify/bugs/`: `corrupt-design-asset`, `sona-visual-core-fix`, `sona-waveform-missing-in-screenshot` precisam `fix`/`test`; fechar bugs bloqueadores para 005 converge.
- [ ] `directory-reading`: decidir go/kill/clarify (baixa prioridade).

---

*Atualizado: 2026-10-31 · Branch atual: `pr/005-sc001-sc002-sc007-tests` (HEAD `8a4dd32e`). Baseado em evidência direta de arquivos, STATE.md, git history, gates e pesquisa web (Tauri 2.12.x, CPAL 0.18.2 AAudio, sherpa-onnx 1.13.8).*
