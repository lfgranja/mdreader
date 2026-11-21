# Feature Specification: Primeiro segmento útil para reduzir TTFA (Ciclo 026)

**Feature Branch**: `026-first-useful-segment`

**Created**: 2026-11-21

**Status**: Draft

**Input**: User description: "Ciclo SDD 026: reduzir o tempo até o primeiro áudio em início fresco. `prepare_session_document` (`src-tauri/src/lib.rs:6706`) usa `split_document` com teto de 5000 bytes; Hamlet de ~3,9 KB vira um segmento e o handover de 013 só ocorre depois dessa unidade inteira ser sintetizada (183 s observados). Emitir um primário pequeno no início fresco, preservar retomada com offset, índices, metadados e o contrato mono f32 22050. Testes no target pinado; nenhuma meta absoluta de TTFA; dispositivo `[NV]`."

## Clarifications

### Session 2026-11-21

- Q: O que exatamente é um "início fresco"? → A: `start_offset_secs == 0.0`. Qualquer valor maior que zero é retomada e não recebe primário.
- Q: Como dividir sem quebrar palavras? → A: o primário termina no último whitespace antes de 160 bytes. Se não houver whitespace antes do limite, o primeiro segmento permanece intacto — nunca se corta no meio de uma palavra.
- Q: O que acontece com os demais segmentos? → A: o restante do primeiro segmento vira a unidade seguinte e todos os índices subsequentes são reindexados, mantendo a sequência estrita de `segment_index` e herdando `source_range`, `boundary_kind` e `resume_point` do segmento de origem.
- Q: A retomada muda? → A: Não. Com `start_offset_secs > 0.0`, a segmentação é exatamente a atual; nenhum primário é acrescentado.
- Q: O sintetizador ou o contrato de áudio mudam? → A: Não. PCM mono `f32` @ 22050 Hz, `synth.rs`, `chunk.rs`, `Cargo.toml`, HTML, capabilities e workflow permanecem como estão.
- Q: Onde ficam os testes? → A: no módulo inline `ttfa_013_tests` (`src-tauri/src/lib.rs:10657`), dentro do target já pinado. Nenhum novo target e nenhuma alteração em `scripts/expected-tests.txt`.
- Q: Qual é a meta de TTFA? → A: estritamente relativa — o tempo até o primeiro áudio no início fresco deve ser menor que o do baseline pareado. Nenhuma meta numérica absoluta é estabelecida; captura em dispositivo é `[NV]` e gates/builds/testes são executados pelo orquestrador, não por esta cadeia documental.

## User Scenarios & Testing *(mandatory)*

<!--
  Histórias priorizadas por jornada do ouvinte. Cada uma é testável de forma
  independente em host por inspeção estrutural; nenhuma afirmação de
  comportamento em dispositivo é feita nesta especificação.
-->

### User Story 1 - Primeiro áudio chega antes da síntese integral (Priority: P1)

Quando o ouvinte inicia a reprodução de um documento do começo, ele deve ouvir
áudio assim que uma pequena parte legível do texto estiver pronta, em vez de
esperar a síntese completa do documento. Hoje um documento de ~3,9 KB gera uma
única unidade e o primeiro áudio só chega depois dessa unidade inteira.

**Why this priority**: é a falha central — sem ouvir nada, o ouvinte não tem
confirmação de que o play funcionou; a latência observada foi de 183 s.

**Independent Test**: em host, inspecionar o preparador de sessão e confirmar
que, com `start_offset_secs == 0.0` e primeiro segmento maior que 160 bytes, a
primeira unidade entregue à produção tem no máximo 160 bytes e termina em
whitespace, e que o restante segue como unidade imediatamente posterior.

**Acceptance Scenarios**:

1. **Given** um início fresco e um primeiro segmento maior que 160 bytes,
   **When** a sessão é preparada, **Then** a primeira unidade de síntese tem no
   máximo 160 bytes e sua fronteira é um whitespace, nunca o meio de uma
   palavra.
