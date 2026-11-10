# Objetivo do Projeto — mdreader / Ressoa Audio Media Shell

**Referência:** `AGENTS.md`, `.specify/memory/constitution.md` v2.0.0 (amended 2026-10-28), `specs/001-android-media-shell/`, `ROADMAP.md`.

## O que é

Um leitor de documentos Markdown (`mdreader`, produto **Ressa**) voltado a áudio/síntese de voz, construído como workspace modular Rust com shell Android (Kotlin `Service` / `MediaSession`), engine de síntese (`adapter-synthesis`), e design-system (`DESIGN-SONA`). A autoridade de posição, transporte, buffering e rate pertence ao Rust (`crates/player`, `crates/session`, `crates/diag`, `crates/engine-core`); Kotlin é apenas espelho estrutural (`SC-035`).

**Diferencial central (UVP):** Leitor de Markdown 100% offline e air-gapped, com zero permissão de internet, projetado para notas e documentos técnicos (Obsidian, Logseq, Notion). Nenhum texto sai do dispositivo.

## Por que existe (evidência)

- `AGENTS.md`: greenfield com `specs/`, `constitution.md`; arquitetura intencional (mono f32 @ 22050 Hz, `blake3` fingerprint, `redb` bookmarks, fingerprint de documento; zero playback logic em Kotlin).
- Constituição: governança de gates (1–8), supply-chain (`deny` + `audit`), closed-set (rate ladder `[0.75,1.0,1.25,1.5,1.75,2.0,4.0]`, 7 `EventKind`, 9 `EventKind` no diag, 7 confirmações de `EarconKind`), design-dir (`DESIGN-SONA`) como fonte de verdade de tokens/paleta.
- `specs/`: 10 features rastreadas (001 shell → 010 directory-reading); 10 assessments (7 completos, 3 parciais).

## Mercado e contexto competitivo (pesquisa 2026)

### Tamanho e dinâmica do mercado
- O mercado global de apps de TTS (text-to-speech) está em crescimento acelerado, impulsionado por acessibilidade, produtividade e consumo de conteúdo em áudio.
- Apps dominantes: **Speechify** (~$139/ano, cloud-based, exige internet), **NaturalReader** (~$16-20/mês ou $99-199/ano, híbrido), **ElevenReader** (ElevenLabs, freemium com créditos), **Voice Dream Reader** ($19.99 one-time, iOS-only, offline Acapela voices).
- **Lacuna identificada:** Nenhum concorrente principal é 100% offline, air-gapped, e otimizado para Markdown. Speechify e NaturalReader exigem cloud; Voice Dream é iOS-only; @Voice/TTSReader focam em PDFs/EPUBs, não Markdown.

### Posicionamento competitivo (UVP)
| Atributo | Ressoa (mdreader) | Speechify | NaturalReader | Voice Dream Reader | Lector (F-Droid) |
|---|---|---|---|---|---|
| Offline 100% | ✅ Zero internet (air-gapped) | ❌ Cloud obrigatório | ⚠️ Parcial | ✅ Acapela local | ✅ Offline local |
| Privacidade (air-gapped) | ✅ Dados nunca saem | ❌ Texto enviado à nuvem | ❌ Cloud | ⚠️ Parcial | ✅ Zero permissões |
| Markdown nativo | ✅ Cabeçalhos, seções, prosódia, sanitização | ❌ Texto linear | ❌ Texto linear | ❌ EPUB/PDF | ⚠️ Plain text básico |
| Modelo de preço | Freemium + one-time (R$29-49 / $9.99-14.99) | Assinatura $139/ano | Assinatura $99-199/ano | Assinatura $59-79/ano | Gratuito (GPLv3) |
| Plataforma | Android (minSdk 29+) | iOS/Android/Web | iOS/Android/Web | iOS-only | Android |
| TTS engine | Piper pt-BR (sherpa-onnx local) | Cloud neural | Cloud neural | Acapela offline | Android TTS do SO (robótico) |
| Design & UX | Lente Sinestésica SONA (Canvas 5 camadas, RMS) | Padrão corporativo | Padrão | Leitor simples | UI minimalista simples |

### Tendências relevantes (2026)
- **Anti-subscription fatigue:** Usuários de produtividade (devs, pesquisadores, estudantes) cada vez mais preferem pagamento único ("Pay Once, Own Forever") a assinaturas recorrentes.
- **Privacy-first / data sovereignty:** Regulamentações (GDPR, LGPD) e consciência de privacidade impulsionam demanda por apps air-gapped. "Cloud transcription tools expose your private conversations" — narrativa forte.
- **Offline AI:** Modelos locais (Piper, sherpa-onnx) permitem inferência on-device com custo zero de servidor por minuto de áudio.
- **ASO para nichos:** Apps indie vencem em nichos (audiobook, podcast, sleep) em vez de competir head-on com gigantes (Spotify, Audible).

## Resultado esperado (convergência)

