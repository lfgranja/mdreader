# Idea Intake: Entonação/prosódia semântica para leitura de markdown

- **Slug**: md-semantic-prosody
- **Criado**: 2026-11-07T07:20:17-04:00
- **Fonte**: texto colado (solicitação do usuário no repositório mdreader/Ressoa)
- **Tipo**: improvement / exploration

## Ideia (conforme capturada)

> "Leitura com entonação diferente de acordo com a semântica do arquivo .md, oferecendo mudanças na postura da voz semelhantes à leitura natural humana".

## Reafirmação

A leitura em voz alta (TTS) de arquivos `.md` deveria variar entonação/postura vocal conforme a semântica do markdown (por exemplo: títulos, ênfase, citações, listas, código, separadores, perguntas/exclamações, parágrafos), aproximando-se de uma leitura natural humana, em vez de uma fala uniforme.

## Origem & Contexto

- **Levantado por**: usuário (solicitação direta no contexto do projeto Ressoa/mdreader)
- **Gatilho**: desejo de naturalidade/expressividade na escuta de markdown; relacionado ao contexto de qualidade TTS do Ressoa (avaliação separada `tts-symbol-fatigue`, `needs-clarification`).

## Contexto técnico inicial (observado no repositório, não avaliativo)

- Ressoa/mdreader: leitor TTS Android; Rust é autoridade de playback/ingest; Kotlin shell mirror/transporte.
- Síntese: `crates/adapter-synthesis` com sherpa-onnx/Piper pt-BR `faber` (eSpeak-NG phonemizer), mono f32 @ 22050 Hz.
- Capacidades declaradas observadas: `SsmlLevel::None`, `AlignmentGranularity::None`.
- `synth.rs` envia `segment.content` diretamente a `tts.generate_with_config` (sem SSML/sem camada prosódica observada). `chunk.rs` faz segmentação line-scan (headings/parágrafos; desvios conhecidos: não fence-aware, sem setext) preservando `SourceRange`/`resume_point`/`segment_index`.

## Desconhecidos de Primeira Vista

- [NEEDS CLARIFICATION: Quais elementos semânticos de `.md` devem receber “postura vocal” diferente? (H1–H6, ênfase/strong, blockquote, listas, código, links, tabelas, thematic break, perguntas, exclamações, notas/callouts, front matter)]
- [NEEDS CLARIFICATION: O Piper `faber` pt-BR (VITS) + eSpeak-NG/sherpa-onnx suporta controle prosódico utilizável sem SSML (pausas, taxa, tom, ênfase)? Qual é o baseline empírico?]
- [NEEDS CLARIFICATION: `SsmlLevel::None` é restrição contratual/implementação atual — evolução exigiria mudança de capability/engine/pipeline?]
- [NEEDS CLARIFICATION: Como mapear semântica markdown para controles de fala determinísticos sem quebrar `SourceRange`/resume/segmentação/CLI JSON/fingerprint?]
- [NEEDS CLARIFICATION: Risco de “prosódia artificial/exagerada” reduzir compreensão ou cansar; acessibilidade tem preferências configuráveis?]
- [NEEDS CLARIFICATION: Interação com `tts-symbol-fatigue` — normalização de marcadores pode remover pistas semânticas necessárias para prosódia?]
