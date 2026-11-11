# Objetivo do Projeto — mdreader / Ressoa Audio Media Shell

**Referência:** `AGENTS.md`, `.specify/memory/constitution.md` v2.0.0 (amended 2026-10-28), `specs/001-android-media-shell/`, `ROADMAP.md`.

## O que é

Um leitor de audiolivros e notas Markdown (`mdreader`, produto **Ressoa**) voltado a áudio e síntese de voz neural, construído como um workspace modular em **Rust** integrado a um shell Android nativo (Kotlin `Service` / `MediaSession` / `AudioFocusRequest`) e interface Tauri v2 (`DESIGN-SONA`).

**Princípio de Autoridade Rigoroso:** O Rust (`crates/player`, `crates/session`, `crates/diag`, `crates/engine-core`) é a **única autoridade** sobre reprodução, transporte, taxa de velocidade, buffer PCM e relógio de posição. O Kotlin e a WebView atuam exclusivamente como **espelhos estruturais** (`SC-035`), apenas renderizando o estado JSON publicado pelo Rust.

**Diferencial Central (UVP):** Leitor de Markdown **100% offline e air-gapped**, com **zero permissão de internet** (`android.permission.INTERNET` ausente no manifesto). Projetado sob medida para documentos técnicos, anotações de PKMS (Obsidian, Logseq, Notion) e leitura passiva/condução. Nenhum byte de texto ou áudio jamais sai do dispositivo do usuário.

---

## Por que existe (Evidência & Fundamentos)

- **Arquitetura Imutável:** Formato de handoff mono `f32` @ 22.050 Hz, identificação de documentos via hash criptográfico `Blake3`, persistência atômica em `redb` a cada 20 segundos e em pausas, e ausência absoluta de lógica de playback em Kotlin.
- **Constituição e Governança:** Gates 1 a 8 rigorosamente aplicados via `scripts/verify.sh` (formatação, censo de testes com `nextest`, documentação, `clippy::pedantic -D warnings`, auditoria de supply-chain com `cargo deny`/`cargo audit`, conjuntos fechados sem braços coringa `_ =>`, e pureza do shell).
- **Design System SONA:** Interface de alto contraste, alvos de toque WCAG (≥48x48px), visualizador de espectro sonoro em 5 camadas alimentado por medição RMS assíncrona a ~20 Hz (`AtomicU32`) e compatibilidade total com TalkBack (10/10).

---

## Mercado e Contexto Competitivo (Pesquisa 2026)

### Tamanho e Dinâmica do Mercado
- O mercado global de síntese de voz e audiolivros cresce fortemente impulsionado pela busca por produtividade, aprendizado passivo e acessibilidade.
- **Principais Concorrentes:**
  - **Speechify:** Dominante, porém caro (~$139/ano recorrente), requer conexão de nuvem contínua e envia todo o texto lido para servidores remotos.
  - **NaturalReader:** Híbrido, assinatura mensal/anual ($99–199/ano), foco voltado a PDFs e documentos educacionais genéricos.
  - **ElevenReader:** Dependente da nuvem da ElevenLabs, modelo baseado em créditos de consumo.
  - **Voice Dream Reader:** Pago uma única vez ($19.99), mas restrito exclusivamente ao iOS e com foco em EPUB/PDF.
  - **Leitores Tradicionais Android (@Voice, TTSReader):** Dependem do sintetizador padrão do SO Android (frequentemente mecânico/robótico) ou aplicam interfaces complexas e cheias de anúncios.

### Matriz Comparativa
| Atributo | Ressoa (mdreader) | Speechify | NaturalReader | Voice Dream Reader | Leitores Android Genéricos |
|---|---|---|---|---|---|
| **100% Offline (Air-Gapped)** | ✅ Zero internet | ❌ Requer nuvem | ⚠️ Híbrido | ✅ Vozes locais | ⚠️ Parcial |
| **Privacidade Absoluta** | ✅ Sem telemetria/nuvem | ❌ Dados na nuvem | ❌ Dados na nuvem | ⚠️ Parcial | ❌ Anúncios / Trackers |
| **Markdown Estruturado** | ✅ Títulos, listas, código | ❌ Texto linear | ❌ Texto linear | ❌ EPUB/PDF | ❌ Sem sanitização |
| **Modelo Comercial** | Freemium + Compra Única | Assinatura $139/ano | Assinatura $99-199/ano | Compra $19.99 (iOS) | Assinatura ou Anúncios |
| **Motor TTS** | Piper pt-BR / ONNX local | Nuvem proprietária | Nuvem proprietária | Acapela offline | Android OS TTS |
| **Acessibilidade & Design** | Lente Sinestésica SONA (RMS) | Padrão corporativo | Básico | Leitor simples | UI desatualizada |

