# Roadmap — mdreader / Spec-Driven Development

> **Constituição:** `.specify/memory/constitution.md` v2.0.0 (amended 2026-10-28 — expansão de design tokens §VII, elementos visuais suplementares §VIII)  
> **Framework de Produto:** `Now / Next / Later` + Spec Kit (`assess.*` → `specify` → `clarify` → `plan` → `tasks` → `implement` → `converge` / `bug.*`)  
> **Objetivo Primário:** `OBJETIVO.md` — leitor de documentos Markdown com síntese de voz 100% offline, Rust como única autoridade de playback/transporte/rate/posição, Kotlin como espelho estrito (`SC-035`), Tauri 2.12.1, CPAL 0.18.2 + AAudio, mono f32 @ 22050 Hz, sherpa-onnx 1.13.8 + Piper pt-BR (`offline_default`). Zero permissão de INTERNET.

*Atualizado: 2026-11-03 · Branch atual: `dev` (HEAD `02cadc43`). Consolidação executiva pós-auditoria de código, specs, PRs (#181, #182, #184), benchmark da stack TTS e estratégia de marketing/monetização para o app.*

---

## 1. Visão Estratégica: Now / Next / Later

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│ NOW (Sprint Corrente - ~92% Concluído)                                                      │
│ • Fechar T029 (metacritérios) e convergir Spec 007 (Waveform 5 camadas + RMS)               │
│ • Validar T184 e convergir Spec 009 (SAF Ingestão a partir da caixa de título)              │
│ • Limpeza de diretórios de bugs vazios em .specify/bugs/                                    │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
                                              ↓
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│ NEXT (Próximo Marco - Hardening & Hardware Gate 8)                                          │
│ • Benchmark acústico SC-009 (< 3s TTFA) em hardware físico conectado via ADB (53cf9aa7)    │
│ • Transição de `gate-8-full` de DEFERRED para PASS (Spec 002 V4 Attestation)                 │
│ • Bateria de verificação completa no device físico (Spec 003)                              │
│ • Dogfooding & Packaging para distribuição local / release candidate (Spec 004)            │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
                                              ↓
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│ LATER (Go-To-Market, Monetização e Expansão)                                                │
│ • Posicionamento de Mercado: Leitor de Markdown Air-Gapped / Privacy-First (Anti-Speechify)  │
│ • Modelo de Monetização: Freemium com Licença Perpétua (Pay Once, Own Forever) / IAP       │
│ • ASO (App Store Optimization): Foco em "Markdown Audio Reader", "Offline TTS", "Privacidade" │
│ • Features de Expansão: Directory Reading, Catálogo de Vozes Locais e Tradutor Offline     │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Mapa do caminho percorrido (Evidência Direta)

### 2.1 Discovery — `.specify/assessments/`

| Slug | Estágios | Veredito | Status | Evidência |
|---|---|---|---|---|
| `design-son-adoption` | 5/5 (`intake→research→problem→concept→decision`) | `go` → `005`/`006`/`007` | **Completo / entregue** | `decision.md` promovido e convergido |
| `fr-024-synthesis-adapter` | 5/5 | `go` condicional | **Completo / promovido para `002`** | `decision.md`: SC-009 e limites Piper |
| `lente-sinestesica-completa` | 5/5 | `go` → `006` | **Completo / entregue** | Promovido e convergido em `specs/006` |
| `ref-device-emulator` | 5/5 (+ `6-gaps-research`) | `go` → `008` | **Completo / entregue** | Promovido e convergido em `specs/008` |
| `ressoa-mobile-app` | 5/5 | Decidido | **Completo** | `decision.md` base da spec `004` |
| `open-file-from-title` | 5/5 | `go` → `009` | **Completo / entregue** | Promovido e implementado em `specs/009` |
| `directory-reading` | `intake.md` apenas | — | **Backlog** | `.specify/assessments/directory-reading/intake.md` |
| `text-translator` | `intake.md` apenas | — | **Backlog** | `.specify/assessments/text-translator/intake.md` |
| `custom-voices` | `intake.md` apenas | — | **Backlog** | `.specify/assessments/custom-voices/intake.md` |

---

### 2.2 Delivery — `specs/`

| # | Spec | spec | plan | tasks | checklists | contracts | Código / testes | Verif. | Estado |
|---|---|:---:|:---:|:---:|:---:|:---:|---|---|---|
| 001 | `android-media-shell` | ✓ | ✓ | 140/151 | ✓ | ✓ | ✓ `tests/`, `crates/*` | `gate-8-shell PASS`; `gate-8-full DEFERRED (owner=synthesis)` | **Convergido no núcleo (estável)** |
| 002 | `fr-024-synthesis-adapter` | ✓ | ✓ | 80/89 | ✓ | ✓ | ✓ `crates/adapter-synthesis/`; 110 pass | `SC-044 DEFERRED`; `V3_ATTESTATION_2026-10-10.md` | **Implementado / defer condicional de benchmark (SC-009)** |
| 003 | `reference-device` | ✓ | ✓ | 12/12 | ✓ | ✓ | Docs, harness e protocolos | Emulador OK; aguarda run final em device físico via ADB | **Harness pronto; aguarda execução final** |
| 004 | `ressoa-dogfood-build` | ✓ | ✓ | 172/173 | ✓ | ✓ | ✓ `verification/` | `verify.sh` GREEN `9501bba` (Phase 17) | **Convergido / verificado** |
| 005 | `adopt-design-sona` | ✓ | ✓ | 55/55 | ✓ | ✓ | ✓ `surface_audit` integrado; SC-001/SC-002/SC-007 pass | Gates 1-8 host OK; TalkBack 10/10 | **Convergido** |
| 006 | `lente-sinestesica-completa`| ✓ | ✓ | 55/55 | ✓ | ✓ | ✓ | Convergido completo | **Convergido** |
| 007 | `sona-complete-design` | ✓ | ✓ | **28/29** | ✓ | ✓ | ✓ RMS instrumentation, canvas 5-layer, repeat button, T028 em `lib.rs` | Host tests green; resta apenas T029 (metacritérios) | **Fase final de implementação / Pronto para converge** |
| 008 | `ref-device-emulator` | ✓ | ✓ | 40/40 | ✓ | ✓ | Harness de emulação estrutural | Convergido completo | **Convergido** |
| 009 | `open-file-from-title` | ✓ | ✓ | **26/27** | ✓ | ✓ | ✓ Storage Access Framework JNI, auto-play, redb offer dialog, recusa segura | T158–T183 concluídas; resta T184 (validação device ADB) | **Implementado / Pronto para converge** |

---

### 2.3 Triage & Bug Fixing — `.specify/bugs/`

| Bug | assess | fix | test | Veredito / Status | Descrição |
|---|:---:|:---:|:---:|---|---|
| `controls-not-working` | ✓ | ✓ | ✓ | **Fechado / PASS** (PR #184) | Sincronização de status de reprodução e ancoragem de controles em 100dvh. |
| `crash-opening-document` | ✓ | ✓ | ✓ | **Fechado / PASS** | Cache de GlobalRef no JNI de ShellHost prevenindo ClassNotFoundException em background. |
| `sona-visual-core-fix` | ✓ | ✓ | ✓ | **Fechado / PASS** (PR #153) | Header, logo, waveform container e botões corrigidos e testados verde. |
| `sona-waveform-missing-in-screenshot` | ✓ | ✓ | ✓ | **Fechado / PASS** | Causa-raiz resolvida via `publish_rms(0.0)` em `lib.rs` (T028). |
| `corrupt-design-asset` | ✓ | ✓ | ✓ | **Fechado / PASS** | Substituição de mockups corrompidos por assets vetoriais oficiais em `DESIGN-SONA/assets/`. |
| `sona-ui-proposta-alignment` | ✓ | ✓ | ✓ | **Fechado / PASS** | Alinhamento tipográfico e tokens visuais validados. |
| `sona-lente-sinestesica-refinement` | ✓ | ✓ | ✓ | **Fechado / PASS** | Refinamentos de animação, contraste e freeze sob reduced-motion validados. |
| `broken-header-and-controls-regression`| ✓ | — | — | **Superado** | Absorvido e resolvido pelas correções finais do PR #184. |

---

## 3. Análise Quantitativa e Qualitativa de Avanço

### O que já percorremos (~92% do Produto Global):
- **Motor de Áudio e Playback Authority (100%):** O runtime Rust (`crates/player`, `crates/session`, `crates/diag`, `crates/engine-core`) gerencia exclusivamente transporte, buffer, taxa pitch-preserving e `PositionClock` (audible progress puro).
- **Design System SONA & Lente Sinestésica (98%):** Acessibilidade TalkBack (10/10), alto contraste para ambiente externo, visualizador de 5 camadas no canvas alimentado por RMS lock-free a ~20 Hz (`AtomicU32`).
- **Ingestão SAF & Bookmarking Durável (100% em código):** Abertura de arquivos Markdown pela caixa de título via Android SAF, cálculo de fingerprint Blake3, persistência atômica em `redb` a cada 20s e em pausas, e diálogo de retomada inteligente (`offerDialog`).
- **Padrão de Qualidade e Governança:** Gates 1 a 8 verdes, clippy estrito sem `unwrap`/`expect`, pureza arquitetural Shell-Kotlin e histórico git sem branches órfãs ou PRs pendentes.

### O que ainda falta (~8% para Fechamento Total):
1. **Convergência Formal das Specs 007 e 009 (NOW):**
   - Fechar T029 na spec 007 e rodar `/speckit.converge`.
   - Fechar T184 na spec 009 e rodar `/speckit.converge`.
2. **Benchmark Acústico em Hardware Real - SC-009 (NEXT):**
   - Comprovar latência acústica inicial < 3s (Time to First Audio) entre clique no Play e emissão PCM audível usando o modelo Piper pt-BR local via Sherpa-ONNX.
   - Atualizar a attestation da Spec 002 para V4 e transformar `gate-8-full` em `PASS`.
3. **Validação Finais em Dispositivo Físico (NEXT):**
   - Executar suite automatizada via ADB no hardware conectado (`53cf9aa7`).
4. **Housekeeping:**
   - Limpar pastas vazias residuais de bugs em `.specify/bugs/`.

---

## 4. Estratégia de Go-To-Market, Posicionamento e Monetização (LATER)

### 4.1 Posicionamento Competitivo e Proposta Única de Valor (UVP)
*   **O Problema do Mercado:** Aplicativos de ponta como *Speechify* cobram assinaturas caras (~$139/ano), transmitem todo o texto para servidores remotos na nuvem e exigem internet ativa. Apps como *Readwise Reader* focam em gestão de links, mas não em foco e audição pura de Markdown.
*   **O Posicionamento do Ressoa:**
    *   **100% Offline e Air-Gapped:** Zero permissão de internet (`android.permission.INTERNET` inexistente). Textos, notas e livros confidenciais nunca saem do aparelho.
    *   **Anti-Subscription Fatigue:** Foco na comunidade de desenvolvedores, estudantes, pesquisadores e usuários de Obsidian, Logseq e Notion que valorizam a soberania dos dados.
    *   **Projetado para Markdown:** Entende cabeçalhos, seções e estrutura de documentos técnicos em vez de apenas ler texto linear.

### 4.2 Modelo de Monetização Recomendado (Freemium + Licença Perpétua)
Conforme as melhores práticas de monetização para apps utilitários e de produtividade no ecossistema Android:
1.  **Nível Gratuito (Core Experience):**
    *   Leitura ilimitada de arquivos Markdown individuais com voz padrão Piper pt-BR.
    *   Controle de velocidade de 0.75x a 2.0x e marcadores locais em `redb`.
2.  **Nível Pro / Lifetime Unlock (Pagamento Único via Google Play IAP ou F-Droid):**
    *   **Preço recomendado:** Faixa de **R$ 29,90 a R$ 49,90 (ou $9.99 a $14.99 USD)** pagamento único ("Pay Once, Own Forever").
    *   **Diferenciais Pro:** Suporte a múltiplos catálogos de vozes neurais de alta fidelidade, leitura de pastas completas (`directory-reading`), temas visuais adicionais e exportação de relatórios diagnósticos.
    *   **Evitar Assinatura Recorrente:** Como o app não tem custo de servidor na nuvem por minuto de áudio gerado, a licença perpétua alinha perfeitamente a estrutura de custo zero do produto com a preferência dos usuários "privacy-first".

### 4.3 Estratégia de ASO e Distribuição Orgânica
*   **Canais Iniciais:** Comunidades de produtividade (Reddit `r/ObsidianMD`, `r/PKMS`, Hacker News, comunidades de acessibilidade visual no Brasil).
*   **Palavras-Chave ASO:** "Leitor Markdown Áudio", "Markdown TTS Offline", "Audiobook Markdown", "Text to Speech Privado", "Voz Neural Offline".

---

## 5. Próxima Ação Prioritária (O 80/20 Máximo)

### Ação 80/20: Fechar T029 na Spec 007 e T184 na Spec 009, executando `/speckit.converge`

**Justificativa de Impacto vs Esforço:**
- O código de ambas as features (RMS/Waveform e SAF File Picker) está 100% implementado, compilando e testado verde nos gates locais.
- A finalização formal de convergência consolida o ciclo de desenvolvimento das interfaces e controles do leitor, habilitando o projeto a focar exclusivamente nos testes físicos de hardware via ADB e no empacotamento do produto para lançamento.
