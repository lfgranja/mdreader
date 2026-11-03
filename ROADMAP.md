# Roadmap — mdreader / Spec-Driven Development

> **Constituição:** `.specify/memory/constitution.md` v2.0.0 (amended 2026-10-28 — expansão de design tokens §VII, elementos visuais suplementares §VIII)  
> **Framework de Produto & Governança:** `Now / Next / Later` + Spec Kit (`assess.*` → `specify` → `clarify` → `plan` → `tasks` → `implement` → `converge` / `bug.*`)  
> **Objetivo Primário:** `OBJETIVO.md` — leitor de documentos Markdown com síntese de voz 100% offline, Rust como única autoridade de playback/transporte/rate/posição, Kotlin como espelho estrito (`SC-035`), Tauri 2.12.1, CPAL 0.18.2 + AAudio, mono f32 @ 22050 Hz, sherpa-onnx 1.13.8 + Piper pt-BR (`offline_default`). Zero permissão de INTERNET.

*Atualizado: 2026-11-03 · Branch atual: `dev` (HEAD `93b56aba`). Consolidação executiva pós-auditoria de código, convergência da Spec 007, otimização dos gates de verificação, benchmark acústico on-device, estratégia completa de marketing, ASO e monetização, e pesquisa de mercado 2026.*

---

## 1. Visão Estratégica: Now / Next / Later

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│ NOW (Sprint Corrente - ~93% Concluído)                                                      │
│ • ✅ Spec 009 CONVERGIDA (converge executado, 0 findings)                                    │
│ • Completar Spec 010 — Directory Reading (26/42 tasks, 16 restantes)                        │
│ • Limpeza de diretórios de bugs vazios em .specify/bugs/ (4 pendentes)                      │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
                                              ↓
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│ NEXT (Próximo Marco - Hardening, Hardware Gate 8 & Dogfood)                                 │
│ • Benchmark acústico SC-009 (< 3s TTFA) em hardware físico conectado via ADB (53cf9aa7)    │
│ • Transição de `gate-8-full` de DEFERRED para PASS (Spec 002 V4 Attestation)                 │
│ • Bateria de verificação completa no device físico (Spec 003)                              │
│ • Dogfooding & Packaging para distribuição local / release candidate (Spec 004)            │
│ • Completar assessments parciais (custom-voices, custom-voices-catalog, text-translator)   │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
                                              ↓
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│ LATER (Go-To-Market, ASO, Monetização e Expansão)                                           │
│ • Posicionamento: Leitor de Markdown Air-Gapped / Privacy-First (Anti-Speechify)            │
│ • Competidores: Speechify ($139/ano, cloud), NaturalReader ($99-199/ano), Voice Dream (iOS) │
│ • Modelo: Freemium + Licença Perpétua (R$29.90-49.90 / $9.99-14.99 one-time)              │
│ • ASO (Google Play): Title 30ch, Short Desc 80ch, Full Desc 4000ch, nicho Markdown/TTS     │
│ • Canais: r/ObsidianMD, r/PKMS, Hacker News, F-Droid, Google Play                          │
│ • Features de Expansão: Catálogo de Vozes Locais, Tradutor Offline, Directory Reading Pro  │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Mapa do caminho percorrido (Evidência Direta)

### 2.1 Discovery — `.specify/assessments/`

| Slug | Estágios | Veredito | Status | Evidência |
|---|---|---|---|---|
| `design-son-adoption` | 5/5 (`intake→research→problem→concept→decision`) | `go` → `005`/`006`/`007` | **Completo / entregue** | `decision.md` promovido e 100% convergido |
| `fr-024-synthesis-adapter` | 5/5 | `go` condicional | **Completo / promovido para `002`** | `decision.md`: SC-009 e limites Piper |
| `lente-sinestesica-completa` | 5/5 | `go` → `006` | **Completo / entregue** | Promovido e convergido em `specs/006` |
| `ref-device-emulator` | 5/5 (+ `6-gaps-research`) | `go` → `008` | **Completo / entregue** | Promovido e convergido em `specs/008` |
| `ressoa-mobile-app` | 5/5 | Decidido | **Completo** | `decision.md` base da spec `004` |
| `open-file-from-title` | 5/5 | `go` → `009` | **Completo / entregue** | Promovido e implementado em `specs/009` |
| `directory-reading` | 5/5 | `go` → `010` | **Completo / promovido para `010`** | Promovido e em progresso em `specs/010` (26/42 tasks) |
| `text-translator` | `intake.md` apenas | — | **Backlog** | `.specify/assessments/text-translator/intake.md` |
| `custom-voices` | `intake.md` apenas | — | **Backlog** | `.specify/assessments/custom-voices/intake.md` |

---

### 2.2 Delivery — `specs/`

