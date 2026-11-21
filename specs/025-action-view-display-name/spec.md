# Feature Specification: Nome de exibição no ACTION_VIEW via MediaStore (Ciclo 025)

**Feature Branch**: `025-action-view-display-name`

**Created**: 2026-11-21

**Status**: Draft

**Input**: User description: "Ciclo SDD 025: corrigir o título exibido quando um `.md` é aberto via ACTION_VIEW com URI `content://media/external/file/<id>`. Resolver o nome na coleção MediaStore.Files por `_id`, rejeitar nomes numéricos, montar sempre `effectiveUri = content://display_name/<nome codificado>`, preservar SAF/ACTION_SEND/fingerprint. Latência de 183 s é DEFER."

## Clarifications

### Session 2026-11-21

- Q: Qual é a causa do título `document:1000848919.md`? → A: `queryName` consulta `OpenableColumns.DISPLAY_NAME` no próprio URI do item do MediaStore, que responde `null`. A coluna é a mesma (`_display_name`); o que muda é consultar a coleção `MediaStore.Files` com selection `_id=?`.
- Q: O que acontece quando nenhum nome é resolvido? → A: O ramo `ACTION_VIEW` NÃO devolve mais a URI bruta. A cadeia de fallback é `displayName ?: últimoSegmentoUtilizável ?: "document.md"`, e o URI sintético é montado incondicionalmente.
- Q: Como um identificador numérico é impedido de virar título? → A: Um predicado único de "nome utilizável" rejeita valores nulos, em branco e cujos caracteres são todos dígitos, tanto no resultado da consulta direta quanto no da consulta à coleção e no último segmento da URI.
- Q: O nome precisa ser resolvido no Rust? → A: Não. O Rust já decodifica o último segmento de `content://display_name/…` por `percent_decode` em `picked_document_name`; nenhuma mudança na ponte JNI, no fingerprint ou no núcleo é necessária.
- Q: A latência de 183 s até o primeiro áudio entra neste ciclo? → A: Não. Está registrada como **DEFER** (área de pré-síntese/T232). Este ciclo trata exclusivamente da titulação.
- Q: Que evidência é reivindicada? → A: Inspeção estrutural em host do diff e do consumidor Rust. Comportamento em dispositivo é `[NV]`; gates, builds e testes são executados pelo agente principal, não por esta especificação.

## User Scenarios & Testing *(mandatory)*

<!--
  User stories are prioritized as user journeys. Each is independently testable
  in host by structural inspection; no device claim is made in this specification.
-->

### User Story 1 - Nome correto ao abrir pelo gerenciador de arquivos (Priority: P1)

Quando o ouvinte abre um documento Markdown a partir do app de arquivos do
Android, a interface deve exibir o nome real do arquivo. Hoje, para URIs de item
do MediaStore, o nome não é resolvido e o título é derivado do identificador
numérico do item.

**Why this priority**: é a falha central — sem o nome legível, o documento aberto
não pode ser identificado nem associado ao bookmark correspondente.

**Independent Test**: em host, inspecionar o resolvedor de nome e confirmar que,
após a tentativa direta falhar, uma segunda consulta é feita à coleção
`MediaStore.Files` com selection `_id=?`, projeção da coluna de nome e filtro de
nome utilizável; confirmar que a consulta à coleção só ocorre para URIs
`content://` com authority do MediaStore e com id numérico no último segmento.

**Acceptance Scenarios**:

1. **Given** uma URI `content://media/external/file/<id>` cuja consulta direta a
   `OpenableColumns.DISPLAY_NAME` retorna `null`, **When** o nome é resolvido,
   **Then** a coleção `MediaStore.Files` do volume extraído é consultada por
   `_id=?` e o valor de `_display_name` é usado como nome.
2. **Given** o valor retornado pela coleção é "Hamlet trecho.md", **When** o
   documento é ingerido, **Then** o título final observado no consumidor Rust é
   "Hamlet trecho.md" e não `document:<id>.md`.
