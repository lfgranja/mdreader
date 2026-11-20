# Decisão: Entonação/prosódia semântica para leitura de markdown

- **Slug**: md-semantic-prosody
- **Decidido**: 2026-11-07T07:27:10-04:00
- **Atualizado**: 2026-11-07T07:31:00-04:00 (defaults 80/20 aplicados por autorização do usuário)
- **Veredito**: go
- **Artefatos revisados**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Critério | Classificação | Justificativa |
|----------|---------------|---------------|
| Validade do problema | strong | Pedido direto; pipeline atual entrega texto literal com `SSML=None` e sem plano prosódico/semântica rica para fala. |
| Força da evidência | adequate | Mapeamento local forte + pesquisa consolidada (SSML/Piper/sherpa/eSpeak/acessibilidade, contra-evidências). Baseline empírico `faber` pt-BR ainda ausente, mas defaults 80/20 + Fase 0 mandatória reduzem risco de especificação cega. |
| Valor vs. inação | adequate | Naturalidade funcional/estrutura pode melhorar conforto em leitura longa; risco de exagero mitigado por default conservador e `off` futuro/config. |
| Viabilidade / apetite | adequate | Opção A faseada viável em Rust, sem SSML, preservando invariantes; escopo MVP conservador. |
| Ajuste estratégico | adequate | Alinha com Rust autoridade/determinismo/acessibilidade; coordena com `tts-symbol-fatigue`. |
| Postura de risco | adequate | Riscos identificados; mitigação: Fase 0 baseline empírico antes de heurísticas amplas, zonas protegidas, defaults conservadores, pipeline ordenado com `tts-symbol-fatigue`, preservação de `SourceRange`/resume/fingerprint/CLI chunk. |

## Veredito & Justificativa

`go` (defaults 80/20). O problema é válido e a Opção A faseada é a menor abordagem defensável sob `SSML=None`. As incógnitas remanescentes são tratadas como **bloqueantes de implementação/Fase 0**, não como impedimento para especificar: o `/speckit.specify` deve registrar defaults 80/20, zonas protegidas, invariantes e a exigência de baseline empírico Piper `faber` pt-BR antes de calibrar regras.

## Defaults 80/20 (autorizados)

- **Nível padrão**: `conservadora` (funcional: estrutura/pausas previsíveis; sem teatralidade). Caminho futuro: `off` e `expressiva` opt-in apenas com evidência.
- **MVP semântico v1 (conservador)**: parágrafo, heading, thematic break, lista, blockquote; perguntas/exclamações preservadas no texto (sem reescrita). Ênfase/código/links/tabelas/front matter/math: zonas protegidas/literalidade neutra por padrão (não adivinhar prosódia).
- **Pausas sem SSML**: preferir estratégia determinística validada no `faber` que preserve `SourceRange`/`resume_point`/`segment_index`/fingerprint (silêncio/segmentação controlada ou transformação textual segura — decisão final após baseline Fase 0).
- **Pipeline com `tts-symbol-fatigue`**: ordem canônica `classificar estrutura (pistas) → derivar plano prosódico → gerar texto falável/normalização de marcadores` com zonas protegidas compartilhadas. Não remover pistas antes da classificação.
- **Evolução**: manter `SSML=None`/capabilities atuais no MVP; avaliar capability/engine/SSML parcial apenas em fase futura com evidência cross-device.
- **Zonas protegidas v1**: fenced/inline code (melhor esforço sob chunker line-scan não fence-aware), URLs/paths, padrões numéricos/literais simples (`2*3`, `*.ext` onde seguro), tabelas/front matter/HTML — fallback neutro.

## Se go — entrega a `/speckit.specify`

- **Problema**: leitura TTS uniforme em `.md`; semântica estrutural não gera sinalização prosódica funcional.
- **Abordagem escolhida**: Opção A faseada — plano prosódico determinístico conservador em Rust no boundary de fala, sem SSML, coordenado com `tts-symbol-fatigue`.
- **Dentro / fora de escopo**: dentro — Fase 0 baseline `faber` pt-BR/corpus, MVP estrutural conservador, zonas protegidas, pipeline ordenado com `tts-symbol-fatigue`, testes/invariantes, defaults 80/20; fora — SSML/engine swap imediato, prosódia expressiva fina por palavra, parser completo, suporte total a estruturas avançadas.
- **Métricas de sucesso**: baseline antes/depois; invariantes `SourceRange`/resume/fingerprint/CLI chunk preservados; melhora percebida de estrutura sem fadiga/artefatos; zonas protegidas seguras; sem verbalização literal de marcadores-alvo quando coberta por `tts-symbol-fatigue`.
- **Incógnitas carregadas adiante (especificar como tarefas/clarificações)**: capacidade real do binding por chamada; mecanismo de pausa validado; parsing seguro sob line-scan; matriz de interação detalhada com `tts-symbol-fatigue`; configuração de usuário (`off/conservadora`).
