# Roadmap — mdreader / Spec-Driven Development

> **Constituição:** `.specify/memory/constitution.md` v2.0.0 (amended 2026-10-28 — expansão de design tokens §VII, elementos visuais suplementares §VIII)  
> **Fluxos:** `/speckit.assess.*` (discover) → `/speckit.specify` → `/speckit.clarify` → `/speckit.plan` → `/speckit.checklist` → `/speckit.tasks` → `/speckit.analyze` → `/speckit.implement` → `/speckit.converge` (deliver) → `/speckit.bug.assess` → `/speckit.bug.fix` → `/speckit.bug.test` (triage)  
> **Objetivo:** `OBJETIVO.md` — leitor de documentos Markdown com áudio/síntese de voz, Rust como única autoridade de posição/transporte/buffering/rate, Kotlin apenas espelho estrutural (SC-035), Tauri 2.12.1, CPAL 0.18.2 + AAudio, mono f32 @ 22050 Hz, sherpa-onnx 1.13.8 + Piper pt-BR (`offline_default`).

*Atualizado: 2026-11-01 · Base: `AGENTS.md`, `specs/*/`, `.specify/assessments/*`, `.specify/bugs/*`, `OBJETIVO.md`, git HEAD `a360a6d5` on branch `dev`.*

---

## 1. Mapa do caminho percorrido (evidência direta)

### 1.1 Discovery — `.specify/assessments/`

| Slug | Estágios | Veredito | Status | Evidência |
|---|---|---|---|---|
| `directory-reading` | `intake.md` apenas (1/5) | — | **Bloqueado / Pendente** (intake) | `.specify/assessments/directory-reading/intake.md` |
| `design-son-adoption` | 5/5 (`intake→research→problem→concept→decision`) | `go` → `005`/`006`/`007` | **Completo** | `decision.md` |
| `fr-024-synthesis-adapter` | 5/5 | `go` condicional | **Completo; clarificações transportadas para `/speckit.clarify`** | `decision.md` |
| `lente-sinestesica-completa` | 5/5 | `go` | **Completo** → gerou spec `006` | `decision.md` |
| `ref-device-emulator` | 5/5 (+ `6-gaps-research-2026-10-29.md`) | `needs-clarification` → `go` após G1–G6 (2026-10-29) | **Completo / `VERIF-EMULATOR` definido** | `decision.md` (atualizado 2026-10-29), `6-gaps-research-2026-10-29.md` |
| `ressoa-mobile-app` | 5/5 | `go` | **Completo** | `decision.md` (sem promoção para spec `00x` no estado atual) |

**Percorrido:** 5/6 ideias com avaliação completa. Apenas `directory-reading` permanece em intake (1/5 estágios). Decisões já refletidas na constituição v2.0.0 e contratos.

### 1.2 Delivery — `specs/`

