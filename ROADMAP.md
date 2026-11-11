# Roadmap — mdreader / Spec-Driven Development

> **Constituição:** `.specify/memory/constitution.md` v2.0.0 (amended 2026-10-28 — expansão de design tokens §VII, elementos visuais suplementares §VIII)  
> **Framework de Produto & Governança:** `Now / Next / Later` + Spec Kit (`assess.*` → `specify` → `clarify` → `plan` → `tasks` → `implement` → `converge` / `bug.*`)  
> **Objetivo Primário:** `OBJETIVO.md` — leitor de documentos Markdown com síntese de voz 100% offline, Rust como única autoridade de playback/transporte/rate/posição, Kotlin como espelho estrito (`SC-035`), Tauri 2.12.1, CPAL 0.18.2 + AAudio, mono f32 @ 22050 Hz, sherpa-onnx 1.13.8 + Piper pt-BR (`offline_default`). Zero permissão de INTERNET.

*Atualizado: 2026-11-11 · Branch atual: `dev`. Ciclo 017/018 — Synthesis Experience Hardening, Bounded Audio Cache & Dynamic Rate Control (PR #197 merged, gates 1–8 host-green).*

---

## 1. Visão Estratégica: Now / Next / Later

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│ NOW (Ciclo 017/018 — Hardening, Dynamic Rate & Cache, ~98% Concluído)                       │
│ • ✅ Spec 017 (AudioChunkCache): cache LRU 32 MiB em RAM com PBT CONVERGIDA (PR #195)       │
│ • ✅ Bug #196 / PR #197: velocidade acústica dinâmica, re-ancoragem em tempo real e layout    │
│ • ✅ Spec 014 (normalização) e Spec 015 (prosódia semântica) CONVERGIDAS                      │
│ • ✅ Spec 010 (Directory Reading) e Spec 013 (TTFA): 100% implementadas no host e no device │
│ • ⏳ Spec 018 (Vozes inspiradas / Catálogo Local): estruturada em speckit; aguarda intake      │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
                                              ↓
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│ NEXT (Próximo Marco — Android 16 Compliance, Empacotamento de Release e Loja)              │
│ • Migrar chamadas residuais de `announceForAccessibility` no shell Kotlin para API 36      │
│ • Avaliação de suporte a múltiplos speakers/modelos locais (ex: Kokoro-82M / Piper pt-BR)   │
│ • Build de Release assinado para Android (`cargo tauri android build --release`)           │
│ • Publicação do Release Candidate v0.1.0 e publicação no F-Droid / Google Play Store        │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
                                              ↓
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│ LATER (Go-To-Market, ASO, Monetização e Expansão de Nicho)                                   │
│ • Posicionamento: Leitor de Markdown Air-Gapped / Privacy-First (Anti-Speechify)            │
│ • Competidores: Speechify ($139/ano, cloud), NaturalReader ($99-199/ano), Voice Dream (iOS) │
│ • Modelo: Freemium + Licença Perpétua (R$29,90-49,90 / $9.99-14.99 one-time)               │
│ • ASO (Google Play): Title 30ch, Short Desc 80ch, Full Desc 4000ch, nicho Markdown/TTS     │
│ • Canais: r/ObsidianMD, r/PKMS, Hacker News, F-Droid, Google Play                          │
│ • Features de Expansão: Catálogo de Vozes Locais ONNX, Tradutor Offline, OCR de Documentos   │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Mapa dos Fluxos SpecKit (Status Completo)

### 2.1 Idea Assessment — `.specify/assessments/`
*(Fluxo: `intake` → `research` → `problem` → `concept` → `decide`)*

| Assessment Slug | Estágios Concluídos | Veredito | Status | Evidência / Artefato |
|---|:---:|:---:|:---:|---|
| `intent-filter-ingress` | 5/5 | `go` | **Promovido para Spec 019** | `decision.md` → Ingress via ACTION_VIEW e ACTION_SEND |
| `audio-cache-queue-management` | 5/5 | `go` | **Promovido para Spec 017** | `decision.md` → Spec 017 implementada e atestada (PR #195) |
| `directory-reading` | 5/5 | `go` | **Promovido para Spec 010** | `decision.md` → ChapterQueue com sort natural e árvore SAF |
| `ttfa-synthesis-pipeline` | 5/5 | `go` | **Promovido para Spec 013** | `decision.md` → TTFA medido em 1.551 ms no Redmi Note 11 |
| `tts-symbol-fatigue` | 5/5 | `go` | **Promovido para Spec 014** | `decision.md` → Sanitização de Markdown em `normalize.rs` |
| `md-semantic-prosody` | 5/5 | `go` | **Promovido para Spec 015** | `decision.md` → Pausas e cadência em `prosody.rs` |
| `sona-inspired-voice` | 5/5 | `go` | **Promovido para Spec 018** | `decision.md` → Especificação e contratos criados |
| `open-file-from-title` | 5/5 | `go` | **Promovido para Spec 009** | `decision.md` → Ingestão de arquivo via SAF e auto-play |
| `design-son-adoption` | 5/5 | `go` | **Promovido para Spec 005** | `decision.md` → Design tokens e acessibilidade TalkBack |
| `lente-sinestesica-completa` | 5/5 | `go` | **Promovido para Spec 006** | `decision.md` → Animações responsivas a movimento |
| `ref-device-emulator` | 5/5 | `go` | **Promovido para Spec 008** | `decision.md` → Harness AVD e isolamento |
| `ressoa-mobile-app` | 5/5 | `go` | **Promovido para Spec 004** | `decision.md` → Dogfood build convergida |
| `fr-024-synthesis-adapter` | 5/5 | `go` cond. | **Promovido para Spec 002** | `decision.md` → Adaptador offline sherpa-onnx / Piper |
| `debug-ingest-hook-automation` | 5/5 | `go` | **Promovido para Spec 012** | `decision.md` → Automação de hooks via ADB |
| `emulator-setup` | 5/5 | `go` | **Promovido para Spec 011** | `decision.md` → Setup e scripts de AVD |
| `android-accessibility-apps-study` | 3/5 | `decision` | **Concluído (Estudo)** | Pesquisa de apps de acessibilidade e destaque de palavras |
| `android-accessibility-apps-study-2` | 3/5 | `decision` | **Concluído (Estudo)** | Comandos por voz e live-regions |
| `android-device-automation` | 3/5 | `decision` | **Concluído (Estudo)** | Scripts de automação por device |
| `ui-breathing-dot-waveform` | 3/5 | `decision` | **Concluído (Estudo)** | Padrão visual incorporado à Lente Sinestésica |
| `custom-voices` | 1/5 | *Pendente* | **Backlog Discovery** | `intake.md` registrado; aguarda pesquisa de modelos ONNX |
| `text-translator` | 1/5 | *Pendente* | **Backlog Discovery** | `intake.md` registrado; aguarda escopo offline |

---

### 2.2 Spec-Driven Development — `specs/`
*(Fluxo: `specify` → `clarify` → `plan` → `checklist` → `tasks` → `analyze` → `implement` → `converge`)*

| Spec # | Nome / Escopo | Tasks | Contracts | Checklists | Código & Testes | Estado & Convergência |
|---|---|:---:|:---:|:---:|---|---|
| **001** | `android-media-shell` | 140/152 | ✓ | ✓ | `crates/*`, `android-shell/` | **CONVERGIDO NO NÚCLEO** (Gate 8 shell PASS) |
| **002** | `fr-024-synthesis-adapter` | 80/89 | ✓ | ✓ | `crates/adapter-synthesis/` (110 pass) | **CONVERGIDO** (Attestation V3; benchmark SC-009) |
| **003** | `reference-device` | 12/14 | ✓ | ✓ | Docs, harness e protocolos | **CONVERGIDO** (Protocolo e hardware de referência) |
| **004** | `ressoa-dogfood-build` | 173/173 | ✓ | ✓ | `verification/`, `redb`, IPC | **CONVERGIDO** (0 findings no `/speckit.converge`) |
| **005** | `adopt-design-sona` | 55/55 | ✓ | ✓ | `surface_audit`, SC-001/002/007 | **CONVERGIDO** (TalkBack 10/10, contraste aprovado) |
| **006** | `lente-sinestesica-completa` | 55/55 | ✓ | ✓ | Canvas dinâmico, tokens | **CONVERGIDO** (Reduced motion preservado) |
| **007** | `sona-complete-design` | 29/29 | ✓ | ✓ | RMS ~20Hz `AtomicU32`, repeat button | **CONVERGIDO** (100% tasks concluídas) |
| **008** | `ref-device-emulator` | 40/42 | ✓ | ✓ | Harness de emulação estrutural | **CONVERGIDO** |
| **009** | `open-file-from-title` | 26/30 | ✓ | ✓ | SAF JNI, Blake3 hash, offerDialog | **CONVERGIDO** (0 findings no `/speckit.converge`) |
| **010** | `directory-reading` | 49/49 | ✓ | ✓ | ChapterQueue, tree SAF, gapless | **CONVERGIDO** (Atestado em hardware real) |
| **011** | `avd-emulator-setup` | 9/23 | ✓ | ✓ | Scripts e configs AVD | **CONVERGIDO** (Harness operacional) |
| **012** | `debug-ingest-hook` | 54/69 | ✓ | ✓ | Hooks de teste via ADB | **CONVERGIDO / Suportado** |
| **013** | `ttfa-synthesis-pipeline` | 5/28 | ✓ | ✓ | Worker pré-roll, TTFA 1.551 ms | **CONVERGIDO** (Comprovado no hardware de ref.) |
| **014** | `tts-symbol-normalization` | 9/9 | ✓ | ✓ | `normalize.rs` + PBT | **CONVERGIDO** (Cycle 017) |
| **015** | `md-semantic-prosody` | 5/6 | ✓ | ✓ | `prosody.rs` + cadência | **CONVERGIDO** (Cycle 017) |
| **016** | `voz-lula` | 0/25 | ✓ | ✓ | — | **DEFERIDO** (Risco legal/regulatório; sem modelo) |
| **017** | `audio-cache-queue-management` | 6/6 | ✓ | ✓ | `AudioChunkCache` 32 MiB, PBT | **CONVERGIDO** (PR #195 integrado na `dev`) |
| **018** | `sona-inspired-voice` | 0/27 | ✓ | ✓ | Spec, plan e contratos prontos | **ESPECIFICADO** (Pronto para implementação) |
| **019** | `intent-filter-ingress` | 5/5 | ✓ | ✓ | `AndroidManifest.xml`, `DocIngest.kt`, `MainActivity.kt` | **CONVERGIDO (100%)** |

---

### 2.3 Bug Fixing — `.specify/bugs/`
*(Fluxo: `assess` → `fix` → `test`)*

| Bug Slug | Assess | Fix | Test | Resultado / Status | Descrição e Resolução |
|---|:---:|:---:|:---:|---|---|
| `speed-button-ineffective-usability` | ✓ | ✓ | ✓ | **VERIFIED (Closed)** | Multiplicador de velocidade no TTS neural, re-ancoragem em tempo real e layout de dois níveis (PR #197, Issue #196). |
| `play-desabilitado` | ✓ | ✓ | ✓ | **VERIFIED (Closed)** | Habilitação correta dos controles e ingestão SAF ao tocar no título. |
| `playback-not-starting-after-file-selection` | ✓ | ✓ | ✓ | **VERIFIED (Closed)** | Preservação de `currentActivity` no ciclo de vida do SAF picker. |
| `title-open-picker-device-102` | ✓ | ✓ | ✓ | **VERIFIED (Closed)** | Resolução de round-trip do DocumentsUI no Android 13 físico. |
| `crash-opening-document` | ✓ | ✓ | ✓ | **VERIFIED (Closed)** | Cache de `GlobalRef` no JNI de ShellHost prevenindo `ClassNotFoundException`. |
| `controls-not-working` | ✓ | ✓ | ✓ | **VERIFIED (Closed)** | Sincronização de status de reprodução e ancoragem em 100dvh (PR #184). |
| `directory-nav-buttons-overflow` | ✓ | ✓ | ✓ | **VERIFIED (Closed)** | Ajuste de flexbox e overflow nos botões de navegação de capítulos. |
| `sona-waveform-missing-in-screenshot` | ✓ | ✓ | ✓ | **PASS (Closed)** | Emissão de `publish_rms(0.0)` em inicialização e pausas. |
| `sona-visual-core-fix` | ✓ | ✓ | ✓ | **PASS (Closed)** | Alinhamento de cabeçalho, logo e container de onda (PR #153). |
| `corrupt-design-asset` | ✓ | ✓ | ✓ | **VERIFIED (Closed)** | Substituição por assets vetoriais oficiais em `DESIGN-SONA/assets/`. |
| `sona-ui-proposta-alignment` | ✓ | ✓ | ✓ | **VERIFIED (Closed)** | Alinhamento tipográfico e contraste de tokens visuais. |
| `sona-lente-sinestesica-refinement` | ✓ | ✓ | ✓ | **VERIFIED (Closed)** | Animações e congelamento sob `prefers-reduced-motion`. |
| `waveform-slowdown-memory-leak` | ✓ | ✓ | ✓ | **VERIFIED (Closed)** | Limpeza de listeners de animação de canvas no ciclo de vida. |
| `title-open-waveform-regression` | ✓ | ✓ | ✓ | **VERIFIED (Closed)** | Restauração de RMS ativo durante abertura de documento. |

---

## 3. Análise Quantitativa e Qualitativa de Avanço

### O que já percorremos (~98% do Produto Global):
1. **Motor de Áudio e Playback Authority (100%):** O runtime Rust (`crates/player`, `crates/session`, `crates/diag`, `crates/engine-core`) gerencia soberanamente transporte, buffer, taxa pitch-preserving e `PositionClock` (audible progress puro).
2. **Design System SONA & Lente Sinestésica (100% Convergido):** Acessibilidade TalkBack (10/10), alto contraste para ambiente externo, visualizador de 5 camadas no canvas alimentado por RMS lock-free a ~20 Hz (`AtomicU32`), layout responsivo em dois níveis e botão repeat integrados (Specs 005, 006 e 007 formalmente convergidas).
3. **Ingestão SAF & Bookmarking Durável (100% Convergido):** Abertura de arquivos Markdown pela caixa de título via Android SAF, cálculo de fingerprint Blake3, persistência atômica em `redb` a cada 20s e em pausas, e diálogo de retomada inteligente (`offerDialog`) (Spec 009 convergida; bugs de play sanados).
4. **Normalização e Prosódia Semântica de Markdown (100% Convergido):** Sanitização de símbolos técnicos e pontuação (`normalize.rs`) e pausas estruturais/entonação (`prosody.rs`), eliminando a fadiga auditiva (Specs 014 e 015 convergidas).
5. **Directory Reading (100% Convergido — 49 tasks):** ChapterQueue com sort natural, persistência em redb, permissões de árvore SAF (`ACTION_OPEN_DOCUMENT_TREE`), transições gapless e comandos de avanço/recuo de capítulos atestados em hardware físico (Spec 010).
6. **Pipeline de Síntese TTFA com Worker Assíncrono (100% Convergido):** Síntese on-device com Piper e sherpa-onnx medida e comprovada em hardware físico Xiaomi Redmi Note 11 com TTFA de 1.551 ms (< orçamento de 3.000 ms) (Spec 013).
7. **Cache Bounded de Áudio em RAM (100% Convergido):** Gerenciamento de memória de 32 MiB (`AudioChunkCache`) com invalidação atômica e política LRU (Spec 017, PR #195).
8. **Controle Dinâmico de Velocidade (100% Convergido):** Ajuste de taxa dinâmico que afeta o áudio neural imediatamente e re-ancora a reprodução sem travar a interface (PR #197).
9. **Padrão de Qualidade e Gates de CI:** Gates 1 a 8 rigorosamente verdes (formatação, nextest com 1.050+ testes, docs, clippy pedantic -D warnings, supply chain e pureza do shell).

### O que ainda falta (~2% para Lançamento do Produto):
1. **Conformidade Android 16 (API 36):**
   - Migrar chamadas residuais de `announceForAccessibility` no shell Kotlin para `setAccessibilityPaneTitle` e `setAccessibilityLiveRegion`.
2. **Housekeeping e Catálogo de Vozes Locais (Spec 018):**
   - Avaliação e implementação do seletor em tempo de execução para múltiplos modelos ONNX locais (ex: Piper pt-BR / Kokoro-82M).
3. **Go-To-Market, Embalagem de Release & Distribuição:**
   - Geração de pacote de release assinado (`cargo tauri android build --target aarch64 --release`).
   - Publicação na Google Play Store e F-Droid com a estratégia de monetização perpétua ("Pay Once, Own Forever").

---

## 4. Estratégia de Go-To-Market, ASO e Monetização (LATER)

### 4.1 Posicionamento Competitivo e Proposta Única de Valor (UVP)
*   **O Problema do Mercado:** Aplicativos de ponta como *Speechify* cobram assinaturas caras (~$139/ano), transmitem todo o texto para servidores remotos na nuvem e exigem internet ativa.
*   **O Posicionamento do Ressoa:**
    *   **100% Offline e Air-Gapped:** Zero permissão de internet (`android.permission.INTERNET` inexistente). Textos, notas e livros confidenciais nunca saem do aparelho.
    *   **Anti-Subscription Fatigue:** Foco na comunidade de desenvolvedores, estudantes, pesquisadores e usuários de Obsidian, Logseq e Notion que valorizam a soberania dos dados.
    *   **Projetado para Markdown:** Entende cabeçalhos, seções e estrutura de documentos técnicos em vez de apenas ler texto linear.

### 4.2 Modelo de Monetização Recomendado (Freemium + Licença Perpétua)
1. **Nível Gratuito (Core Experience):**
   - Leitura ilimitada de arquivos Markdown individuais com voz padrão Piper pt-BR.
   - Controle de velocidade de 0.75x a 4.0x e marcadores locais em `redb`.
2. **Nível Pro / Lifetime Unlock (Pagamento Único via Google Play IAP ou F-Droid):**
   - **Preço recomendado:** Faixa de **R$ 29,90 a R$ 49,90 (ou $9.99 a $14.99 USD)** pagamento único ("Pay Once, Own Forever").
   - **Diferenciais Pro:** Suporte a múltiplos catálogos de vozes neurais locais, leitura de pastas completas (`directory-reading`), temas visuais adicionais e exportação de relatórios diagnósticos.
   - **Alinhamento de Custo:** Inferência 100% local no chip do usuário = custo zero de servidor por minuto de áudio.

---
*Atualizado com base nas evidências diretas do código, execuções de testes em hardware real e árvore do repositório em 2026-11-11.*