3. **Given** uma URI `content://` de outro provedor ou um último segmento que
   não é numérico, **When** o nome é resolvido, **Then** a consulta à coleção
   MediaStore não é feita e o resultado é `null` sem efeito colateral.

---

### User Story 2 - URI sintética estável no ACTION_VIEW (Priority: P1)

O ramo `ACTION_VIEW` deve entregar ao núcleo uma URI que codifique um nome
legível, independentemente de o ContentProvider ter devolvido um nome. A forma
do URI é o contrato já consumido pelo Rust.

**Why this priority**: é a única origem do título no caminho `ACTION_VIEW`;
enquanto a URI bruta puder ser usada como fallback, o identificador numérico
continuará aparecendo na interface.

**Independent Test**: em host, inspecionar o ramo `ACTION_VIEW` e confirmar que
a URI sintética é construída em todos os caminhos de saída, que o nome passa por
codificação de URI e que não resta nenhum retorno à URI original como fallback.

**Acceptance Scenarios**:

1. **Given** um nome resolvido com espaços ou acentos, **When** a URI sintética
   é construída, **Then** ela tem a forma `content://display_name/<nome
   codificado>` e o consumidor Rust a decodifica de volta ao nome original.
2. **Given** nenhum nome utilizável foi resolvido e o último segmento é
   numérico, **When** a URI sintética é construída, **Then** o nome empregado é
   `document.md` — nunca o número e nunca a URI bruta.
3. **Given** a extensão do documento, **When** o nome é derivado, **Then** a
   extensão continua sendo inferida do nome resolvido ou do tipo resolvido,
   preservando `md`, `markdown` ou `txt`.

---

### User Story 3 - Preservação dos caminhos existentes (Priority: P2)

A correção não deve alterar a ingestão via SAF/diretório, o compartilhamento
via `ACTION_SEND`, o cálculo do fingerprint BLAKE3 nem a assinatura da ponte
JNI. A mudança deve ser auditável como um diff pequeno e localizado.

**Why this priority**: esses caminhos já funcionam e são usados por outros
chamadores; uma mudança ampla traria risco de regressão sem benefício para o
problema relatado.

**Independent Test**: em host, comparar o diff e confirmar que ele toca apenas
os imports, o predicado de nome, o resolvedor de nome da URI e o ramo
`ACTION_VIEW`, deixando o resolvedor SAF, o ramo `ACTION_SEND`, a gravação de
snapshot, o hash e a chamada JNI com o mesmo conteúdo.

**Acceptance Scenarios**:

1. **Given** o diff do ciclo, **When** os arquivos alterados são enumerados,
   **Then** apenas o arquivo Kotlin do resolvedor de documentos aparece, sem
   mudanças em Rust, manifesto ou contratos.
2. **Given** um documento recebido via `ACTION_SEND` ou pelo seletor SAF,
   **When** a ingestão ocorre, **Then** o comportamento é idêntico ao anterior
   ao ciclo.
3. **Given** um mesmo conteúdo de arquivo, **When** o snapshot é gravado,
   **Then** o fingerprint BLAKE3 é o mesmo de antes da mudança, pois não depende
   do nome exibido.

---

### User Story 4 - Evidência em host e limites do ciclo (Priority: P2)

A correção deve ser verificável sem dispositivo, por inspeção estrutural do
código e do consumidor Rust, com os limites de verificação declarados
explicitamente.

**Why this priority**: o sintoma foi observado em dispositivo, mas a causa e o
contrato são verificáveis em host; declarar `[NV]` evita reivindicar uma prova
que não foi obtida.

**Independent Test**: em host, percorrer a tabela de rastreabilidade e confirmar
que cada requisito funcional tem uma tarefa e um símbolo de código
correspondentes, que nenhuma afirmação de device é feita e que o item de
latência está marcado como DEFER.

**Acceptance Scenarios**:

1. **Given** os requisitos FR-025-001 a FR-025-005, **When** o relatório de
   análise é lido, **Then** todos estão mapeados a tarefas T001..Tnnn e a
   símbolos reais do código.