| # | Spec | specify | clarify | plan | checklist | tasks | analyze | implement | converge | Código / testes | Verif. | Estado |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 001 | `android-media-shell` | ✓ | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (parcial) | ✓ `tests/`, `crates/*` | `gate-8-shell PASS`; `gate-8-full DEFERRED (owner=synthesis)` | **Convergido com defer** |
| 002 | `fr-024-synthesis-adapter` | ✓ | ✓ (batch resolvido) | ✓ | ✓ | ✓ | ✓ | ✓ (parcial) | ✗ | ✓ `crates/adapter-synthesis/`; V3 attestation (110 pass, `clippy` zero) | `SC-044 DEFERRED`; `V3_ATTESTATION_2026-10-10.md` | **Implementado / não convergido** |
| 003 | `reference-device` | ✓ | (após assess ref-device-emulator) | ✓ | ✓ | ✓ (140 linhas) | — | ✗ | ✗ | Apenas refs/docs | `VERIF-EMULATOR` + `gate-8-full` DEFERRED | **Planejado; implementação pendente** |
| 004 | `ressoa-dogfood-build` | ✓ | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ `verification/` | `verify.sh` GREEN `9501bba` (Phase 17, 2026-10-22) | **Convergido / verificado** |
| 005 | `adopt-design-sona` | ✓ | ✓ (post-clarify) | ✓ | ✓ (16/16) | ✓ (55/55) | ✓ | ✓ | ✓ | ✓ `surface_audit` integrado; testes SC-001/SC-002/SC-007 passing | Host gates 1-8 verificados; convergence completo | **Convergido / pronto para revisão/merge** |
| 006 | `lente-sinestesica-completa` | ✓ | — | ✓ | ✓ | ✓ (10/10) | ✓ | ✓ | ✓ | ✓ | Converge completo | **Convergido** |
| 007 | `sona-complete-design` | ✓ | — | ✓ | ✓ | ✓ (27/27) | — | ✓ | ✓ | ✓ `src-tauri/src/status_view.html`, RMS/waveform/repeat, testes | **CONVERGIDO** | **Convergido**** |
| 008 | `ref-device-emulator` | ✓ | — | ✓ | ✓ | ✓ | ✓ | ✓ (estrutural) | ✓ | Apenas docs/estrutural | Converge completo (camada estrutural) | **Convergido (camada estrutural)** |

**Percorrido:** 5 specs totalmente convergidos (`001`, `004`, `005`, `006`, `008`), 1 implementado com defer (`002`), 1 em implementação ativa (`007`), 1 com plan/tasks mas implementação pendente (`003`).

### 1.3 Triage — `.specify/bugs/`

| Slug | assess | fix | test | Status | Resumo |
|---|---|---|---|---|---|
| `corrupt-design-asset` | ✓ | **Iniciado** (`fix.md` presente) | **Iniciado** (`test.md` presente) | **Em progresso** | Asset `ui-proposta-2-lente-sinestesica.png` contém imagem de Rust tutorial, não mockup SONA. Afeta verificação visual de 005/006/007. |
| `corrupt-ui-proposta-asset` | ✗ | ✗ | ✗ | **Sem artefatos completos** | Pasta vazia; assessment incompleto. |
| `sona-visual-core-fix` | ✓ | ✗ | ✗ | **Só assess** | Header quebrado, waveform vazio, botões pequenos persistem. Sem fix/test artefatos ainda. |
| `sona-visual-fix` | ✗ | ✗ | ✗ | **Sem artefatos** | Sem progresso evidente. |
| `sona-waveform-missing-in-screenshot` | ✓ (re-avaliado) | ✓ (aplicado 2026-10-28) | ✓ (parcial) | **Fix parcial / teste parcial** | Precisa `channel_waveform` com `kind='determinate'` inicial e reprodução ativa confirmada. Funcionalidade completa inconclusiva. |

**Percorrido:** 3/5 com assessment; apenas `corrupt-design-asset` e `sona-waveform-missing-in-screenshot` têm artefatos de fix/test. **Nenhum bug fechado com `bug.test` green**. Sem regressões confirmadas que quebrem gates 1–7.

---

## 2. Quanto já percorremos vs quanto falta

### 2.1 Por fluxo

| Fluxo | % percorrido (evidência) | Status | Comentário |
|---|---|---|---|
| Idea Assessment (`/speckit.assess.*`) | ~92% | **Quase completo** | 5/6 completos (todos exceto `directory-reading` em intake 1/5). Decisões refletidas na constituição v2.0.0 e contratos. |
| Spec-Driven Development (núcleo 001) | ~95% | **Consolidado** | `gate-8-shell` PASS, gates 1-7 verdes. Único gap: `gate-8-full` DEFERRED aguardando síntese (002). |
| Spec-Driven Development (Design SONA 005/006/007) | ~100% | \*\*Convergido\*\* | `005`, `006`, `007` todos convergidos (design SONA completo). |
| Spec-Driven Development (Síntese 002) | ~75% | **Implementado com defer** | Código em `crates/adapter-synthesis/`, V3 attestation OK. Pendentes `T045/T046/T048`, `SC-009` → `SC-044`, V4 attestation. |
| Spec-Driven Development (Device/Emulador 003/008) | ~55% | **Parcial** | `008` concluído estruturalmente (`VERIF-EMULATOR`). `003` planejado mas não implementado; build Android configurado (gate 9). |
| Bug Fixing | ~20% | **Inicial/Parcial** | Maioria em assess; poucos com fix/test artefatos; **nenhum fechado com `bug.test` green**. |

