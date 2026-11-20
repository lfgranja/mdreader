# Concept: Entonação/prosódia semântica para leitura de markdown

- **Slug**: md-semantic-prosody
- **Criado**: 2026-11-07T07:27:10-04:00
- **Opção recomendada**: Opção A (faseada)

## Opções

### Opção A — Plano prosódico determinístico conservador no boundary de síntese (Rust, sem SSML)
- **Esboço**: Introduzir camada pura em Rust que classifica pistas estruturais disponíveis com segurança (inicialmente metadados já confiáveis + heurísticas conservadoras), deriva um plano de fala determinístico e produz texto falável/pausas controladas para o Piper `faber`. Prioriza **segmentação funcional** (pausas/fronteiras por parágrafo/heading/thematic break/lista, conforme validado) e preserva literalidade em zonas protegidas. Não injeta SSML; não altera `SourceRange`/`resume_point`/`segment_index`/fingerprint/capabilities declaradas no MVP.
- **Apetite**: medium (semanas; exige baseline empírico + corpus + coordenação com `tts-symbol-fatigue`)
- **Trade-offs**:
  - **Ganha**: alinhado a Rust autoridade/determinismo/`SSML=None`; menor risco contratual se confinado ao planejamento de fala; permite progresso incremental e defaults conservadores.
  - **Sacrifica**: expressividade limitada (especialmente ênfase/F0/H1–H6) sem SSML/controles por segmento; depende de comportamento empírico do `faber`; heurísticas frágeis se escopo crescer.
- **Buracos de coelho**: pausas sem SSML que respeitem resume/alinhamento; chunker line-scan não fence-aware; conflito de ordem com normalização de símbolos; microvariações (`length_scale/noise`) podem ser globais/não determinísticas percebidas ou indisponíveis por chamada.

### Opção B — Evolução de capability/engine para suporte prosódico explícito (SSML parcial ou engine expressiva)
- **Esboço**: Evoluir adapter/capabilities para um motor ou trilha que suporte subconjunto confiável (`break`, `emphasis`, `say-as`, talvez `prosody`) ou usar modelo/voz mais expressivo com controles. Rust gera marcação/plano compatível com o contrato do engine.
- **Apetite**: large
- **Trade-offs**:
  - **Ganha**: caminho mais direto para prosódia semântica rica e interoperável se bem suportado.
  - **Sacrifica**: alto blast radius (capabilities, engine-core/contratos fechados, testes, Android/offline, tamanho/licença/latência, determinismo cross-device); risco de suporte SSML inconsistente.
- **Buracos de coelho**: binding sherpa-rs/engine real não suportar SSML; custo operacional; migração de `AlignmentGranularity`/boundaries; validação em dispositivos.

### Opção C — Não fazer / manter fala uniforme (ou apenas mitigação de símbolos)
- **Esboço**: Não implementar prosódia semântica; concentrar apenas em `tts-symbol-fatigue`/naturalidade básica do texto falável.
- **Apetite**: small
- **Trade-offs**:
  - **Ganha**: zero risco novo de heurística prosódica; entrega valor parcial mais rápido.
  - **Sacrifica**: não atende pedido de entonação/postura por semântica markdown; leitura pode permanecer monótona mesmo sem “asterisco, asterisco”.
- **Buracos de coelho**: frustração de expectativa; retrabalho se prosódia for exigida depois.

## Recomendação

**Opção A, faseada**, condicionada a baseline empírico. Tratar “leitura natural humana” como objetivo de **naturalidade funcional** (clareza estrutural, pausas previsíveis, sem ruído) no MVP, não imitação expressiva. Sequência recomendada: **Fase 0** baseline Piper `faber` pt-BR + corpus + contrato de invariantes; **Fase 1** pipeline Rust puro `estrutura→plano prosódico→texto falável/pausas` coordenado com `tts-symbol-fatigue`, zonas protegidas e defaults conservadores; **Fase 2** apenas se binding permitir controles determinísticos por unidade; **Fase 3** reavaliar capability/engine/SSML com evidência. Não promover a `go` de implementação ampla sem clarificações/baseline.

## Fora do Escopo (para a opção recomendada)

- SSML no pipeline atual; troca imediata de engine/modelo/voz.
- Prosódia expressiva por palavra/ênfase fina sem controle suportado.
- Parser markdown completo/AST como pré-requisito do MVP, salvo prova de necessidade.
- Suporte completo a tabelas, HTML, matemática, front matter, diagramas.
- Configuração UI avançada no primeiro corte (pode ser flag/config interna documentada).

## Suposições a Validar

- O Piper `faber` pt-BR produz pausas/naturalidade aceitáveis com segmentação e pontuação/forma textual controladas deterministicamente.
- Pistas estruturais suficientes podem ser obtidas sem quebrar zonas literais, apesar do chunker line-scan.
- Pausas (silêncio ou boundaries) podem ser implementadas sem corromper `SourceRange`/resume/fingerprint.
- Usuários preferem prosódia conservadora configurável a variação agressiva por padrão.