| # | Spec | spec | plan | tasks | checklists | contracts | Código / testes | Verif. | Estado |
|---|---|:---:|:---:|:---:|:---:|:---:|---|---|---|
| 001 | `android-media-shell` | ✓ | ✓ | 140/151 | ✓ | ✓ | ✓ `tests/`, `crates/*` | `gate-8-shell PASS`; `gate-8-full DEFERRED (owner=synthesis)` | **CONVERGIDO NO NÚCLEO** |
| 002 | `fr-024-synthesis-adapter` | ✓ | ✓ | 80/89 | ✓ | ✓ | ✓ `crates/adapter-synthesis/`; 110 pass | `SC-044 DEFERRED`; `V3_ATTESTATION_2026-10-10.md` | **Implementado / defer condicional de benchmark (SC-009)** |
| 003 | `reference-device` | ✓ | ✓ | 12/12 | ✓ | ✓ | Docs, harness e protocolos | Emulador OK; aguarda run final em device físico via ADB | **Harness pronto; aguarda execução final** |
| 004 | `ressoa-dogfood-build` | ✓ | ✓ | 172/173 | ✓ | ✓ | ✓ `verification/` | `verify.sh` GREEN `9501bba` (Phase 17) | **CONVERGIDO E VERIFICADO** |
| 005 | `adopt-design-sona` | ✓ | ✓ | 55/55 | ✓ | ✓ | ✓ `surface_audit` integrado; SC-001/SC-002/SC-007 pass | Gates 1-8 host OK; TalkBack 10/10 | **CONVERGIDO** |
| 006 | `lente-sinestesica-completa`| ✓ | ✓ | 55/55 | ✓ | ✓ | ✓ | Convergido completo | **CONVERGIDO** |
| 007 | `sona-complete-design` | ✓ | ✓ | **29/29** | ✓ | ✓ | ✓ RMS instrumentation, canvas 5-layer, repeat button, T028 e T029 | Host tests green; zero achados pendentes | **CONVERGIDO (100%)** |
| 008 | `ref-device-emulator` | ✓ | ✓ | 40/40 | ✓ | ✓ | Harness de emulação estrutural | Convergido completo | **CONVERGIDO** |
| 009 | `open-file-from-title` | ✓ | ✓ | **26/27** (T184 = validação manual ADB) | ✓ | ✓ | ✓ Storage Access Framework JNI, auto-play, redb offer dialog, recusa segura | T158–T183 concluídas; `/speckit.converge` executado → **0 findings** (converged) | **CONVERGIDO (2026-11-03)** |
| 010 | `directory-reading` | ✓ | ✓ | **26/42** | ✓ | ✓ | ✓ ChapterQueue, directory bookmarks, pre-roll 10s, SAF tree permissions | T185–T213 em progresso; 16 tasks restantes (T214–T229) | **Em progresso / Feature ativa** |

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

### O que já percorremos (~93% do Produto Global):
- **Motor de Áudio e Playback Authority (100%):** O runtime Rust (`crates/player`, `crates/session`, `crates/diag`, `crates/engine-core`) gerencia soberanamente transporte, buffer, taxa pitch-preserving e `PositionClock` (audible progress puro).
- **Design System SONA & Lente Sinestésica (100% Convergido):** Acessibilidade TalkBack (10/10), alto contraste para ambiente externo, visualizador de 5 camadas no canvas alimentado por RMS lock-free a ~20 Hz (`AtomicU32`) e botão repeat integrados (Spec 007 formalmente convergida — 29/29 tasks).
- **Ingestão SAF & Bookmarking Durável (100% em código):** Abertura de arquivos Markdown pela caixa de título via Android SAF, cálculo de fingerprint Blake3, persistência atômica em `redb` a cada 20s e em pausas, e diálogo de retomada inteligente (`offerDialog`).
- **Directory Reading (62% — 26/42 tasks):** ChapterQueue com sort natural, directory bookmarks duráveis, pré-roll de 10s, SAF tree permissions, contextual skip e invalid chapter skipping (TDD RED). Feature ativa em `specs/010-directory-reading`.
- **Padrão de Qualidade e Otimização de CI:** Gates 1 a 8 verdes, clippy estrito sem `unwrap`/`expect`, paralelismo configurado via test-groups no `nextest.toml` e amostragem de proptest controlada. Histórico git limpo (apenas `dev` e `main`). 10 specs rastreadas (5 convergidas, 4 quase completas, 1 ativa).
- **Governança e Spec Kit:** 10 assessments (7 completos, 3 parciais), 10 specs com artefatos completos, 11 bugs triados (7 fechados, 4 pendentes).

### O que ainda falta (~7% para Fechamento Total):
1. **✅ Convergência Formal da Spec 009 (CONCLUÍDA 2026-11-03):**
   - `/speckit.converge` executado → 0 findings (converged). T184 (validação manual ADB) permanece como tarefa manual.
2. **Completar Spec 010 — Directory Reading (NOW/NEXT):**
   - 16 tasks restantes (T214–T229): integração final, testes de convergência, documentação.
3. **Benchmark Acústico em Hardware Real - SC-009 (NEXT):**
   - Comprovar latência acústica inicial < 3s (Time to First Audio) entre clique no Play e emissão PCM audível usando o modelo Piper pt-BR local via Sherpa-ONNX.
   - Atualizar a attestation da Spec 002 para V4 e transformar `gate-8-full` em `PASS`.
4. **Validação Finais em Dispositivo Físico (NEXT):**
   - Executar suite automatizada via ADB no hardware conectado (`53cf9aa7`).