### 2.2 O que falta (detalhado)

#### Assessment
- **`directory-reading`**: Completar `research → define → shape → decide` ou executar `/speckit.assess.decide` com veredito `kill`/`go` justificado. **Baixa prioridade**.
- **`fr-024-synthesis-adapter` (002)**: Fechar `SC-009` (benchmark de latência) e validar fronteiras `Piper` — necessário para habilitar `SC-044`/`gate-8-full`.

#### Specs / Implementação
- **007 `sona-complete-design` (PRIORITÁRIO)**: Completar US1-US4 conforme `specs/007-sona-complete-design/tasks.md` (RMS instrumentation/exposição, canvas waveform 5-layer, repeat button, aria/progress, glassmorphism/glow/decorative wave, title binding, tokens jade/celeste). Executar testes US1/US2/US3/US4 + surface-audit. Após completo → `/speckit.converge`.
- **002 `fr-024-synthesis-adapter` (BLOQUEIO FUNCIONAL)**: Fechar `SC-009`, completar `T045/T046/T048` → habilitar `SC-044` → attestation V4 → `gate-8-full` PASS. Este é o único gap real entre "núcleo funciona" e "pipeline completo funcional".
- **003 `reference-device`**: Implementar validação device/emulador conforme `specs/003-reference-device/`, executar testes próprios, gerar attestation. Emulador OK para estrutural; físico obrigatório para `gate-8-full`.
- **001/004/005/006/008**: Convergidos. Manter `gate-8-full` DEFERRED até 002.

#### Bugs
- **`corrupt-design-asset`**: Finalizar substituição do asset corrompido (`ui-proposta-2-lente-sinestesica.png` contém imagem Rust tutorial), completar `bug.fix` e `bug.test`. Considerar *design-asset integrity gate*.
- **`sona-visual-core-fix`**: Completar fix (header/waveform/botões) + `bug.test`.
- **`sona-waveform-missing-in-screenshot`**: Garantir publicação de `channel_waveform` com `kind='determinate'` inicial e reprodução ativa confirmada; rebuild + `bug.test` verde.
- **`sona-visual-fix`**: Avaliar se ainda relevante; pode ser coberto por 007.
- **`corrupt-ui-proposta-asset`**: Decidir se criar assessment/fix ou fechar como não aplicável.

---

## 3. Próximo passo recomendado (80/20 — maior avanço, menor esforço)

### Recomendação: Trabalhar em bugs visuais ou em `002`/`SC-009`

**Por que é 80/20:**
1. **Alta prontidão**: `spec.md`, `plan.md`, `tasks.md` (27/27) completos; contratos/data-model definidos; foundation T001-T006 já feita.
2. **Momentum ativo**: Branch `007-sona-complete-design` em andamento, PR #146/#147 mergeados com ajustes US1.
3. **Alto impacto transversal**: Completa design system SONA (RMS/waveform, repeat button, elementos suplementares) integrando com 005. Resolve/naturaliza muitos bugs visuais abertos.
4. **Maior parte host-verificável**: Foco em `src-tauri/src/status_view.html`, `crates/player`, testes unitários/property-based — sem exigir benchmark físico imediato.
5. **Destrava fechamento visual**: Provavelmente reduz trabalho de `sona-visual-core-fix` e `sona-waveform-missing-in-screenshot`.

