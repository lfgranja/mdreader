# Roadmap — mdreader / Spec-Driven Development

> **Constituição:** `.specify/memory/constitution.md` v2.0.0 (amended 2026-10-28 — expansão de design tokens §VII, elementos visuais suplementares §VIII)  
> **Fluxos:** `/speckit.assess.*` (discover) → `/speckit.specify` → `/speckit.clarify` → `/speckit.plan` → `/speckit.tasks` → `/speckit.implement` → `/speckit.converge` (deliver) → `/speckit.bug.assess` → `/speckit.bug.fix` → `/speckit.bug.test` (triage)  
> **Objetivo:** `OBJETIVO.md` — leitor de documentos Markdown com áudio/síntese de voz, Rust como única autoridade de posição/transporte/buffering/rate, Kotlin apenas espelho estrutural (SC-035), Tauri 2.12.1, CPAL 0.18.2 + AAudio, mono f32 @ 22050 Hz, sherpa-onnx 1.13.8 + Piper pt-BR (`offline_default`).

*Atualizado: 2026-11-01 · Base: `AGENTS.md`, `specs/*/`, `.specify/assessments/*`, `.specify/bugs/*`, `OBJETIVO.md`, git HEAD `26a2ce1d` on branch `dev`.*

---

## 1. Mapa do caminho percorrido (evidência direta)

### 1.1 Discovery — `.specify/assessments/`

| Slug | Estágios | Veredito | Status | Evidência |
|---|---|---|---|---|
| `directory-reading` | `intake.md` apenas | — | **Bloqueado** (intake) | `.specify/assessments/directory-reading/intake.md` |
| `design-son-adoption` | 5/5 (`intake→research→problem→concept→decision`) | `go` → `005`/`006`/`007` | **Completo / handoff ativo** | `decision.md` (0 clarifications pendentes) |
| `fr-024-synthesis-adapter` | 5/5 | `go` condicional | **Completo; clarificações transportadas para `/speckit.clarify`** | `decision.md`: 5 `[NEEDS CLARIFICATION]` (`SC-009`, fronteiras `Piper`) |
| `ref-device-emulator` | 5/5 (+ `6-gaps-research`) | `needs-clarification` (`go` após G1–G6 2026-10-29; `VERIF-EMULATOR.md` existente) | **Completo / `VERIF-EMULATOR` definido; `go` após clarificação** | `decision.md` (atualizado 2026-10-29) |
| `ressoa-mobile-app` | 5/5 | Decidido | **Completo** | `decision.md` presente (sem promoção para spec `00x` no estado atual) |

**Percorrido:** 5/5 ideias com avaliação completa. Apenas `directory-reading` permanece em intake. Nenhum assessment pendente de decisão.

### 1.2 Delivery — `specs/`

