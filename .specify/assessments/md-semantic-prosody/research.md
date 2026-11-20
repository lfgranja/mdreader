# Idea Research: Entonação/prosódia semântica para leitura de markdown

- **Slug**: md-semantic-prosody
- **Criado**: 2026-11-07T07:24:51-04:00
- **Confiança da evidência (geral)**: medium (mapeamento local forte + pesquisa web consolidada; **faltam** baseline empírico Piper `faber` pt-BR e medições A/B de UX/carga cognitiva)

Base: intake, inspeção local, mapeamento `AncientWildfowl`, pesquisa web consolidada `CivilianAnglerfish` (2026-11-07). Afirmações sem fonte primária verificada nesta sessão são `ASSUMPTION`/`[INFERÊNCIA]`. Relatório detalhado: `agent://CivilianAnglerfish`.

## Usuários & Demanda

- Relato do usuário: leitura TTS de `.md` deveria variar entonação/postura vocal conforme semântica (títulos, ênfase, citações, listas, código, separadores, perguntas/exclamações, parágrafos), aproximando fala natural humana. — [fonte: intake 2026-11-07] (confiança: high)
- Benefício esperado: maior naturalidade, compreensão de estrutura e menor monotonia em leitura longa. — [ASSUMPTION/alinhado a UX de TTS] (confiança: medium)
- Risco de demanda divergente: alguns usuários preferem fala neutra/rápida; usuários técnicos podem preferir literalidade em código/links. — [ASSUMPTION; interação com acessibilidade/configurabilidade] (confiança: medium)

## Anterior