2. **Given** o primário produzido, **When** a produção avança, **Then** o
   handover existente (`handover_pending_for_generation`) enfileira e inicia o
   backend com essa primeira peça, sem nova lógica de sincronização.
3. **Given** o mesmo documento e as mesmas condições de captação, **When** o
   TTFA do início fresco é comparado ao baseline de unidade integral, **Then**
   ele é menor; a medição em dispositivo é `[NV]` neste ciclo.

---

### User Story 2 - Retomada preserva o comportamento atual (Priority: P1)

Quando o ouvinte retoma a reprodução a partir de uma posição armazenada, a
segmentação e o timeline devem continuar exatamente como hoje. O primário é um
recurso exclusivo do início fresco.

**Why this priority**: retomada, bookmarks e offsets são contratos já
estáveis; uma mudança aqui trocaria um ganho de TTFA por regressão de posição.

**Independent Test**: em host, inspecionar os dois chamadores do preparador e
confirmar que o caminho de estacionamento a frio nunca solicita primário e que
o caminho de início só o solicita quando `start_offset_secs == 0.0`.

**Acceptance Scenarios**:

1. **Given** `start_offset_secs > 0.0`, **When** a sessão é preparada, **Then**
   a lista de segmentos é idêntica à produzida pela segmentação atual, sem
   primário adicional e sem nova contagem de unidades.
2. **Given** um documento que seria elegível a primário em início fresco,
   **When** ele é apenas estacionado para uma oferta de retomada, **Then**
   nenhuma síntese ou segmentação primária ocorre e o payload de espera é o
   atual.
3. **Given** `start_offset_secs == 0.0`, **When** o início é preparado, **Then**
   o primário é aplicado; a distinção entre os dois caminhos é explícita.

---

### User Story 3 - Integridade de segmentos e do contrato de áudio (Priority: P2)

A divisão do primeiro segmento não pode alterar a ordem dos índices, os
metadados de origem nem o formato do PCM entregue ao backend.

**Why this priority**: são invariantes usados por progresso, seções,
`resume_point` e pelo consumidor de áudio; quebrá-los causaria regressões
silenciosas em vários ciclos.

**Independent Test**: em host, inspecionar a construção dos segmentos
resultantes e confirmar `segment_index` estritamente sequencial, metadados
herdados do segmento de origem e nenhuma alteração em `synth.rs`, `chunk.rs`,
`Cargo.toml` ou nos contratos de configuração.

**Acceptance Scenarios**:

1. **Given** um primeiro segmento dividido em primário + restante, **When** a
   lista final é observada, **Then** os índices são 0, 1, 2… sem lacunas e os
   demais campos dos dois novos segmentos preservam os valores do segmento
   original.
2. **Given** a cadeia de síntese, **When** o áudio é enfileirado, **Then** ele
   continua sendo mono `f32` @ 22050 Hz, com o mesmo produtor limitado e o
   mesmo mecanismo de geração de 013.
3. **Given** o diff do ciclo, **When** os arquivos alterados são enumerados,
   **Then** nenhum arquivo de configuração de teste, target, HTML,
   capabilities, workflow, `synth.rs` ou `chunk.rs` aparece.

---

### User Story 4 - Evidência em host e limites do ciclo (Priority: P2)

O mecanismo deve ser verificável no target de testes já existente, com os
limites de verificação declarados explicitamente.

**Why this priority**: o sintoma foi visto em dispositivo, mas a causa e o
contrato são verificáveis em host; declarar `[NV]` evita reivindicar uma prova
que não foi obtida neste checkout.

**Independent Test**: em host, percorrer a tabela de rastreabilidade e
confirmar que cada FR tem tarefa e símbolo correspondentes, que os testes estão
em `ttfa_013_tests`, que nenhuma meta absoluta de TTFA aparece e que toda
afirmação de dispositivo está marcada `[NV]`.

**Acceptance Scenarios**:

1. **Given** os requisitos FR-026-001 a FR-026-006, **When** o relatório de
   análise é lido, **Then** todos estão mapeados a tarefas T001..Tnnn e aos
   símbolos reais do código.
