# Feature Specification: Retomada de Diretório na Reabertura (Ciclo 024)

**Feature Branch**: `024-directory-resume-offer`

**Created**: 2026-11-20

**Status**: Draft

**Input**: User description: "Ciclo SDD 80/20 #024 — T233: ligar o diálogo de retomada na reabertura de diretório. Fazer a cola completa: rota InAppDialog publicada, filename/offset no payload, route_offer uma vez, refreshOffer no JS, teste comportamental no target já pinado, byte-identity dos bundles."

## Clarifications

### Session 2026-11-20

- Q: A janela de 3 s e o auto-continue em primeiro plano são verificáveis neste ciclo? → A: Não em dispositivo; apenas em host, por constantes e transições da máquina de oferta. Toda afirmação de device é `[NV]` e não é reivindicada.
- Q: A rota de oferta precisa ser re-derivada no loop de poll para cobrir o expiry? → A: Não. A máquina de oferta é mutata e resolve `expiry → AutoStart`; a rota é publicada uma única vez na reabertura e nunca re-derivada em poll.
- Q: A posição salva deve ser copiada para a interface como nova origem de verdade? → A: Não. O payload deriva do bookmark ativo; a interface apenas o exibe, sem segundo estado de posição.
- Q: Um novo target de teste é necessário para a evidência comportamental? → A: Não. O target `src-tauri::test_directory_resume` já está pinado em `scripts/expected-tests.txt:113`; o teste existente é reescrito e o pin permanece intacto.
- Q: A mudança na interface exige alteração de marcação HTML? → A: Não. O JS passa a ler os campos já presentes no payload e a chamar a atualização da oferta; os cinco bundles são espelhados byte a byte.

## User Scenarios & Testing *(mandatory)*

<!--
  User stories are prioritized as user journeys. Each is independently testable
  in host; no device claim is made in this specification.
-->

### User Story 1 - Oferta de retomada alcançável na reabertura (Priority: P1)

Quando o ouvinte reabre uma pasta que estava com leitura em andamento, o sistema
deve apresentar a oferta de retomada no mesmo prazo e com as mesmas alternativas
já definidas para a retomada em geral. Hoje a oferta não aparece porque o caminho
de reabertura não publica a rota de diálogo e não estaciona a retomada.

**Why this priority**: é a falha central — sem a rota publicada, nenhum outro
comportamento de retomada é alcançável pelo usuário.

**Independent Test**: em host, com um bookmark `Paused` já persistido, invocar o
caminho de reabertura e observar que a rota publicada é `in_app_dialog` e que a
janela configurada é de 3.000 ms; verificar que a máquina de oferta foi resolvida
exatamente uma vez.

**Acceptance Scenarios**:

1. **Given** um bookmark de diretório com `end_kind = Paused`, **When** a pasta é
   reaberta, **Then** a rota publicada é a do diálogo in-app e a oferta permanece
   estacionada até a resposta do usuário.
2. **Given** a oferta publicada, **When** os 3.000 ms decorrem com a janela em
   primeiro plano, **Then** a transição para auto-continue é produzida pela mesma
   resolução da máquina de oferta, sem re-derivação de rota em poll.
3. **Given** a oferta publicada, **When** o usuário escolhe "Continuar" ou
   "Começar do início", **Then** a resposta é aplicada à posição salva sem criar
   uma segunda origem de estado.

---

### User Story 2 - Contexto da retomada visível na interface (Priority: P1)

A oferta deve indicar o nome do arquivo ativo e o ponto de reprodução salvo.
Esses dados já vêm do bookmark, mas não chegam ao diálogo: o campo de nome é lido
pela interface e nunca recebe valor.

**Why this priority**: sem nome e posição, a escolha entre retomar e recomeçar não
é informada, o que anula o valor da oferta mesmo quando ela aparece.

**Independent Test**: em host, disparar o evento de seleção de diretório com o
payload enriquecido e verificar que os campos de nome e offset são atribuídos ao
estado da interface antes da atualização da oferta, com offset igual ao do
bookmark.

**Acceptance Scenarios**:

1. **Given** um bookmark com capítulo ativo e offset salvo, **When** o evento de
   seleção de diretório é tratado, **Then** o payload traz nome de arquivo não
   vazio e offset igual ao do bookmark.
2. **Given** o payload enriquecido, **When** a interface o consome, **Then** os
   valores são atribuídos ao estado e a oferta é atualizada em seguida.
3. **Given** a mudança na interface, **When** os cinco bundles HTML são
   comparados em host, **Then** todos são byte-idênticos entre si.

---

### User Story 3 - Evidência comportamental no target já pinado (Priority: P2)

O teste que cobre a reabertura deve medir o comportamento observável — rota
publicada, payload entregue e resolução da oferta — em vez de verificar apenas que
determinadas strings aparecem no código-fonte.

**Why this priority**: sem evidência comportamental, a regressão desta cola pode
voltar a acontecer sem que o teste pinado acuse nada.

**Independent Test**: em host, executar o target de teste já pinado e verificar
que as asserções falham se a rota não for publicada ou se o payload não contiver
nome e offset; confirmar que nenhum novo target foi criado.

**Acceptance Scenarios**:

1. **Given** o teste existente no target pinado, **When** ele é executado, **Then**
   afirma a rota publicada e o conteúdo do payload, não a presença literal de
   trechos de fonte.
2. **Given** a suíte de testes, **When** `scripts/expected-tests.txt` é comparado
   antes e depois, **Then** não há linhas adicionadas ou removidas.

---

### Edge Cases