2. **Given** a evidência disponível, **When** o resultado é relatado, **Then**
   toda afirmação dependente de dispositivo é marcada `[NV]` e nenhum gate,
   build ou teste é atribuído a esta especificação.
3. **Given** a latência de 183 s registrada no bug, **When** o escopo é
   revisado, **Then** ela aparece como DEFER e nenhuma tarefa deste ciclo a
   endereça.

---

### Edge Cases

- **URI de item sem nome na coleção**: a consulta à coleção retorna `null` ou
  exceção; o predicado rejeita o valor e o fallback `document.md` é usado.
- **Nome somente dígitos**: tanto o resultado da consulta direta quanto o da
  coleção e o último segmento são rejeitados pelo predicado; nenhum número
  chega ao título.
- **Nome em branco ou nulo**: tratado como inutilizável; a cadeia de fallback
  avança para o próximo candidato.
- **URI `content://` de outro provedor**: a consulta à coleção MediaStore é
  ignorada (authority diferente); o nome direto, quando útil, é aproveitado.
- **Último segmento não numérico**: não há consulta à coleção; isso evita
  interpretar um nome de arquivo como id do MediaStore.
- **Volume ausente ou incomum**: o primeiro segmento do caminho é usado como
  volume, com default `external`; falha na consulta degrada para o fallback,
  nunca para a URI bruta.
- **Nome com espaços, acentos ou caracteres especiais**: codificado na URI
  sintética e decodificado pelo consumidor Rust; a codificação é verificada por
  leitura estrutural em host, não em dispositivo.
- **Extensão ausente no nome resolvido**: mantida a inferência por tipo
  resolvido, com `md` como último recurso.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-025-001**: O sistema DEVE resolver o nome de exibição de um documento
  recebido por `ACTION_VIEW` primeiramente consultando
  `OpenableColumns.DISPLAY_NAME` no URI recebido e, quando essa consulta não
  produzir um nome utilizável e o URI for `content://` com authority do
  MediaStore e último segmento numérico, consultando a coleção
  `MediaStore.Files` do volume correspondente com selection `_id=?` e projeção
  da coluna de nome de exibição. A consulta à coleção NÃO DEVE ser feita para
  URIs de outros provedores nem para URIs sem id numérico.

- **FR-025-002**: O ramo `ACTION_VIEW` DEVE entregar ao núcleo uma URI sintética
  da forma `content://display_name/<nome codificado por Uri.encode>`, com
  codificação aplicada ao nome. Essa URI DEVE ser produzida em todos os caminhos
  de saída do ramo, inclusive quando nenhum nome é resolvido. O consumidor Rust
  existente DEVE decodificar o último segmento dessa URI para obter o título,
  sem alteração na ponte JNI.

- **FR-025-003**: Um único predicado de nome utilizável DEVE rejeitar valores
  nulos, em branco e compostos exclusivamente por dígitos. O predicado DEVE ser
  aplicado ao resultado da consulta direta, ao resultado da consulta à coleção
  e ao último segmento da URI quando usado como candidato. Quando nenhum
  candidato é utilizável, o nome DEVE ser `document.md`. A URI original NÃO
  DEVE ser usada como fallback de nome em nenhuma circunstância.

- **FR-025-004**: A mudança DEVE preservar integralmente o resolvedor
  `queryName(resolver, treeUri, docId)` usado por SAF/diretório, o ramo
  `ACTION_SEND`, a gravação de snapshot, o cálculo do fingerprint BLAKE3 e a
  assinatura da chamada JNI. O diff DEVE ficar confinado a um único arquivo
  Kotlin, nas seções de imports, predicado de nome, resolvedor de nome da URI e
  ramo `ACTION_VIEW`. Nenhuma mudança DEVE ser feita em Rust, manifesto,
  contratos ou specs de outros ciclos.