2. **Given** os testes novos, **When** o target é enumerado, **Then** eles
   aparecem no módulo inline `ttfa_013_tests` e `scripts/expected-tests.txt`
   permanece byte-idêntico.
3. **Given** a evidência disponível, **When** o resultado é relatado, **Then**
   toda afirmação dependente de dispositivo é `[NV]`, nenhum gate/build/teste é
   atribuído a esta especificação e nenhuma meta numérica absoluta de TTFA é
   definida.

---

### Edge Cases

- **Primeiro segmento já pequeno** (≤160 bytes): nenhuma divisão; a lista de
  segmentos é a original mesmo em início fresco.
- **Sem whitespace antes de 160 bytes**: não se corta no meio da palavra; a
  segmentação original é preservada e a ausência de primário é registrada como
  comportamento esperado.
- **Caracteres multibyte**: o limite de 160 é contado em bytes UTF-8, nunca em
  caracteres; a fronteira escolhida é um whitespace válido antes desse limite.
- **Primeiro segmento com heading**: a divisão não pode separar um cabeçalho do
  texto que ele introduz além do previsto pela regra de whitespace; os
  metadados de fronteira são herdados do segmento de origem.
- **Documento vazio**: o preparador continua devolvendo o erro de ausência de
  segmentos; nenhum primário é criado.
- **Retomada com offset muito alto**: permanece sem primário;
  `samples_before_entry` e o timeline atual não são recalculados por este
  ciclo.
- **Cold-start parked**: somente prepara a estrutura; nunca sintetiza nem cria
  primário.
- **Mudança de documento/g geração**: o primário é gerado no preparador, antes
  da geração; os guardas de geração de 013 continuam protegendo enfileiramento e
  start do backend.
- **Múltiplas aberturas do mesmo documento**: cada início fresco aplica a mesma
  regra determinística; nenhum estado persistido é escrito.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-026-001**: Para um início fresco (`start_offset_secs == 0.0`) cujo
  primeiro segmento excede 160 bytes, o sistema DEVE emitir um primeiro
  segmento "primário" com no máximo 160 bytes, cuja fronteira seja o último
  whitespace dentro desse limite. Quando não houver whitespace antes do limite,
  o sistema NÃO DEVE dividir o segmento. O limite é medido em bytes UTF-8.

- **FR-026-002**: O sistema DEVE entregar o primário como primeira unidade da
  produção, de modo que o mecanismo de handover já existente enfileire e inicie
  o backend com ele antes de sintetizar o restante. NENHUMA nova lógica de
  sincronização, geração ou inicialização de backend DEVE ser criada.

- **FR-026-003**: Quando `start_offset_secs > 0.0`, o sistema NÃO DEVE aplicar
  o primário: a segmentação, a contagem de unidades e o timeline devem ser
  idênticos aos atuais. O caminho de estacionamento a frio também NÃO DEVE
  solicitar primário.

- **FR-026-004**: Após a divisão, o restante do antigo primeiro segmento DEVE
  virar a unidade seguinte e todos os segmentos DEVEM ter `segment_index`
  estritamente sequencial, sem lacunas. `source_range`, `boundary_kind` e
  `resume_point` dos dois segmentos resultantes DEVEM ser herdados do segmento
  de origem.

- **FR-026-005**: O contrato de áudio NÃO DEVE mudar: o PCM continua mono
  `f32` @ 22050 Hz com o mesmo produtor limitado e os mesmos guardas de
  geração. NENHUMA mudança DEVE ser feita em `synth.rs`, `chunk.rs`,
  `Cargo.toml`, `scripts/expected-tests.txt`, HTML, capabilities ou workflow,
  e NENHUM novo target de teste DEVE ser criado.

- **FR-026-006**: Os testes do ciclo DEVEM residir no módulo inline
  `ttfa_013_tests` do target já pinado e DEVEM cobrir primário em início
  fresco, ausência de primário na retomada, sequência de índices e metadados
  herdados. Toda evidência de comportamento em dispositivo DEVE ser marcada
  `[NV]`; gates, builds e testes são executados pelo orquestrador, não por
  esta especificação.