- **SSML 1.1 (W3C REC 2010-09-07)** define controle prosódico declarativo: `<prosody>` (pitch, rate, volume, contour), `<emphasis>`, `<break>`, `<s>`/`<p>`; útil quando o engine suporta SSML. — [fonte normativa conhecida: https://www.w3.org/TR/speech-synthesis11/] (confiança: high, **não verificada novamente nesta chamada de tool**)
- **Ressoa estado atual (confirmado localmente)**: `adapter-synthesis::synth::synthesize` passa `segment.content` literal a `tts.generate_with_config(..., GenerationConfig::default())`; VITS config fixa `noise_scale=0.667`, `noise_w=0.8`, `length_scale=1.0`, `num_threads=1`; `SsmlLevel::None`, `AlignmentGranularity::None`. `chunk.rs` preserva `SourceRange/resume_point/segment_index` e só `BoundaryKind::{Paragraph,Heading}` via line-scan (não fence-aware, sem setext). — [fonte: `AncientWildfowl` + inspeção] (confiança: high)
- **Semântica perdida**: ênfase, strong, blockquote, listas, código, thematic break, links, tabelas, perguntas/exclamações (além de pontuação no texto) não viram metadados prosódicos; motor recebe texto plano. — [fonte: `AncientWildfowl`] (confiança: high)
- **Avaliação relacionada**: `tts-symbol-fatigue` (`needs-clarification`) — normalização de marcadores pode remover pistas visuais/semânticas; precisa coordenar “texto limpo para fala” vs. “sinais de estrutura/postura”. — [fonte: `.specify/assessments/tts-symbol-fatigue/`] (confiança: high)

## Mercado & Contexto

- TTS de leitura moderna (apps/leitores) busca naturalidade via modelos neurais, pausas, ênfase e segmentação; sem SSML/controles ricos, a qualidade depende muito do modelo e do pré-processamento textual. — [ASSUMPTION; pendente fontes do `CivilianAnglerfish`] (confiança: medium)
- Markdown é formato estrutural comum para notas/docs; mapear estrutura para fala é oportunidade clara, mas ambígua sem parser/heurísticas robustas. — [fonte: CommonMark; ASSUMPTION de produto] (confiança: medium)

## Dados & Restrições

- **Rust autoridade, determinismo**; Kotlin mirror. — [fonte: governança] (confiança: high)
- **Piper pt-BR `faber` (VITS) + sherpa-onnx/eSpeak-NG**, mono f32@22050Hz. Controle prosódico direto provável limitado ao que o texto/phonemization/modelo suportam; `length_scale/noise` são globais no config atual. — [fonte: `synth.rs`/`AncientWildfowl`; capacidade exata por segmento **não medida**] (confiança: medium)
- **`SSML=None`, `AlignmentGranularity=None`**: sem `<prosody>/<emphasis>/<break>` declarativo; sem alinhamento fino garantido. — [fonte: capabilities/consume] (confiança: high)
- **Contratos sensíveis**: `SourceRange`/`resume_point`/`segment_index`, fingerprint blake3 do conteúdo original, CLI `chunk` JSON metadata-only (sem `content`), budget/byte_length e imutabilidade de segmento. — [fonte: `chunk.rs`/`session`/CLI/`AncientWildfowl`] (confiança: high)
- **Chunker line-scan**: desvios (não fence-aware, sem setext); inferência semântica adicional herda risco de misclassificação. — [fonte: `chunk.rs`] (confiança: high)

## Evidência Contra a Ideia

- **Prosódia exagerada/artificial** pode reduzir compreensão, soar teatral ou cansar em leitura longa (títulos/ênfases excessivos, pausas longas). — [ASSUMPTION, comum em UX TTS; pendente fontes] (confiança: medium)
- **Heurísticas frágeis**: markdown malformado, código, tabelas, links, matemática, fenced blocks podem ser interpretados errado; chunker atual não é fence-aware. — [fonte: `chunk.rs` desvios + `AncientWildfowl`] (confiança: high)
- **Conflito com normalização de símbolos**: remover `*`/`#`/`>` antes de inferir semântica destrói pistas; inferir antes de normalizar pode verbalizar marcadores. Requer pipeline ordenado e zonas protegidas. — [fonte: `tts-symbol-fatigue`] (confiança: high)
- **Limite do engine**: sem SSML/capacidade prosódica, variação de “postura vocal” pode ser sutil ou inconsistente; forçar por hacks textuais pode introduzir artefatos. — [ASSUMPTION técnica; requer baseline `faber`] (confiança: medium)
- **Determinismo/configurabilidade**: mudanças globais de voz não representam semântica local; preferências de usuários variam. — [ASSUMPTION] (confiança: medium)

## Lacunas & Perguntas em Aberto

- [NEEDS CLARIFICATION: Baseline empírico Piper `faber` pt-BR/eSpeak: como reage a quebras de linha, pontuação, títulos em maiúsculas, perguntas, exclamações, reticências, parênteses, marcadores textuais de pausa? Há controle utilizável por segmento sem SSML?]
- [NEEDS CLARIFICATION: Quais elementos markdown são prioritários (MVP) e qual “postura vocal” aceitável (sutil/expressiva/configurável)?]
- [NEEDS CLARIFICATION: Evolução de `SsmlLevel`/capability/engine é aceitável em fase futura, ou solução deve permanecer em texto plano determinístico no adapter atual?]
- [NEEDS CLARIFICATION: Estratégia para código/fenced/math/links/tabelas — preservar literalidade e evitar prosódia enganosa?]
- [NEEDS CLARIFICATION: Ordem de pipeline com `tts-symbol-fatigue` (projeção semântica→fala compartilhada) e política de zonas protegidas.]

## Fontes

| URL sanitizada | Título | Autor/Org | Data | Língua | Confiança | Acesso |
|---|---|---|---|---|---:|---|
| https://www.w3.org/TR/speech-synthesis11/ | Speech Synthesis Markup Language (SSML) 1.1 | W3C | 2010-09-07 | EN | Alta (normativa; não revalidada nesta tool call) | 2026-11-07 |
| local: crates/adapter-synthesis/src/synth.rs | Síntese Piper/sherpa, GenerationConfig default, VITS params | Ressoa repo | atual | — | Alta | 2026-11-07 |
| local: crates/adapter-synthesis/src/chunk.rs | Segmentação line-scan, SourceRange/resume, BoundaryKind | Ressoa repo | atual | — | Alta | 2026-11-07 |
| local: .specify/assessments/tts-symbol-fatigue/* | Avaliação relacionada (símbolos markdown) | Ressoa repo | 2026-11-07 | PT-BR | Alta | 2026-11-07 |
| agent://AncientWildfowl | Mapeamento codebase prosódia/semântica | subagente | 2026-11-07 | PT-BR | Média-alta | 2026-11-07 |
| agent://CivilianAnglerfish | Pesquisa web multilíngue consolidada (SSML, Piper/sherpa/eSpeak, prosódia, acessibilidade, contra-evidências) | subagente | 2026-11-07 | EN/PT/ES/multi | Média-alta (síntese; validar primárias antes de spec) | 2026-11-07 |
