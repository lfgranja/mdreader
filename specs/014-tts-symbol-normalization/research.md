# Idea Research: Fadiga por verbalização de símbolos markdown no TTS

- **Slug**: tts-symbol-fatigue
- **Criado**: 2026-11-07T07:16:18-04:00
- **Confiança da evidência (geral)**: medium

Base: pesquisa web multilíngue (EN/PT/ES) por subagente `ImpressedReindeer` (acesso 2026-11-07) + inspeção local do repositório mdreader/Ressoa (2026-11-07). Afirmações sem fonte externa são marcadas `ASSUMPTION` ou `[INFERÊNCIA]`.

## Usuários & Demanda

- Relato do usuário Ressoa: áudio cansativo por leituras constantes de símbolos como “Asterisco, asterisco, asterisco”, típico de markdown (`*`, `**`, `***`). — [fonte: intake do usuário, 2026-11-07] (confiança: high)
- Perfis afetados: consumo TTS por conveniência, baixa visão/cegueira, dislexia/dificuldades de leitura, fadiga visual; conteúdos markdown crus ou mal normalizados. — [ASSUMPTION/inferência alinhada à literatura de acessibilidade/TTS] (confiança: medium)
- Limiar varia: usuários técnicos podem querer preservar marcação/código; ouvintes casuais preferem fala natural. — [ASSUMPTION] (confiança: medium)
- Demanda implícita: normalização de texto para fala (TN) sensível a contexto, configurável, preservando estrutura/semântica quando relevante, sem depender de SSML no pipeline atual. — [fonte: restrição observada `SsmlLevel::None` em `crates/adapter-synthesis`] (confiança: high)

## Anterior