- **FR-025-005**: A verificação DEVE ser realizável em host por inspeção
  estrutural do diff e do consumidor Rust, com rastreabilidade de cada FR a uma
  tarefa e a um símbolo de código. NENHUMA afirmação de comportamento em
  dispositivo DEVE ser feita neste ciclo: toda evidência de device é `[NV]`. A
  latência de 183 s até o primeiro áudio (pré-roll de 60 s) DEVE permanecer
  registrada como **DEFER**, fora do escopo, sem tarefa associada.

### Key Entities *(include if feature involves data)*

- **ResolvedName**: nome de exibição obtido da consulta direta ou da coleção
  MediaStore, submetido ao predicado de nome utilizável antes de ser aceito.
- **EffectiveUri**: URI sintética entregue ao núcleo no lugar da URI original;
  codifica o nome resolvido sob o esquema `content://display_name/`.
- **UsableNamePredicate**: regra única que define o que conta como nome válido,
  aplicada a todos os candidatos antes da construção da `EffectiveUri`.
- **IngestSnapshot**: produto já existente da leitura de bytes e do hash
  BLAKE3; é independente do nome, o que garante a preservação do fingerprint.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-025-001**: 100% dos URIs de item `content://media/<volume>/file/<id>`
  cuja coleção `MediaStore.Files` expõe `_display_name` produzem esse valor como
  nome de exibição (baseline em host: 0%, a consulta direta retornava `null`).
  Mensurável em host pela presença da consulta à coleção com selection `_id=?` e
  pelo filtro aplicado ao resultado.

- **SC-025-002**: 100% dos caminhos de saída do ramo `ACTION_VIEW` constroem a
  URI sintética `content://display_name/<nome codificado>`, com zero ocorrências
  do retorno à URI original como fallback (baseline: 1 ocorrência em host).
  Mensurável por inspeção estrutural do ramo.

- **SC-025-003**: 100% dos nomes aceitos passam pelo predicado de nome
  utilizável, com zero nomes somente dígitos ou vazios propagados ao título; o
  fallback é `document.md` em 100% dos casos sem candidato válido (baseline: o
  identificador numérico virava parte do título).

- **SC-025-004**: zero alterações em SAF/diretório, `ACTION_SEND`, BLAKE3,
  snapshot e assinatura JNI; o diff modifica exatamente 1 arquivo e apenas as
  quatro seções permitidas (baseline em host: 1 arquivo, 36 inserções e 10
  remoções). Mensurável por enumeração do diff em host.

- **SC-025-005**: 100% dos 5 requisitos funcionais mapeados a tarefas e a
  símbolos reais no relatório de análise, com `[NV]` explícito para dispositivo
  e exatamente 1 item marcado como DEFER para a latência de 183 s. Mensurável
  pela tabela de rastreabilidade e pela contagem de marcadores.

> Todas as SCs acima são mensuráveis em host por inspeção estrutural. O nome
> efetivamente exibido e o áudio em dispositivo são `[NV]` e não são
> reivindicados como verificados neste ciclo.

## Assumptions

- O volume inferido do primeiro segmento do caminho da URI, com default
  `external`, é suficiente para os URIs de item do MediaStore encontrados na
  prática; volumes não-padrão degradam para o fallback, nunca para a URI bruta.
- `Uri.encode` no Kotlin e `percent_decode` no Rust formam um par simétrico para
  nomes com espaços e acentos (verificado por leitura estrutural em host;
  dispositivo `[NV]`).
- O predicado de nome utilizável cobre os casos que importam: nulo, em branco e
  somente dígitos.
- `document.md` é um fallback aceitável como último recurso, pois preserva a
  forma de arquivo e evita a exposição do identificador numérico.
- A titulação permanece sendo decidida no Rust a partir da URI original recebida
  via JNI; o Kotlin apenas transporta bytes e codifica o nome nessa URI.
- FR-INT-004 da spec 019 é referência de comportamento; `specs/019` e
  `specs/009` não são editados e nenhum bug ou `converge.md` de outra spec é
  alterado.
- Gates, builds, testes e formatação são executados pelo agente principal;
  nenhuma afirmação sobre o resultado de `scripts/verify.sh` é feita por este
  ciclo documental.