### Key Entities *(include if feature involves data)*

- **Primário**: primeira unidade de síntese de um início fresco, no máximo 160
  bytes e terminada em whitespace, extraída do antigo primeiro segmento.
- **Restante**: texto do antigo primeiro segmento após o primário, emitido como
  unidade imediatamente posterior.
- **DocumentSegment**: unidade já existente, com `content`, `source_range`,
  `boundary_kind`, `segment_index` e `resume_point`; os metadados são
  preservados quando a unidade é dividida.
- **Handover**: mecanismo de 013 que enfileira a primeira peça produzida e
  inicia o backend; é reutilizado, não reimplementado.
- **TTFA relativo**: comparação do tempo até o primeiro áudio entre o início
  fresco com primário e o baseline de unidade integral na mesma condição —
  nunca um valor absoluto fixo.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-026-001**: Sob as mesmas condições de documento e captação, o tempo até
  o primeiro áudio no início fresco é menor que o do baseline de segmento
  integral (baseline histórico: 183089 ms em 2201117TY). Em host, mensurável
  pela redução da primeira unidade de síntese de ~3,9 KB para ≤160 bytes; a
  medição pareada em dispositivo é `[NV]`. NÃO há meta numérica absoluta.

- **SC-026-002**: 100% dos inícios frescos com primeiro segmento maior que 160
  bytes e whitespace disponível produzem um primário ≤160 bytes terminado em
  whitespace; zero divisões no meio de uma palavra. Quando não há whitespace
  antes do limite, 100% dos casos preservam a segmentação original.

- **SC-026-003**: 100% das retomadas com `start_offset_secs > 0.0` e 100% dos
  estacionamentos a frio mantêm a lista de segmentos original, com zero
  primários adicionais e zero mudanças na contagem de unidades.

- **SC-026-004**: 100% dos segmentos após a divisão têm `segment_index`
  sequencial e metadados herdados corretamente; zero lacunas ou campos
  divergentes em relação ao segmento de origem.

- **SC-026-005**: 100% do contrato PCM inalterado (mono, `f32`, 22050 Hz) e zero
  mudanças em `synth.rs`, `chunk.rs`, `Cargo.toml`, `expected-tests.txt`, HTML,
  capabilities e workflow; 100% dos testes novos no módulo
  `ttfa_013_tests` e nenhum target novo.

- **SC-026-006**: 100% dos 6 requisitos funcionais rastreados a tarefas e
  símbolos reais no relatório de análise, checklist de requisitos 23/23 itens
  `[x]`, zero metas numéricas absolutas de TTFA e `[NV]` explícito para toda
  afirmação de dispositivo.

> Todas as SCs são mensuráveis em host por inspeção estrutural ou por testes no
> target pinado. O TTFA efetivamente observado em dispositivo é `[NV]` e não é
> reivindicado como verificado neste ciclo.

## Assumptions

- Existe whitespace antes de 160 bytes nos documentos representativos; quando
  não existir, preservar o segmento original é preferível a quebrar uma palavra.
- `start_offset_secs == 0.0` identifica um início fresco e valores maiores que
  zero identificam retomada, inclusive no caminho parked.
- A parcela do TTFA dominada pela síntese da primeira unidade é relevante no
  caso reportado; a parcela fixa de inicialização do modelo pode limitar o
  ganho e será avaliada em captura pareada, quando disponível (`[NV]`).
- O módulo `ttfa_013_tests` aceita novos testes sem alterar a lista canônica de
  testes, respeitando o gate 2 de diff bidirecional.
- Os guardas de geração de 013 continuam válidos: o primário é uma decisão de
  segmentação e não substitui invalidação, cancelamento ou start do backend.
- `specs/013`, `specs/025`, bugs e `converge.md` de outras specs são dados de
  contexto; nenhum deles é editado por este ciclo.
- Gates, builds, testes e formatadores são executados pelo agente principal;
  nenhuma afirmação sobre o resultado de `scripts/verify.sh` é feita por esta
  cadeia documental.