- **W3C SSML 1.1 (REC 2010-09-07)**: síntese/TN convertem forma ortográfica em forma falada; sem marcação SSML, o processador infere por análise automatizada (pontuação, dados da língua). Mecanismos declarativos: `say-as`, `sub`, `phoneme`. — [fonte: https://www.w3.org/TR/speech-synthesis11/, W3C, 2010-09-07, EN] (confiança: high)
- **CommonMark 0.31.2 (2024-01-28)**: `*`/`_` delimitam ênfase; `***`/`---`/`___` (3+ marcadores, regras) formam *thematic break* (`<hr/>`). `***` frequentemente tem função estrutural de seção, não intenção de pronunciar três asteriscos. — [fonte: https://spec.commonmark.org/0.31.2/, CommonMark, 2024-01-28, EN] (confiança: high)
- **NVDA**: pronúncia de símbolos configurável por nível (`none/some/most/all/char`) e dicionários `symbols.dic`. `*` → “star” (EN, nível `some`); `*` → “asterisco” (pt_BR). — [fonte: https://download.nvaccess.org/releases/2025.2/documentation/userGuide.html; https://raw.githubusercontent.com/nvaccess/nvda/master/source/locale/pt_BR/symbols.dic, NV Access/NVDA, 2025/master, EN/PT-BR] (confiança: high)
- **VoiceOver macOS**: permite controlar pontuação (All/Some/None), criar grupos e substituir leituras (ex.: “star” em vez de “asterisk”). — [fonte: https://support.apple.com/en-am/guide/voiceover/vod1f1248b5e/mac, Apple, acesso 2026-11-07, EN] (confiança: high)
- **TalkBack Android**: verbosidade e controle de pontuação/símbolos falados (níveis variam por versão). — [fonte: https://support.google.com/accessibility/android/answer/6283655?hl=en, Google, acesso 2026-11-07, EN] (confiança: high)
- **eSpeak NG**: opção `--punct` fala nomes de caracteres de pontuação; controle fino por contexto semântico normalmente exige pré-processamento. — [fonte: https://man.archlinux.org/man/espeak-ng.1, eSpeak NG/Arch, acesso 2026-11-07, EN] (confiança: high)
- **Piper/ecossistema**: comportamento de pausas/pontuação depende de modelo, dados e texto/phonemes; issue histórica relata sensibilidade a pontuação/versão eSpeak/phonemize. — [fonte: https://github.com/rhasspy/piper/issues/122 (2023-07-03); https://raw.githubusercontent.com/OHF-Voice/piper1-gpl/main/README.md, acesso 2026-11-07, EN] (confiança: medium-high)
- **Ressoa estado atual (observado)**: `adapter-synthesis::synth::synthesize` passa `segment.content` diretamente a `tts.generate_with_config` (`crates/adapter-synthesis/src/synth.rs`); `chunk.rs` segmenta por blocos/linhas markdown (headings/parágrafos, scan por linha, não fence-aware, sem setext — desvio registrado no módulo) e monta `content` unindo linhas com `\n`/separadores, **sem normalização de símbolos para fala** observada. Capacidades: `SsmlLevel::None`, `AlignmentGranularity::None`. — [fonte: inspeção local, 2026-11-07] (confiança: high)

## Mercado & Contexto

- Leitores/TTS (Speechify, NaturalReader, Voice Dream, Moon+ Reader + TTS do sistema) competem por naturalidade e controle; documentação pública raramente detalha regras específicas de `*`/markdown — comportamento frequentemente depende do motor TTS/SO e configurações de pontuação. — [ASSUMPTION/baixa evidência direta pública; subagente ImpressedReindeer] (confiança: low-medium)
- Screen readers maduros estabelecem padrão de **controle de verbosidade de pontuação/símbolos** (níveis, grupos, substituições). — [fonte: NVDA/VoiceOver/TalkBack docs acima] (confiança: high)
- Custo de não fazer nada: experiência de escuta degradada em documentos markdown (separadores/ênfase/listas), possível abandono do recurso TTS para esse tipo de conteúdo. — [ASSUMPTION, consistente com relato] (confiança: medium)

## Dados & Restrições

- **Rust autoridade**; Kotlin shell mirror/transporte. Decisão “o que será falado” deve ser determinística e rastreável no Rust. — [fonte: constituição/AGENTS.md] (confiança: high)
- **Piper pt-BR `faber` + sherpa-onnx, eSpeak-NG phonemizer**, mono f32 @ 22050 Hz. Não assumir comportamento de TTS cloud; testar empiricamente. — [fonte: adapter-synthesis/provenance] (confiança: high)
- **`SSML=None`, `AlignmentGranularity=None`**: sem `say-as/sub/break`; pausas/substituições via texto normalizado e/ou segmentação. Transformações não devem depender de alinhamento fino do motor; cuidado com `SourceRange`/resume/fingerprint. — [fonte: capabilities/contratos observados] (confiança: high)
- **Segmentação atual preserva `source_range`, `segment_index`, `resume_point`** (`chunk.rs`: `DocumentSegment` imutável após entrega; ranges referem linhas originais). Qualquer normalização não pode quebrar contratos sem migração coordenada. — [fonte: `crates/adapter-synthesis/src/chunk.rs`] (confiança: high)
- **pt-BR**: locuções/nomenclatura devem respeitar pt-BR (“asterisco”, “quebra de seção” opcional). — [ASSUMPTION de UX/localização] (confiança: high)
- **Determinismo**: mesma entrada + mesma config + mesma versão de regras = mesma saída falada. — [fonte: governança/expectativa do projeto] (confiança: high)

## Evidência Contra a Ideia

- **Remover todos `*` globalmente**: perde ênfase/listas e altera literais (`2*3`, `*.txt`, código). — [fonte: https://spec.commonmark.org/0.31.2/] (confiança: high)
- **Remover `***` sempre**: apaga fronteira estrutural (*thematic break*), prejudicando navegação mental. — [fonte: CommonMark 0.31.2; https://www.w3.org/WAI/WCAG2/supplemental/patterns/o3p06-format-punctuation/] (confiança: high)
- **Substituir todo `*` por “asterisco”**: pode manter/agravar verbosidade em ênfase/listas e gerar “asterisco asterisco”. — [fonte: comportamento configurável NVDA/VoiceOver/TalkBack] (confiança: medium-high)
- **Regex simples sobre markdown cru**: ambiguidade (ênfase intrapalavra, escapes, listas, fenced code, links). — [ASSUMPTION técnica; consistente com CommonMark] (confiança: medium)
- **Mesma regra em código/matemática**: altera sentido (multiplicação, ponteiro, wildcard, regex). — [ASSUMPTION técnica] (confiança: high)
- **Depender do engine para resolver**: comportamento de pontuação varia; Piper/eSpeak sensível a fonemização/modelo/texto. — [fonte: Piper #122; eSpeak `--punct`] (confiança: medium-high)
- **Anunciar toda estrutura verbalmente**: excesso de metalinguagem causa fadiga. — [ASSUMPTION/inferência UX] (confiança: medium)

## Lacunas & Perguntas em Aberto

- [NEEDS CLARIFICATION: Medição empírica do Piper `faber` pt-BR + eSpeak para `*`, `**`, `***`, `* * *`, listas, fenced code, `2*3`, URLs — comportamento atual exato?]
- [NEEDS CLARIFICATION: Escopo de entrada suportado: markdown puro, texto extraído, HTML/EPUB? Como código/matemática são representados hoje?]
- [NEEDS CLARIFICATION: Política de produto: padrão fluido vs. literal vs. configurável; necessidade de modo verbatim para revisão/código?]
- [NEEDS CLARIFICATION: Onde normalizar sem quebrar `SourceRange`/resume/bookmarks/fingerprint — transformar `segment.content` apenas, ou também pipeline de ingestão/chunking com mapeamento de offsets?]
- [NEEDS CLARIFICATION: Requisitos de acessibilidade formais (WCAG/usuários de leitor de tela usando TTS do app vs. TalkBack) e expectativas pt-BR.]

## Fontes

| URL sanitizada | Título | Autor/Org | Data publicação | Língua | Confiança | Acesso |
|---|---|---|---|---|---:|---|
| https://www.w3.org/TR/speech-synthesis11/ | Speech Synthesis Markup Language (SSML) Version 1.1 | W3C Voice Browser WG | 2010-09-07 | EN | Alta | 2026-11-07 |
| https://spec.commonmark.org/0.31.2/ | CommonMark Spec 0.31.2 | CommonMark | 2024-01-28 | EN | Alta | 2026-11-07 |
| https://download.nvaccess.org/releases/2025.2/documentation/userGuide.html | NVDA 2025.2 User Guide | NV Access | 2025 | EN | Alta | 2026-11-07 |
| https://raw.githubusercontent.com/nvaccess/nvda/master/source/locale/pt_BR/symbols.dic | NVDA pt_BR symbols.dic | NVDA Contributors | master (acesso) | PT-BR | Alta | 2026-11-07 |
| https://support.apple.com/en-am/guide/voiceover/vod1f1248b5e/mac | Customize punctuation in VoiceOver on Mac | Apple | acesso | EN | Alta | 2026-11-07 |
| https://support.google.com/accessibility/android/answer/6283655?hl=en | Customize Android settings for TalkBack | Google | acesso | EN | Alta | 2026-11-07 |
| https://man.archlinux.org/man/espeak-ng.1 | espeak-ng(1) | eSpeak NG/Arch | acesso | EN | Alta | 2026-11-07 |
| https://github.com/rhasspy/piper/issues/122 | Incorrect Reading of Punctuation Marks (#122) | rhasspy-piper | 2023-07-03 | EN | Média-alta | 2026-11-07 |
| https://raw.githubusercontent.com/OHF-Voice/piper1-gpl/main/README.md | Piper 1 README | OHF-Voice | main (acesso) | EN | Alta | 2026-11-07 |
| https://www.w3.org/WAI/WCAG2/supplemental/patterns/o3p06-format-punctuation/ | Use Clear, Unambiguous Formatting and Punctuation | W3C WAI/COGA | acesso | EN | Alta | 2026-11-07 |
| https://www.elevenways.be/en/articles/screenreaders-special-characters | How screen readers read special characters: an update | Eleven Ways | 2023-03-17 | EN | Média-alta | 2026-11-07 |
| https://pubs.asha.org/doi/10.1044/2025_AJA-25-00018 | Listening-Related Fatigue (AV stimuli/captions study) | Esser, Picou, Hornsby / AJA | 2025 | EN | Alta | 2026-11-07 |

Pesquisa detalhada completa (multilíngue, tabelas, recomendações candidatas, evidências contra): relatório do subagente `ImpressedReindeer` (agent transcript), incorporado/resumido aqui com verificações locais.

---

## Deep research update (2026-11-08, multilingual, parallel subagents)

### Research 3 — Codebase map (insertion point for normalization)
- Flow: `chunk.rs` builds `DocumentSegment.content` (trimmed, joined with `\n\n`); `synth.rs::synthesize` (line 183) reads `segment.content` (NUL check line 196); src-tauri `synthesize_segment` (≈line 5160) is the production chokepoint.
- `synthesize` callsites: `cli.rs:429`; tests `audio_chunk` (540,612,679,753), `unit_failure` (641,701), `player` prop_first_audio_3s:1009, `test_play_latency`:204, `src-tauri/lib.rs:5167`.
- Recommended insertion: normalize in `synth.rs` before `generate_with_config`, keeping `DocumentSegment.content` original to preserve `SourceRange`/`resume_point`/`segment_index`/`byte_length` and the session-owned blake3 fingerprint. No code changes made in this research wave.

### Research 4 — Protected zones & edge cases (minimal set)
Minimal protected zones (leave verbatim) given line-scan chunker (no full AST):
1. Fenced code blocks (``` ... ```) — entire content verbatim.
2. Inline code spans (`` `...` ``) — verbatim.
3. URLs and file paths — verbatim (or spoken as "link" only if URL-only segment).
4. Simple math patterns (`2*3`, `a*b`, `$...$`) — verbatim.
5. Thematic breaks (`---`, `***`, `___`) — convert to spoken pause/section marker, not literal dashes/stars.
6. Emphasis markers (`*x*`, `**x**`, `_x_`) — strip markers, keep inner text.
7. List bullets (`* item`, `- item`) — strip bullet marker, keep item text.
Headings (`#`): strip `#` markers, keep heading text (heading level handled by section structure).
Edge cases: escapes (`\*`), intraword emphasis, setext headings, HTML tags, emoji, multi-byte UTF-8. Heuristic detection acceptable; false positives default to "leave verbatim" (safer).

### Research 2 — Accessibility & prosody (summary)
- W3C SSML 1.1 provides `say-as`, `sub`, `phoneme`, `break`; mdreader has `SsmlLevel::None`, so all normalization must be plain-text.
- COGA/WCAG guidance: avoid reading punctuation excessively; provide user control over verbosity; preserve meaning.
- Screen readers (NVDA/VoiceOver/TalkBack) make punctuation verbosity configurable; mdreader should offer at least a default "fluid" mode and consider a "literal/verbatim" mode for technical review.
- Listening fatigue research (Esser et al., 2025, AJA) supports reducing non-essential auditory load.

### Research 1 — TTS symbol handling (consolidated)
See preceding sections and sources above; key addition: Piper issue #122 (open) confirms punctuation sensitivity; community pattern is upstream normalization (strip_markdown). mdreader must implement equivalent in Rust at synthesis boundary.

### Empirical baseline (pending)
- Run `adapter-synthesis synthesize` on sample markdown snippets; record sample counts/durations as baseline. Faber pronunciation of `*` to be verified by listening (CLI yields PCM, not phonemes). Mark as behavioral/manual.