- `001` + `002` + `003`: dispositivo de referência verificável (gate-8-shell PASS, gate-8-full DEFERRED até síntese fechar `SC-044`), com `verify.sh` verde e attestation V3 para adaptador.
- `004`: **CONVERGIDO** (2026-11-03). `/speckit.converge` executado → 0 findings. T051 (validação manual ADB) permanece como tarefa manual.
- `005`: adoção completa do `DESIGN-SONA` (tokens, contraste, animações congeladas sob `prefers-reduced-motion`, `SC-007` TalkBack 10/10) — **CONVERGIDO**.
- `006`: Lente Sinestésica completa — **CONVERGIDO**.
- `007`: convergência completa (RMS, waveform 5 camadas, repeat button, T028/T029) — 29/29 tasks — **CONVERGIDO**.
- `008`: harness de emulação estrutural (40/40) — **CONVERGIDO**.
- `009`: abertura de arquivo da caixa de título com auto-play e redb bookmarks — **CONVERGIDO**.
- `010`: leitura de diretório (`directory-reading`), 41/42 tasks concluídas no host; T226 (validação em device) aberta.
- `011`: harness AVD emulator setup — **CONVERGIDO**.
- `012`: hook de ingestão de debug — **CONVERGIDO / Suportado**.
- `013`: pipeline e worker pré-roll de TTFA implementados; `SC-TTFA-001` aguarda medição em device físico.
- `014`: normalização de símbolos Markdown para TTS (`normalize.rs`) — **CONVERGIDO (Cycle 017)**.
- `015`: prosódia semântica e pausas de Markdown (`prosody.rs`) — **CONVERGIDO (Cycle 017)**.
- `016`: síntese de voz clonada pública (Lula) — **DEFERIDA** por ausência de modelo local e risco regulatório (LGPD/STJ/TSE).

## Riscos Tecnológicos e de Licenciamento Críticos (Pesquisa 2026)
- **Licenciamento sherpa-onnx / espeak-ng:** O `sherpa-onnx 2.0.0` removeu `espeak-ng` e `piper-phonemize` devido a conflitos com GPL-3.0 vs Apache-2.0. Manter o pin em `=1.13.8` exige isolamento estrito para compliance do binário.
- **Acessibilidade Android 16 (API 36):** Depreciação de `View.announceForAccessibility()`. Transição requer `setAccessibilityPaneTitle` e `setAccessibilityLiveRegion` no shell Android.
- **Timestamps em nível de palavra:** `sherpa-onnx` ainda não fornece timestamps de palavras (#3705 em aberto); sincronização visual de texto deve focar em sentenças/chunks.

## Restrições não negociáveis (constituição)

- Nenhuma variante `Other`/`Unknown` em `crates/*/src`; nenhum `_ =>` (gate 7).
- Nenhum `unwrap/expect/panic/todo` no código (lints `deny`); erro via `thiserror` + `tracing`.
- Nenhum segredo no repo (`.env`, `*.pem`, `*.key` gitignored); consentimento revogável por provider.
- `Tauri` fixo **2.12.1**; manifest editado diretamente (não via `mobile::update_manifest` que não existe).
- Zero permissão `INTERNET` no manifest (air-gapped por design).
- Formato de handoff mono `f32` @ 22050 Hz; mismatches geram `EngineError` (nunca resample/downmix silencioso).

## Monetização (pesquisa 2026)

### Modelo recomendado: Freemium + Licença Perpétua (Pay Once, Own Forever)
Conforme as melhores práticas de monetização para apps utilitários/produtividade no ecossistema Android:

1. **Nível Gratuito (Core Experience):**
   - Leitura ilimitada de arquivos Markdown individuais com voz padrão Piper pt-BR.
   - Controle de velocidade de 0.75x a 2.0x e marcadores locais em `redb`.
   - Tema padrão SONA.

2. **Nível Pro / Lifetime Unlock (Pagamento Único via Google Play IAP):**
   - **Preço recomendado:** R$ 29,90 a R$ 49,90 (ou $9.99 a $14.99 USD) pagamento único.
   - **Diferenciais Pro:** Catálogo de vozes neurais de alta fidelidade, leitura de pastas completas (directory-reading), temas visuais adicionais, exportação de relatórios diagnósticos, tradutor offline (futuro).
   - **Alinhamento de custo:** Como a inferência roda localmente no chip do usuário, a infraestrutura tem custo zero de servidor por minuto de áudio. Oferecer compra única elimina a fadiga de assinaturas e atrai fortemente a comunidade de desenvolvedores e entusiastas de produtividade (comparável ao Voice Dream Reader: $19.99 one-time, iOS-only).

### Canais de distribuição
- **Google Play Store** (principal): ASO otimizado para nicho Markdown/TTS/offline.
- **F-Droid** (secundário): Alinhado com filosofia open-source e privacy-first; atrai comunidade técnica.
- **GitHub Releases** (terciário): Builds debug/release para early adopters e testers.

## ASO (App Store Optimization — Google Play)

Diferente da Apple App Store, o algoritmo do Google Play indexa o título (30 caracteres), a descrição curta (80 caracteres) e toda a descrição completa (até 4000 caracteres):

- **Title (30 chars):** `Ressoa: Leitor Markdown Áudio`
- **Short Description (80 chars):** `Transforme suas notas Markdown e textos em audiobooks offline com voz neural.`
- **Full Description (4000 chars):** Estrutura focada em densidade semântica (3–5 repetições orgânicas de palavras-chave centrais) destacando:
  - *Privacidade Absoluta:* Zero conexão externa ou permissões de rede.
  - *Controle Total de Áudio:* Relógio de posição sem perdas e escada de velocidades pitch-preserving.
  - *Suporte Nativo ao Markdown:* Títulos, hierarquias e anotações técnicas.
- **Canais Iniciais Orgânicos:** Divulgação direta nas comunidades `r/ObsidianMD`, `r/PKMS`, Hacker News e fóruns de acessibilidade visual.

### Palavras-chave alvo (ASO)
`markdown reader`, `text to speech`, `offline TTS`, `audiobook reader`, `leitor markdown`, `voz neural offline`, `privacy-first reader`, `notas de voz`, `obsidian reader`, `logseq audio`.

---
*Criado: 2026-10-26 · Atualizado: 2026-11-03 · Autor: processo speckit (assessment + specify + plan + tasks) + pesquisa de mercado 2026 · Fonte: `.specify/memory/constitution.md`, `AGENTS.md`, `specs/*/spec.md`, pesquisa web*