| # | Spec | spec | plan | tasks | checklists | contracts | Código / testes | Verif. | Estado |
|---|---|---|---|---|---|---|---|---|---|
| 001 | `android-media-shell` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ `tests/`, `crates/*` | `gate-8-shell PASS`; `gate-8-full DEFERRED (owner=synthesis)` | **Converge / verificado (núcleo)** |
| 002 | `fr-024-synthesis-adapter` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ `crates/adapter-synthesis/`; V3 attestation (110 pass, `clippy` zero) | `SC-044 DEFERRED`; `V3_ATTESTATION_2026-10-10.md` | **Implementado / converge parcial (defer constitucional)** |
| 003 | `reference-device` | ✓ | ✓ | ✓ (140l) | ✓ | ✓ | Apenas refs/docs; emulador viável p/ estrutural; físico obrigatório p/ `gate-8-full` | `VERIF-EMULATOR` + `gate-8-full` DEFERRED | **Plan/Tasks presentes; implementação pendente** |
| 004 | `ressoa-dogfood-build` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ `verification/` | `verify.sh` GREEN `9501bba` (Phase 17, 2026-10-22) | **Converge / verificado** |
| 005 | `adopt-design-sona` | ✓ | ✓ (post-clarify) | ✓ | ✓ (16/16) | ✓ | `surface_audit` integrado; testes property-based SC-001/SC-002/SC-007 em `pr/005-sc001-sc002-sc007-tests` (merge PR #145/#146 context) | Host gates pendentes de execução pós-implementação | **Tasks gerados / pronto para `/speckit.implement`** |
| 006 | `lente-sinestesica-completa` | ✓ | ✓ | ✓ (10/10) | ✓ | ✓ | ✓ | converge completo | **Concluído** |
| 007 | `sona-complete-design` | ✓ | ✓ | ✗ (0/27) | ✓ | ✓ | Parcial (PR #146 mergeou reduções/ajustes US1) | — | **Specify/plan feito; `/speckit.tasks` pendente OU já existe mas não marcado; implementação em andamento** |
| 008 | `ref-device-emulator` | ✓ | ✓ | ✓ | ✓ | ✓ | Apenas docs/estrutural | converge completo | **Concluído (camada estrutural)** |

**Percorrido:** 4 specs totalmente convergidos (`001`, `004`, `006`, `008`), 1 implementado com defer (`002`), 1 pronto para implementar (`005`), 1 sem tasks marcadas mas com implementação iniciada (`007`), 1 com plan/tasks mas implementação pendente (`003`).

### 1.3 Triage — `.specify/bugs/`

| Slug | assess | fix | test | Status | Resumo |
|---|---|---|---|---|---|
| `corrupt-design-asset` | ✓ | ✗ | ✗ | Assess concluído | Asset `ui-proposta-2-lente-sinestesica.png` contém imagem de Rust tutorial, não mockup SONA. Afeta verificação visual de 005/006/007. |
| `corrupt-ui-proposta-asset` | — | — | — | **Sem artefatos completos** | Pasta/assessment incompleto no estado atual. |
| `sona-visual-core-fix` | ✓ | ✓ (parcial) | ✓ (parcial) | Fix parcial | Header quebrado, waveform vazio, botões pequenos persistem. Visual core ainda quebrado. |
| `sona-visual-fix` | ✓ | ✗ | ✗ | Assess concluído | Sem progresso de fix/test evidente. |
| `sona-waveform-missing-in-screenshot` | ✓ (re-evaluated) | ✓ (aplicado 2026-10-28) | ✓ (parcial) | Assess + fix aplicado; funcionalidade completa inconclusiva (`AppState` precisa `kind='determinate'` inicial; reprodução ativa não confirmada) |

**Percorrido:** 4–5 assessados. Apenas correções parciais; **nenhum bug fechado com `bug.test` green**. Não há regressões confirmadas que quebrem gates 1–7.

---

## 2. Quanto já percorremos vs quanto falta

### 2.1 Por fluxo

| Fluxo | % percorrido (evidência) | Status | Comentário |
|---|---|---|---|
| Idea Assessment (`/speckit.assess.*`) | ~95% | **Concluído na maior parte** | Maioria completa com decisão; `directory-reading` apenas em intake. Decisões refletidas na constituição v2.0.0 e contratos. |
| Spec-Driven Development (núcleo 001) | ~95% | **Consolidado** | `001` converge com gates 1-7 PASS, `gate-8-shell` PASS. Único gap: `gate-8-full` DEFERRED aguardando síntese. |
| Spec-Driven Development (Design SONA 005/007) | ~70–80% | **Pronto para implementar / em andamento** | `005` tem spec/plan/tasks/checklists/contracts prontos; testes SC-001/SC-002/SC-007 adicionados. **Próximo natural: `/speckit.implement` em 005**. `007` tem spec/plan, tasks existem mas não marcadas (0/27 done); PR #146 mergeado com ajustes US1; 36 issues abertas (T076-T121) guiam implementação. |
| Spec-Driven Development (Síntese 002) | ~70–80% | **Implementado com defer** | Código em `crates/adapter-synthesis/`, V3 attestation OK, mas `SC-044`/`gate-8-full` pendentes. `T045/T046/T048` e `SC-009` em aberto. |
| Spec-Driven Development (Device/Emulador 003/008) | ~40–60% | **Parcial** | `008` concluído estruturalmente; `003` tem plan/tasks mas implementação/attestation pendentes; build Android configurado (gate 9). |
| Bug Fixing | ~15–25% | **Ativo/Inicial** | Bugs visuais SONA assessados, fixes parciais apenas em casos isolados; nenhum fechado com `bug.test` green. |

### 2.2 O que falta (detalhado)

#### Assessment
- [`directory-reading`]: Completar `research → define → shape → decide` ou `kill` com justificativa. **Baixa prioridade**.
- [`fr-024`]: Fechar `SC-009` (benchmark latência) e validar fronteiras `Piper` → necessário para habilitar `SC-044`/`gate-8-full`.

#### Specs / Implementação
- **005 (adopt-design-sona) — ALTA PRIORIDADE**: Executar `/speckit.implement` na branch atual/contexto adequado. Integrar/validar testes SC-001/SC-002/SC-007 já existentes, fazer `C-H1..C-H7` verdes, completar tarefas pendentes, preparar trials device, depois `/speckit.converge`.
- **007 (sona-complete-design) — EM ANDAMENTO**: Completar implementação conforme issues T076-T121 (RMS instrumentation/exposição, canvas waveform 5-layer, repeat button, aria/progress, glassmorphism/glow/decorative wave, title binding, tokens jade/celeste). Rodar testes US1/US2/US3/US4 + surface-audit; finalizar Fases 7-8. PRs incrementais já sendo feitos.
- **002 (fr-024-synthesis-adapter) — BLOQUEIO FUNCIONAL**: Fechar `SC-009`, completar `T045/T046/T048` → habilitar `SC-044` → atualizar attestation para V4 → fazer `gate-8-full` PASS. Único gap real entre "núcleo funciona" e "pipeline completo funciona".
- **003 (reference-device)**: Implementar validação device/emulador; executar testes próprios; gerar attestation. Emulador OK para estrutural; físico obrigatório para `gate-8-full`.
- **001/004/006/008**: Sem ações pendentes de converge; manter `gate-8-full` DEFERRED até 002.

#### Bugs
- `corrupt-design-asset`: `bug.fix` + `bug.test` (substituir asset corrupto; considerar *design-asset integrity gate*).
- `sona-visual-core-fix`: Completar fix (header/waveform/botões) + `bug.test`.
- `sona-waveform-missing-in-screenshot`: Investigar/publicar `channel_waveform` com `kind='determinate'` e reprodução ativa; rebuild + `bug.test`.
- `sona-visual-fix`, `corrupt-ui-proposta-asset`: Baixa prioridade / sem artefatos completos.

---

## 3. Próximo passo recomendado (80/20 — maior avanço, menor esforço)

### Recomendação: `/speckit.implement` para `005-adopt-design-sona` (MAIOR AVANÇO / MENOR ESFORÇO)

**Por que é 80/20:**
1. **Máxima prontidão**: `spec.md`, `plan.md`, `tasks.md`, `checklists/` (16/16), `contracts/` (status-surface, earcon-mapping, surface-audit), `data-model.md`, `quickstart.md` todos presentes. Zero `NEEDS CLARIFICATION` bloqueando.
2. **Contexto ativo**: Testes property-based para `SC-001/SC-002/SC-007` já existem/foram adicionados; infraestrutura `surface_audit` mergeada. Pronto para implementação.
3. **Alto impacto transversal**: Afeta superfície principal (StatusView/MediaMirror/earcons/tokens/contraste/movimento/reduced-motion). Fechar 005 libera caminho para consolidar Design SONA e reduz ruído dos bugs visuais atuais.
4. **Cadeia destravadora**: 005 está à frente da consolidação visual; `002` pode avançar em paralelo.
5. **Menor risco técnico relativo**: Majoritariamente host-verificável; não requer benchmark físico complexo como `SC-009`; já mapeado com contratos e auditorias.

**Ação exata:**
```bash
# Garantir branch correto (ou trabalhar na branch apropriada)
git -C /home/luis/development/mdreader switch dev  # ou branch dedicada

# Implementar conforme tasks.md
/speckit.implement

# Validar host
cargo test
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
scripts/verify.sh

# Convergir
/speckit.converge
```

**Foco imediato:** Priorizar fases com testes SC-001/SC-002/SC-007 existentes; fazer `C-H1..C-H7` verdes; validar identidade de bundle (`src-tauri/src/status_view.html` ↔ `dist/index.html`); confirmar políticas de burst/latência de earcon.

### Alternativa (maior impacto a longo prazo, maior esforço):
Fechar `SC-009`/`002` para habilitar `gate-8-full` — impacto máximo, mas exige consentimento UI, segundo engine real, benchmark em dispositivo físico e medições precisas. Melhor executar em paralelo ou após iniciar 005.

**Veredito 80/20:** **`/speckit.implement` em `005-adopt-design-sona` é o próximo passo com melhor relação avanço/esforço.**

---

## 4. Marcas de progresso (checklist de convergência)

- [x] Constituição v2.0.0 ratificada/amendada (expansão tokens §VII, elementos suplementares §VIII); gates 1–8 definidos; `SC-035`/`SC-051`/`SC-009`/`SC-044` documentados.
- [x] `004-ressoa-dogfood-build`: verificado verde (`9501bba`, Phase 17, 2026-10-22).
- [x] `001-android-media-shell`: `gate-8-shell PASS`; `gate-8-full DEFERRED` (owner=synthesis).
- [x] `002-fr-024-synthesis-adapter`: `V3_ATTESTATION` (110 pass, `clippy` zero) + `CLARIFICATION_BATCH_RESOLVED`; `SC-044 DEFERRED`. Pendentes: `T045`, `T046`, `T048`, `SC-009`.
- [x] `005-adopt-design-sona`: clarificações resolvidas (E1–E3/D1–D5); spec/plan/tasks/checklists/contracts prontos; `surface_audit` integrado; testes SC-001/SC-002/SC-007 adicionados. **Pronto para `/speckit.implement`**.
- [x] `006-lente-sinestesica-completa`: converge completo.
- [x] `008-ref-device-emulator`: concluído estruturalmente (`VERIF-EMULATOR`).
- [ ] `005`: `/speckit.implement` → integrar testes existentes → fazer `C-H1..C-H7`/gates host verdes → `/speckit.converge` (**PRIORITÁRIO 80/20**).
- [ ] `007`: Implementação em andamento (36 issues abertas T076-T121); completar US1-US4 + verificação/attestation.
- [ ] `002`: fechar `SC-009` → completar `T045/T046/T048` → `SC-044` → attestation V4 → `gate-8-full PASS`.
- [ ] `003`: implementação device/emulador → `tests/` próprio → attestation.
- [ ] `.specify/bugs/`: `corrupt-design-asset`, `sona-visual-core-fix`, `sona-waveform-missing-in-screenshot` precisam `bug.fix`/`bug.test`.
- [ ] `directory-reading`: decidir go/kill/clarify (baixa prioridade).

---

*Atualizado: 2026-11-01 · Branch atual: `dev` (HEAD `26a2ce1d`). Baseado em evidência direta de arquivos, git history, gates, constitution v2.0.0.*
