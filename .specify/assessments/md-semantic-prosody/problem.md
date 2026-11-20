# Definição do Problema: Entonação/prosódia semântica para leitura de markdown

- **Slug**: md-semantic-prosody
- **Criado**: 2026-11-07T07:27:10-04:00
- **Entradas usadas**: intake.md | research.md | mapeamento local (`AncientWildfowl`) | pesquisa web consolidada (`CivilianAnglerfish`)
- **Base de evidência**: parcial (relato direto + mapeamento local forte + estado da arte SSML/Piper/acessibilidade; **falta** baseline empírico com Piper `faber` pt-BR e validação de UX/carga cognitiva)

## Declaração do Problema

Usuários que ouvem arquivos `.md` no Ressoa recebem uma leitura TTS majoritariamente uniforme porque o pipeline atual entrega texto plano literal ao motor (Piper `faber` pt-BR via sherpa-onnx, `SSML=None`) e descarta a maior parte da semântica estrutural do markdown; isso reduz a naturalidade e pode dificultar a percepção de estrutura (títulos, ênfase, citações, listas, separadores, perguntas/exclamações) em leitura contínua.

## Usuários Afetados & Partes Interessadas

- **Usuários**: ouvintes de leitura longa (mãos livres, multitarefa), baixa visão/fadiga visual, dislexia/dificuldades de leitura — beneficiados por sinalização prosódica funcional e previsível. [fonte: intake/research; perfis parcialmente inferidos]
- **Usuários sensíveis a ruído**: usuários técnicos ou que preferem fala neutra/rápida — prejudicados por prosódia exagerada, anúncios verbais excessivos ou heurísticas erradas. [ASSUMPTION]
- **Partes interessadas**: mantenedores Ressoa/mdreader; arquitetura Rust (autoridade, determinismo); produto UX/acessibilidade; owners de avaliações relacionadas (`tts-symbol-fatigue`). [fonte: governança/repo]

## Objetivos

- Melhorar a percepção de estrutura e naturalidade da leitura TTS de `.md` com sinalização prosódica **funcional, conservadora e determinística** (prioridade: compreensão/conforto, não teatralidade).
- Preservar invariantes do sistema: Rust autoridade, `SourceRange`/`resume_point`/`segment_index`, fingerprint blake3 do conteúdo original, CLI `chunk` JSON metadata-only, formato mono f32@22050Hz, falhas nomeadas.
- Tratar semântica markdown de forma segura perante ambiguidades (chunker line-scan, fenced code, links/tabelas/malformação), com zonas protegidas e fallback neutro.
- Coordenar com `tts-symbol-fatigue`: não remover pistas estruturais antes de classificá-las; evitar verbalização literal de marcadores e prosódia enganosa.

## Não-objetivos

- Injetar SSML no texto atual nem assumir suporte SSML/phoneme inline sem evolução explícita de capability/engine/contrato (`SSML=None` hoje).
- Emular leitor de tela ou anunciar toda estrutura verbalmente.
- Garantir prosódia “humana perfeita”, controle fino de F0/ênfase por palavra, ou distinção expressiva completa H1–H6 sem evidência de capacidade do engine.
- Reescrever parser CommonMark completo no primeiro corte, salvo necessidade justificada por evidência.
- Cobrir integralmente código, tabelas, HTML, matemática, front matter e markdown estendido no MVP.

## Métricas de Sucesso

- **[BAIXA CONFIANÇA — baseline ausente]** Corpus determinístico pt-BR/markdown: fronteiras estruturais prioritárias (parágrafo, heading, lista, thematic break, blockquote — escopo MVP definido em clarify/spec) produzem sinalização audível funcional (pausa/segmentação/ritmo conservador) sem verbalizar marcadores estruturais literais nem quebrar literais protegidos. (baseline: `segment.content` literal → `tts.generate`, sem plano prosódico)
- **Invariantes**: para mesma entrada/config/regras, saída de metadados de chunk/resume/fingerprint/CLI chunk permanece compatível; mudanças afetam áudio/planejamento de fala de forma determinística e testável. (baseline: testes chunk/CLI/source ranges existentes)
- **Validação empírica**: baseline A/B com Piper `faber` pt-BR (antes/depois) em amostras representativas, registrando modelo/hash/config; avaliação humana curta de compreensão/conforto/fadiga. (baseline: ausente)
- **Segurança semântica**: fenced code/URLs/padrões literais selecionados não recebem prosódia enganosa nem anúncios espúrios. (baseline: pass-through; risco conhecido por heurísticas)

## Custo da Inação

Leitura de markdown permanece monótona e com menor sinalização estrutural, reduzindo conforto/compreensão em documentos longos e limitando a qualidade percebida do Ressoa frente ao objetivo de leitura natural.

## Questões em Aberto

- [NEEDS CLARIFICATION: Baseline empírico Piper `faber` pt-BR: efeito de pontuação, quebras de linha, segmentação por bloco, comprimento de frase e parâmetros suportados por chamada (length/speed/silence, se expostos no binding Rust) sem quebrar determinismo/formato.]
- [NEEDS CLARIFICATION: MVP semântico prioritário — ex.: parágrafo/heading/thematic break/lista/blockquote/código-literal mínimo? Qual nível: `off|conservadora|expressiva`?]
- [NEEDS CLARIFICATION: Estratégia de pausas sem SSML — silêncio PCM entre segmentos/unidades, segmentação, ou transformação textual — e como mapear para âncoras de resume/SourceRange?]
- [NEEDS CLARIFICATION: Política de zonas protegidas (fenced/inline code, links, tabelas, matemática, front matter) e fallback quando chunker misclassifica.]
- [NEEDS CLARIFICATION: Coordenação de pipeline com `tts-symbol-fatigue` (ordem: classificar estrutura → plano prosódico → texto falável/normalização) e critério para avançar `tts-symbol-fatigue` a `go`.]
- [NEEDS CLARIFICATION: Evolução futura aceitável de capability/engine (`SSML`, alinhamento fino, modelo mais expressivo) ou restrição permanente ao adapter atual.]