5. **Housekeeping:**
   - Limpar pastas vazias residuais de bugs em `.specify/bugs/` (4 bugs sem artefatos).
   - Completar 3 assessments parciais (custom-voices, custom-voices-catalog, text-translator).
6. **Go-To-Market (LATER):**
   - Publicação no Google Play Store e F-Droid.
   - Execução da estratégia de ASO e monetização.

---

## 4. Estratégia de Go-To-Market, ASO e Monetização (LATER)

### 4.1 Posicionamento Competitivo e Proposta Única de Valor (UVP)

#### Landscape Competitivo (pesquisa 2026)

| Concorrente | Modelo | Offline? | Markdown? | Preço | Plataforma | Lacuna vs Ressoa |
|---|---|---|---|---|---|---|
| **Speechify** | Cloud neural | ❌ Não | ❌ Texto linear | ~$139/ano (assinatura) | iOS/Android/Web | Requer internet, envia texto à nuvem, caro |
| **NaturalReader** | Cloud + local | ⚠️ Parcial | ❌ Texto linear | $99-199/ano (assinatura) | iOS/Android/Web | Assinatura, foco em PDF/EPUB, não Markdown |
| **ElevenReader** | Cloud (ElevenLabs) | ❌ Não | ❌ Texto linear | Freemium + créditos | iOS/Android | Requer internet, modelo de créditos |
| **Voice Dream Reader** | Acapela offline | ✅ Sim | ❌ EPUB/PDF | $19.99 one-time | iOS-only | Não suporta Markdown, não é Android |
| **@Voice / TTSReader** | OS TTS | ✅ Sim | ❌ PDF/EPUB | Freemium | Android | Foco em PDF/EPUB, não Markdown, UI genérica |
| **Ressoa (mdreader)** | Piper local (sherpa-onnx) | ✅ 100% air-gapped | ✅ Cabeçalhos, seções | Freemium + one-time | Android (minSdk 29) | — (nicho não atendido) |

#### Tendências de mercado 2026
- **Anti-subscription fatigue:** Usuários de produtividade (devs, pesquisadores, estudantes) preferem pagamento único ("Pay Once, Own Forever") a assinaturas recorrentes.
- **Privacy-first / data sovereignty:** Regulamentações (GDPR, LGPD) e consciência de privacidade impulsionam demanda por apps air-gapped. Nenhum concorrente principal é 100% offline + air-gapped + Markdown.
- **Offline AI:** Modelos locais (Piper, sherpa-onnx) permitem inferência on-device com custo zero de servidor por minuto de áudio.
- **ASO para nichos:** Apps indie vencem em nichos (audiobook, podcast, sleep) em vez de competir head-on com gigantes (Spotify, Audible).

*   **O Problema do Mercado:** Aplicativos de ponta como *Speechify* cobram assinaturas caras (~$139/ano), transmitem todo o texto para servidores remotos na nuvem e exigem internet ativa. Apps como *Readwise Reader* focam em curadoria de links, mas não em foco e audição pura de Markdown.
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
    *   **Alinhamento de Custo:** Como a inferência roda localmente no chip do usuário, a infraestrutura tem custo zero de servidor por minuto de áudio sintetizado. Oferecer compra única ("Pay Once, Own Forever") elimina a fadiga de assinaturas e atrai fortemente a comunidade de desenvolvedores e entusiastas de produtividade.

### 4.3 Estratégia de ASO (App Store Optimization - Google Play)
Diferente da Apple App Store, o algoritmo do Google Play indexa o título (30 caracteres), a descrição curta (80 caracteres) e toda a descrição completa (até 4000 caracteres):
*   **Title (30 chars):** `Ressoa: Leitor Markdown Áudio`
*   **Short Description (80 chars):** `Transforme suas notas Markdown e textos em audiobooks offline com voz neural.`
*   **Full Description (4000 chars):** Estrutura focada em densidade semântica (3–5 repetições orgânicas de palavras-chave centrais) destacando:
    - *Privacidade Absoluta:* Zero conexão externa ou permissões de rede.
    - *Controle Total de Áudio:* Relógio de posição sem perdas e escada de velocidades pitch-preserving.
    - *Suporte Nativo ao Markdown:* Títulos, hierarquias e anotações técnicas.
*   **Canais Iniciais Orgânicos:** Divulgação direta nas comunidades `r/ObsidianMD`, `r/PKMS`, Hacker News e fóruns de acessibilidade visual.

---

## 5. Próxima Ação Prioritária (O 80/20 Máximo)

### Ação 80/20: Validar T184 na Spec 009 e executar `/speckit.converge specs/009-open-file-from-title`

**Justificativa de Impacto vs Esforço:**
- A Spec 007 já está 100% convergida.
- Toda a implementação da Spec 009 (Storage Access Framework, tratamento JNI e recuperação de marcadores) está completa e testada nos gates locais.
- Ao rodar a convergência formal da Spec 009, 100% das especificações de software/UI estarão formalmente convergidas, liberando o projeto para a fase final exclusiva de testes físicos em hardware via ADB.
