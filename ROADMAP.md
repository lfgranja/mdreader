# Roadmap — mdreader / Spec-Driven Development

> **Constituição:** `.specify/memory/constitution.md` v2.0.0 (amended 2026-10-28 — expansão de design tokens §VII, elementos visuais suplementares §VIII)  
> **Fluxos:** `/speckit.assess.*` (discover) → `/speckit.specify` → `/speckit.clarify` → `/speckit.plan` → `/speckit.tasks` → `/speckit.implement` → `/speckit.converge` (deliver) → `/speckit.bug.assess` → `/speckit.bug.fix` → `/speckit.bug.test` (triage)  
> **Objetivo:** `OBJETIVO.md` — leitor de documentos Markdown com áudio/síntese de voz, Rust como única autoridade de posição/transporte/buffering/rate, Kotlin apenas espelho estrutural (SC-035), Tauri 2.12.1, CPAL 0.18.2 + AAudio, mono f32 @ 22050 Hz, sherpa-onnx 1.13.8 + Piper pt-BR (`offline_default`).

*Atualizado: 2026-11-02 · Branch atual: `dev` (HEAD `0dcd6d5c`). Baseado em `.specify/`, `specs/*/`, bugs testados em 2026-11-02 (`sona-visual-core-fix` PASS; `sona-waveform-missing-in-screenshot` com fix T028 em `lib.rs`), PRs #146/#153/#154 mesclados e dispositivo Android real conectado (`adb: 53cf9aa7`).*

---

## 1. Mapa do caminho percorrido (evidência direta)

### 1.1 Discovery — `.specify/assessments/`

| Slug | Estágios | Veredito | Status | Evidência |
|---|---|---|---|---|
| `design-son-adoption` | 5/5 (`intake→research→problem→concept→decision`) | `go` → `005`/`006`/`007` | **Completo / handoff ativo** | `decision.md` (promovido e entregue) |
| `fr-024-synthesis-adapter` | 5/5 | `go` condicional | **Completo / promovido para `002`** | `decision.md`: SC-009 e limites Piper |
| `lente-sinestesica-completa` | 5/5 | `go` → `006` | **Completo / entregue** | Promovido e convergido em `specs/006` |
| `ref-device-emulator` | 5/5 (+ `6-gaps-research`) | `go` → `008` | **Completo / entregue** | Promovido e convergido em `specs/008` |
| `ressoa-mobile-app` | 5/5 | Decidido | **Completo** | `decision.md` presente |
| `directory-reading` | `intake.md` apenas | — | **Bloqueado** (intake) | `.specify/assessments/directory-reading/intake.md` |

**Progresso Assessments:** 5/6 ideias com avaliação completa (5 promovidas/concluídas). Apenas `directory-reading` permanece em intake isolado.

---

### 1.2 Delivery — `specs/`

| # | Spec | spec | plan | tasks | checklists | contracts | Código / testes | Verif. | Estado |
|---|---|:---:|:---:|:---:|:---:|:---:|---|---|---|
| 001 | `android-media-shell` | ✓ | ✓ | 140/151 | ✓ | ✓ | ✓ `tests/`, `crates/*` | `gate-8-shell PASS`; `gate-8-full DEFERRED (owner=synthesis)` | **Convergido no núcleo (estável)** |
| 002 | `fr-024-synthesis-adapter` | ✓ | ✓ | 80/89 | ✓ | ✓ | ✓ `crates/adapter-synthesis/`; 110 pass | `SC-044 DEFERRED`; `V3_ATTESTATION_2026-10-10.md` | **Implementado / defer condicional de benchmark (SC-009)** |
| 003 | `reference-device` | ✓ | ✓ | 12/12 | ✓ | ✓ | Docs, harness e protocolos | Emulador OK; aguarda run no device físico `53cf9aa7` | **Harness pronto; aguarda execução final** |
| 004 | `ressoa-dogfood-build` | ✓ | ✓ | 172/173 | ✓ | ✓ | ✓ `verification/` | `verify.sh` GREEN `9501bba` (Phase 17) | **Convergido / verificado** |
| 005 | `adopt-design-sona` | ✓ | ✓ | 55/55 | ✓ | ✓ | ✓ `surface_audit` integrado; SC-001/SC-002/SC-007 pass | Gates 1-8 host OK; TalkBack 10/10 | **Convergido** |
| 006 | `lente-sinestesica-completa`| ✓ | ✓ | 55/55 | ✓ | ✓ | ✓ | Convergido completo | **Convergido** |
| 007 | `sona-complete-design` | ✓ | ✓ | **28/29** | ✓ | ✓ | ✓ RMS instrumentation, canvas 5-layer, repeat button, T028 em `lib.rs` | Host tests green; resta apenas T029 (metacritérios) e captura ativa | **Fase final de implementação / Pronto para converge** |
| 008 | `ref-device-emulator` | ✓ | ✓ | 40/40 | ✓ | ✓ | Harness de emulação estrutural | Convergido completo | **Convergido** |

---

### 1.3 Triage & Bug Fixing — `.specify/bugs/`