---

## Estado de Convergência do Produto (Ciclo 017/018)

- **Spec 001 a 003:** Shell Android, adaptador de síntese inicial e protocolo do dispositivo de referência.
- **Spec 004:** Dogfooding build do Ressoa convergida com zero findings.
- **Spec 005 a 007:** Adoção de design tokens SONA, Lente Sinestésica no canvas com RMS a 20 Hz e botão repeat — **100% CONVERGIDO**.
- **Spec 008:** Harness de emulação e testes estruturais de AVD — **CONVERGIDO**.
- **Spec 009:** Ingestão de arquivos Markdown via Storage Access Framework (SAF) ao tocar no título, diálogo inteligente de retomada (`offerDialog`) e persistência Blake3 — **CONVERGIDO**.
- **Spec 010:** Leitura de múltiplos capítulos em diretórios (`directory-reading`), ordenação natural, permissões persistentes de árvore SAF e transições gapless — **CONVERGIDO**.
- **Spec 011 & 012:** Setup de emuladores AVD e automação de hooks de ingestão para debug via ADB — **CONVERGIDO**.
- **Spec 013:** Otimização de tempo até primeiro som (TTFA) via worker pré-roll assíncrono, comprovado em hardware real em 1.551 ms (< orçamento de 3.000 ms) — **CONVERGIDO**.
- **Spec 014 & 015:** Sanitização de símbolos técnicos e formatação (`normalize.rs`) e prosódia semântica estrutural de Markdown (`prosody.rs`) — **CONVERGIDAS**.
- **Spec 016 (Voz Lula):** **DEFERIDA** formalmente por restrições de licenciamento, ausência de modelo permissivo e riscos regulatórios (LGPD/STJ/TSE).
- **Spec 017:** Cache em memória RAM limitado a 32 MiB (`AudioChunkCache`) com política LRU e invalidação por taxa (PR #195) — **CONVERGIDO**.
- **Correções de Usabilidade & Taxa Dinâmica (#196 / PR #197):** Multiplicador de velocidade no TTS neural, re-ancoragem em tempo real durante playback e layout responsivo de dois níveis — **CONVERGIDO**.
- **Spec 018:** Especificação para expansão de múltiplos modelos e catálogo de vozes locais — **ESPECIFICADO**.
- **Spec 019:** Ingress nativo Android via Intent Filters (`ACTION_VIEW` e `ACTION_SEND`) — **CONVERGIDO**.
- **Spec 020:** Expansão do orçamento de documento e capítulo de 91 KiB para 1 MiB (`MAX_DOC_BYTES = 1_048_576`), viabilizando leitura contínua de notas longas de PKMS e livros completos — **CONVERGIDO**.

---

## Modelo de Monetização e Go-To-Market

### Modelo Recomendado: Freemium + Licença Perpétua ("Pay Once, Own Forever")
1. **Camada Gratuita:** Leitura ilimitada de notas e documentos Markdown avulsos com a voz padrão Piper pt-BR, controles de transporte, ajuste de velocidade (0.75x a 4.0x) e marcadores duráveis em `redb`.
2. **Camada Pro (Pagamento Único: R$ 29,90 a R$ 49,90 / $9.99 a $14.99 USD):** Leitura de diretórios completos (`directory-reading`), múltiplos catálogos de vozes neurais locais, temas visuais adicionais e diagnósticos exportáveis.
3. **Custo Estrutural:** Como toda a computação neural ocorre localmente no processador do celular, o custo de servidor por minuto de áudio sintetizado é **zero**, tornando a licença vitalícia viável, sustentável e altamente atrativa.

---
*Documento atualizado em 2026-11-11 com base nos gates executados, especificações atestadas e PRs consolidados na branch `dev`.*