**Ação exata:**
```bash
# Garantir branch correta
git -C /home/luis/development/mdreader switch dev
git -C /home/luis/development/mdreader pull origin dev

# Trabalhar no contexto 007
cd /home/luis/development/mdreader
# Continuar implementação conforme specs/007-sona-complete-design/tasks.md
/speckit.implement
```

**Foco imediato:** Completar tarefas restantes em 007 (US1 RMS instrumentation/waveform, US2 canvas waveform, US3 repeat, US4 elementos suplementares) seguindo TDD com property-based tests. Rodar testes US1/US2/US3/US4 + surface-audit após cada fase; após conclusão → `/speckit.converge`.

### Alternativa (maior impacto a longo prazo, maior esforço): Fechar `002` / `SC-009`
Fechar `SC-009` (benchmark latência em dispositivo/emulador adequado), completar `T045/T046/T048`, habilitar `SC-044`, gerar attestation V4 → habilitar `gate-8-full`. Impacto máximo para "pipeline completo funcional", mas exige medições precisas em hardware/ambiente apropriado. **Melhor executar em paralelo** com 007, não em substituição.

**Veredito 80/20:** **Continuar `/speckit.implement` em `007-sona-complete-design`** oferece maior avanço com menor esforço imediato, mantém momentum e destrava fechamento visual sem bloquear no caminho crítico de síntese.

---

## 4. Marcas de progresso (checklist de convergência)

- [x] Constituição v2.0.0 ratificada/amendada (§VII expansão tokens, §VIII elementos suplementares); gates 1–8 definidos; SC-035/SC-051/SC-009/SC-044 documentados.
- [x] Idea Assessment: 5/6 completos (`directory-reading` pendente em intake).
- [x] `004-ressoa-dogfood-build`: verificado verde (`9501bba`, Phase 17, 2026-10-22).
- [x] `001-android-media-shell`: `gate-8-shell PASS`; `gate-8-full DEFERRED` (owner=synthesis).
- [x] `002-fr-024-synthesis-adapter`: `V3_ATTESTATION` (110 pass, `clippy` zero), clarificações resolvidas; `SC-044 DEFERRED`. Pendentes: `T045`, `T046`, `T048`, `SC-009`.
- [x] `005-adopt-design-sona`: **CONVERGIDO** (55/55 tasks, host gates verificados, surface_audit OK).
- [x] `006-lente-sinestesica-completa`: **CONVERGIDO** (10/10 tasks).
- [x] `007 | `sona-complete-design` | ✓ | — | ✓ | ✓ | ✓ (27/27) | — | ✓ | ✓ | ✓ `src-tauri/src/status_view.html`, RMS/waveform/repeat, testes | **CONVERGIDO** | **Convergido****.
- [x] `008-ref-device-emulator`: **CONVERGIDO (camada estrutural)** (`VERIF-EMULATOR`).
- [x] `007`: **CONVERGIDO** (27/27 tasks, US1-US4 completos)**).
- [ ] `002`: Fechar `SC-009` → `T045/T046/T048` → `SC-044` → attestation V4 → `gate-8-full PASS`.
- [ ] `003`: Implementação device/emulador → testes próprios → attestation.
- [ ] Bugs: `corrupt-design-asset` (em progresso), `sona-visual-core-fix`, `sona-waveform-missing-in-screenshot` (parciais) precisam fechar com `bug.test` green; `corrupt-ui-proposta-asset` e `sona-visual-fix` precisam decisão/ação.
- [ ] `directory-reading`: `/speckit.assess.research` → `define` → `shape` → `decide` ou `/speckit.assess.decide` com `kill` justificado (**baixa prioridade**).

---

*Atualizado: 2026-11-01 · Branch atual: `dev` (HEAD `a360a6d5`). Baseado em evidência direta de `.specify/assessments/`, `specs/*`, `.specify/bugs/`, `AGENTS.md`, `OBJETIVO.md`, histórico git, gates e constitution v2.0.0.*