| Bug | assess | fix | test | Veredito / Status | Descrição |
|---|:---:|:---:|:---:|---|---|
| `sona-visual-core-fix` | ✓ | ✓ | ✓ | **Fechado / PASS** (PR #153) | Header, logo, waveform container e botões corrigidos e testados verde. |
| `sona-waveform-missing-in-screenshot` | ✓ | ✓ | ✓ | **Fechado / PASS** | Causa-raiz resolvida via `publish_rms(0.0)` em `lib.rs` (T028), verificado no device `53cf9aa7` e suíte de testes. |
| `corrupt-design-asset` | ✓ | ✓ | ✓ | **Parcial** | O mockup de referência PNG histórico na pasta precisa de limpeza/substituição por asset oficial. |
| `corrupt-ui-proposta-asset` | — | — | — | **Obsoleto / Vazio** | Diretório sem arquivos; pode ser limpo. |
| `sona-visual-fix` | — | — | — | **Obsoleto / Vazio** | Diretório sem arquivos; incorporado por `sona-visual-core-fix`. |

---

## 2. Quanto do caminho já percorremos? O que ainda falta?

### O que já percorremos (~88% do projeto global):
- **Fundação de Autoridade e Áudio (100%):** O motor Rust (`crates/player`, `crates/session`, `crates/diag`, `crates/engine-core`) é a autoridade absoluta e comprovada por testes unitários e de propriedades. O Kotlin é um espelho passivo puro.
- **Design System SONA e Acessibilidade (95%):** Interface com alto contraste para direção, suporte total a TalkBack (ARIA 10/10), respeito a `prefers-reduced-motion`, escada discreta de velocidade sem distorção de pitch, cálculo de RMS sem bloqueio em tempo real.
- **Ciclo de Especificação (92%):** 5 specs totalmente convergidas (`001`, `004`, `005`, `006`, `008`), e a `007` com 28/29 tarefas concluídas.

### O que ainda falta (~12%):
1. **Fechar a Spec `007-sona-complete-design`:**
   - Resolver T029 (revisar metacritérios não técnicos em `requirements.md`).
   - Validar com áudio ativo na tela conectada (`53cf9aa7`) o visualizador de onda e rodar `/speckit.converge`.
2. **Fechar o Gate 8 Completo (`gate-8-full` em `002-fr-024-synthesis-adapter`):**
   - Validar o benchmark de latência acústica (`SC-009`: <3s entre clique em Play e primeiro som audível) no dispositivo físico real (`53cf9aa7`).
   - Atualizar a attestation para V4 e remover o `DEFERRED` do gate 8.
3. **Validação Final de Hardware (`003-reference-device`):**
   - Executar os testes automatizados diretamente no dispositivo Android real via ADB.
4. **Housekeeping:**
   - Decidir o destino de `directory-reading` (avançar para research ou descartar) e remover diretórios de bugs vazios.

---

## 3. Próximo passo recomendado (O 80/20 Máximo)

### Ação 80/20: Fechar T029 e Convergir a Spec `007-sona-complete-design` (`/speckit.converge`)

**Por que este é o 80/20 (maior avanço com menor esforço):**
- **Esforço mínimo:** A spec 007 já está com **28 das 29 tarefas concluídas** (96%). O código do RMS, canal IPC, botões e canvas de 5 camadas já está implementado e compila limpo sem warnings ou violação de governança Rust. A tarefa restante (T029) trata apenas de documentar ou alinhar os metacritérios de checklist de requisitos em `requirements.md`.
- **Maior impacto:** Concluir a convergência da spec 007 fecha o ciclo visual completo do produto (Lente Sinestésica + Design SONA), consolida os PRs #146 e #153, e encerra a triagem do bug `sona-waveform-missing-in-screenshot`.
- **Desbloqueio:** Libera o projeto para focar exclusivamente na última grande fronteira técnica: o teste em hardware físico (`53cf9aa7`) para fechar o `SC-009` da spec 002.

**Comando de execução recomendado:**
```bash
/speckit.converge
```
*(No contexto de `specs/007-sona-complete-design`, revisando a T029 e confirmando os gates de compilação).*

---

## 4. Marcas de progresso (checklist consolidado)

- [x] Constituição v2.0.0 ratificada com suporte a design tokens expandidos e elementos suplementares.
- [x] `001-android-media-shell`: `gate-8-shell PASS` (núcleo fechado).
- [x] `004-ressoa-dogfood-build`: verificado verde.
- [x] `005-adopt-design-sona`: **CONVERGIDO** (55/55 tarefas).
- [x] `006-lente-sinestesica-completa`: **CONVERGIDO** (55/55 tarefas).
- [x] `008-ref-device-emulator`: **CONVERGIDO** (40/40 tarefas).
- [x] Correção de núcleo visual (`sona-visual-core-fix`): **PASS** (PR #153).
- [ ] `007-sona-complete-design`: Concluir T029 e executar `/speckit.converge` (**PRIORIDADE IMEDIATA 80/20**).
- [ ] `sona-waveform-missing-in-screenshot`: Capturar frame com áudio ativo no device `53cf9aa7` para passar teste para verde.
- [ ] `002-fr-024-synthesis-adapter`: Executar benchmark `SC-009` em hardware real para transformar `gate-8-full` em PASS.
- [ ] `003-reference-device`: Rodar bateria de verificação física no device `53cf9aa7`.