- Bookmark `Paused` cujo offset salvo é zero: a rota ainda é publicada e o offset
  exibido é `0`, sem ausência de campo.
- Reabertura sem bookmark ativo: nenhuma oferta é publicada e o comportamento
  permanece o de início normal do diretório.
- Expiry da janela sem janela em primeiro plano: não há auto-continue; a
  oferta permanece aguardando interação, conforme a regra já existente.
- Reaberturas repetidas: a máquina de oferta é resolvida uma vez por reabertura,
  sem acumular transições de expiry.
- Mudança na interface sem espelhar um dos bundles: considerada falha de
  integridade, não diferença cosmética.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-024-001**: O sistema DEVE publicar a rota do diálogo de retomada in-app
  quando uma pasta é reaberta com bookmark em andamento (`Paused`), de modo que a
  oferta deixe de ser resolvida como `none` e permaneça estacionada até a resposta
  do usuário. A publicação DEVE ocorrer no caminho de reabertura logo após a
  instalação da fila de capítulos, e os marcadores de código morto que impediam
  esse uso DEVEM ser removidos.
- **FR-024-002**: O payload do evento de seleção de diretório DEVE conter o nome
  do arquivo ativo e o offset salvo, ambos derivados do bookmark ativo, sem criar
  segunda origem de verdade para a posição. Nenhum campo DEVE ficar ausente ou
  vazio quando existe bookmark correspondente.
- **FR-024-003**: A máquina de oferta DEVE ser resolvida exatamente uma vez por
  reabertura, no mesmo caminho que publica a rota. A transição de expiração para
  auto-continue DEVE continuar sendo decidida por essa resolução (que muta
  estado), sendo proibido re-derivar a rota no loop de poll.
- **FR-024-004**: A interface DEVE atribuir o nome do arquivo e o offset salvo a
  partir do payload do evento de seleção de diretório e, em seguida, atualizar a
  oferta. A atribuição DEVE ocorrer antes da atualização, e nenhuma alteração de
  marcação HTML DEVE ser necessária.
- **FR-024-005**: O teste de reabertura no target já pinado DEVE substituir as
  asserções de presença literal de strings por asserções comportamentais sobre a
  rota publicada e sobre o conteúdo do payload, incluindo a janela de 3.000 ms e a
  resolução única da máquina de oferta. NENHUM novo target de teste DEVE ser
  criado e `scripts/expected-tests.txt` NÃO DEVE ser alterado.
- **FR-024-006**: O arquivo-fonte da interface e seus quatro espelhos DEVEM ser
  byte-idênticos após a mudança: `src-tauri/src/status_view.html`,
  `dist/index.html`, `src-tauri/src/assets/index.html`, `src-tauri/dist/index.html`
  e `src-tauri/gen/android/app/src/main/assets/index.html`. A identidade DEVE ser
  verificável por comparação de bytes em host.

### Key Entities *(include if feature involves data)*

- **DirectoryResume**: contexto calculado na instalação da fila de capítulos;
  contém a rota de oferta (`in_app_dialog` para `Paused`) e, após a cola, o nome do
  arquivo ativo e o offset salvo originados do bookmark.
- **StoredPosition**: posição previamente persistida (índice do capítulo e offset
  em segundos), usada como única origem para a retomada e para o payload da
  interface.
- **PublishedOfferRoute**: rota publicada no caminho de reabertura, observável
  pelo teste e consumida pela interface para exibir a oferta.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-024-001**: 100% dos bookmarks `Paused` submetidos ao caminho de reabertura
  no teste em host resultam em rota de diálogo publicada e janela de 3.000 ms
  (baseline em host: 0%, rota `none`).
- **SC-024-002**: 100% dos payloads observados no teste em host contêm nome de
  arquivo não vazio e offset igual ao bookmark (baseline: campos ausentes).
- **SC-024-003**: exatamente 1 resolução da máquina de oferta por reabertura no
  teste em host, com zero re-derivações de rota em poll (baseline: 0 no caminho de
  reabertura).
- **SC-024-004**: 5 de 5 superfícies HTML são byte-idênticas após a mudança,
  verificadas por comparação de bytes em host (baseline: 5 de 5 idênticas).
- **SC-024-005**: zero linhas adicionadas ou removidas em
  `scripts/expected-tests.txt` e zero novos targets de teste (baseline: pin
  `src-tauri::test_directory_resume` na linha 113).
- **SC-024-006**: 100% das atribuições de nome e offset na interface ocorrem antes
  da atualização da oferta no tratador do evento de seleção de diretório,
  verificável por inspeção estrutural do arquivo-fonte em host.

> Todas as SCs acima são mensuráveis em host. A experiência visual da janela de
> 3 s e o auto-continue em primeiro plano em dispositivo real são `[NV]` e não
> são reivindicadas como verificadas neste ciclo.

## Assumptions

- `handle_directory_selected` é o único caminho de reabertura que precisa
  publicar a rota neste ciclo; outros ingressos, se descobertos, serão tratados
  como escopo adicional explícito.
- Existe, ou pode ser criada sem novo target, uma seam no bridge para observar a
  rota publicada e o payload no teste já pinado.
- A máquina de oferta existente permanece a única autoridade para expiração e
  escolhas do usuário; a interface apenas consome o que ela publica.
- Os quatro espelhos HTML aceitam a mesma alteração byte a byte do arquivo-fonte,
  preservando a auditoria C-H8.
- FR-009 da spec 010 é referência de comportamento; `specs/010` não é editado e
  T232/T233 não são renumerados.
