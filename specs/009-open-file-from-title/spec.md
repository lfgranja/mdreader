# Feature Specification: Abrir Arquivo Pelo Título e Iniciar Reprodução

**Feature Branch**: `009-open-file-from-title`

**Created**: 2026-11-02

**Status**: Draft

**Input**: User description: "Clicar no botão de título para abrir um arquivo compatível e já inicializar tocando."

## Clarifications

### Session 2026-11-02
- Q: Como deve ser delimitada a área de toque e a affordance interativa para o disparo do seletor de arquivos na Lente Sinestésica? → A: Option A — Container `.meta-box` completo atua como botão com alvo de toque amplo (>48dp), `role="button"`, `tabindex="0"`, `cursor: pointer`, animação suave no `:active` (scale 0.985) e `aria-label="Selecionar documento para leitura"`.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Selecionar Novo Documento e Iniciar Reprodução Imediata (Priority: P1)

Como ouvinte de audiolivros e textos no Ressoa,
Quero tocar no bloco de título do documento na Lente Sinestésica para escolher um arquivo compatível (`.md`, `.txt`) no meu dispositivo e ter a reprodução iniciada imediatamente,
Para que eu possa começar a escutar meu texto com o menor número possível de passos e sem atrito na tela.

**Why this priority**: É o fluxo principal e a entrega de valor central da funcionalidade. Elimina a barreira de entrada para que o usuário utilize o aplicativo com seus próprios arquivos de forma fluida.

**Independent Test**:
Pode ser testado abrindo o aplicativo em estado ocioso/pausado, tocando na caixa de título, escolhendo um arquivo `.md` válido pelo seletor do sistema (SAF) e verificando que a tela atualiza o título do documento e o áudio começa a tocar automaticamente.

**Acceptance Scenarios**:

1. **Given** que o aplicativo está aberto e visível na Lente Sinestésica, **When** o usuário toca no bloco de título (`.meta-box`), **Then** o seletor de arquivos nativo do sistema operacional é apresentado filtrando arquivos de texto compatíveis (`text/markdown`, `text/plain`).
2. **Given** que o seletor de arquivos está aberto, **When** o usuário seleciona um arquivo compatível sem histórico prévio de leitura, **Then** o documento é carregado, o título e as seções são atualizados na interface visual e a reprodução (síntese e saída de áudio) é iniciada imediatamente.

---

### User Story 2 - Respeito à Política de Retomada em Documentos Já Lidos (Priority: P2)

Como ouvinte que já iniciou e pausou a leitura de um documento anteriormente,
Quero que ao selecionar novamente esse mesmo arquivo pelo seletor de títulos o aplicativo respeite meu progresso salvo,
Para que eu não perca a minha posição de leitura nem seja forçado a ouvir o áudio desde o começo involuntariamente.

**Why this priority**: Preserva a integridade dos dados e a intenção do usuário conforme os princípios normativos de sessão e contratos de persistência do Ressoa (FR-009).

**Independent Test**:
Pode ser testado abrindo um documento previamente lido e pausado na metade; ao selecioná-lo novamente no seletor de arquivos, o aplicativo apresenta a caixa de diálogo de oferta de retomada (`offerDialog`) em vez de recomeçar o áudio do início de forma cega.

**Acceptance Scenarios**:

1. **Given** um documento que possui um marcador de progresso (bookmark) salvo com encerramento por pausa deliberada, **When** o usuário escolhe esse documento pelo seletor de títulos, **Then** o sistema exibe o diálogo de oferta de retomada ("Continuar de onde parou ou começar do início?") permitindo ao usuário decidir o ponto de reprodução.
2. **Given** um documento que sofreu encerramento forçado/queda brusca (kill), **When** o usuário escolhe esse documento pelo seletor de títulos, **Then** o sistema retoma a reprodução a partir da posição persistida pelo heartbeat sem exibir o diálogo, conforme a política de recuperação de acidentes.

---

### User Story 3 - Cancelamento ou Recusa sem Efeito Colateral na Reprodução Atual (Priority: P3)

Como ouvinte que abriu acidentalmente o seletor ou decidiu não trocar de arquivo,
Quero fechar ou cancelar o seletor de arquivos do sistema sem que o áudio atualmente em reprodução seja corrompido, interrompido ou alterado,
Para que meu momento de escuta continue preservado.

**Why this priority**: Garante robustez contra toques acidentais na área de título e conformidade com o princípio de que o cancelamento de uma operação de I/O não altera o estado corrente do motor de áudio.

**Independent Test**:
Pode ser testado tocando no título com um áudio em execução, pressionando "Voltar" ou "Cancelar" no seletor de arquivos e confirmando que o áudio continua tocando ininterruptamente e os metadados na tela permanecem intactos.

**Acceptance Scenarios**:

1. **Given** que o áudio está em reprodução, **When** o usuário toca no título e em seguida pressiona "Cancelar" ou o botão Voltar do sistema no seletor, **Then** o seletor é fechado, a reprodução permanece no estado anterior (`playing`) e os títulos exibidos não são alterados.
2. **Given** que o usuário seleciona um arquivo que excede o limite orçado de tamanho (93.636 bytes) ou com formato não suportado, **Then** o documento atual permanece carregado e um aviso discreto de recusa de formato/tamanho é registrado sem travar a interface.

---

### Edge Cases

- **Arquivo corrompido, não-UTF8 ou binário**: O sistema recusa com código de erro nomeado (`InvalidUtf8` no Rust e `FormatUndeliverable` no subsistema de playback/superfície), sem alterar o documento atual nem disparar reprodução em falso.
- **Arquivo vazio (0 bytes)**: Recusado imediatamente na borda (`OpenError::EmptyDocument`), emitindo recusa diagnóstica controlada sem alterar o documento atual nem disparar reprodução em falso.
- **Arquivo maior que o limite orçado (93.636 bytes)**: O motor de ingestão emite recusa controlada (`FormatUndeliverable`), o documento atual é mantido, o relógio de transporte não avança e a UI exibe o aviso discreto via `reasonBanner` / `liveRegion` sem travar a interface nem interromper a reprodução ativa.
- **Armazenamento esgotado (disco cheio) na gravação do snapshot**: A falha de I/O em `writeSnapshot` é capturada como `IngestOutcome.Unreadable`, emitindo registro de log/diagnóstico sem avançar o relógio nem corromper o estado do playback ativo.
- **Dispositivo sem provedor de documentos (SAF ausente/desabilitado ou intent não resolvida)**: A chamada `startActivityForResult` trata `ActivityNotFoundException` e exceções gerais com segurança sem causar crash da aplicação.
- **Processo colocado em segundo plano ou finalizado pelo sistema enquanto SAF está aberto**: Ao retornar, o ciclo de vida Android restaura a Activity e a Lente Sinestésica, preservando o estado do documento e respeitando a política de retomada via bookmark durável (`redb`) caso o processo tenha sofrido recriação.
- **Arquivo com conteúdo idêntico mas nome/caminho diferente**: Como o endereçamento e os marcadores de progresso no `redb` são ancorados exclusivamente na impressão digital Blake3-256 dos bytes de conteúdo, os bookmarks existentes são reaproveitados com fidelidade, atualizando apenas o nome de exibição (`filename`) na interface.
- **Cancelamento pelo usuário no seletor (botão Voltar)**: Nenhuma alteração é enviada ao motor de áudio e o documento ativo continua inalterado.
- **Toques repetidos rápidos no bloco de título**: Aberturas consecutivas do seletor são tratadas com debounce para não abrir múltiplas instâncias da intent do sistema.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O bloco de título da Lente Sinestésica (`.meta-box`) DEVE agir como controle interativo em sua área completa com semântica de acessibilidade (`role="button"`, `tabindex="0"`, `aria-label="Selecionar documento para leitura"`). Suas dimensões de alvo de toque DEVEM atender ou exceder o padrão móvel de 48×48dp (WCAG 2.5.5 / 2.5.8), garantindo `min-height: 48px` mesmo no estado inicial/vazio onde o título ainda não foi carregado, com padding mínimo de 0.5rem ao redor do conteúdo para prevenir colapso da área clicável.
- **FR-002**: O toque ou ativação por teclado/TalkBack (`Enter` ou `Space` com `event.preventDefault()`) na área de título DEVE disparar o seletor nativo do sistema operacional (Storage Access Framework `ACTION_OPEN_DOCUMENT`). Ao retornar do seletor (seja por seleção de arquivo ou cancelamento), o foco DEVE ser restaurado explicitamente para o container `.meta-box` na Webview.
- **FR-003**: O seletor de documentos DEVE aceitar estritamente arquivos com tipo MIME `text/markdown` e `text/plain` (extensões `.md` e `.txt`), configurando a intent com as flags de leitura e persistência de permissão de URI (`FLAG_GRANT_READ_URI_PERMISSION` e `FLAG_GRANT_PERSISTABLE_URI_PERMISSION`).
- **FR-004**: Ao selecionar um arquivo novo (sem bookmark prévio de pausa), o sistema DEVE carregar o documento sob autoridade exclusiva do Rust, atualizar o título na tela e iniciar automaticamente a reprodução do áudio (TTS) em menos de 1,0 segundo a partir do retorno do SAF (SC-002). Em conformidade com WCAG 1.4.2 (Audio Control), a reprodução iniciada DEVE permitir pausa imediata a qualquer momento através do controle de transporte existente (`btnPlayPause` / `Space`), além de respeitar a perda transitória ou permanente de foco de áudio (`AudioFocusRequest`) de modo que a fala de leitores de tela (TalkBack) atenue ou silencie o áudio (ducking/pause) conforme as regras do subsistema de áudio.
- **FR-005**: Ao selecionar um arquivo que já possua bookmark de progresso prévio resultante de pausa deliberada, o sistema DEVE respeitar o contrato de retomada (FR-009 / `contracts/resume-algorithm.md`), apresentando a caixa de diálogo de oferta de retomada (`offerDialog`) com as opções explícitas "Continuar" e "Do início" com janela de expiração de 3,0 segundos (conforme `OFFER_WINDOW_MS` / `contracts/surface.md`), após a qual a reprodução continua silenciosamente da posição salva, em vez de sobrescrever silenciosamente a posição.
- **FR-006**: O cancelamento ou recusa do seletor pelo usuário (ex.: pressionar "Voltar" no SAF) NÃO DEVE interromper a reprodução ativa nem modificar o documento em exibição, garantindo isolamento total do transporte e do motor de áudio.
- **FR-007**: A área interativa de título DEVE oferecer feedback visual explícito através dos tokens DESIGN-SONA: cursor `pointer`, `:active` com animação `transform: scale(0.985)` e `transition: transform 120ms ease`, anel de foco `:focus-visible` com `outline: 2px solid var(--sona-etwahl)` e `outline-offset: 4px` (sem contorno em foco não-teclado), e hover state sutil sem comprometer o contraste nem a estética visual da Lente Sinestésica.
- **FR-008**: O disparo do seletor de arquivos DEVE implementar prevenção contra múltiplos toques (debounce / lock com flag booleana durante a ativação) para garantir que toques rápidos consecutivos não abram múltiplas instâncias da intent SAF do sistema operacional.
- **FR-009**: O limite orçado de tamanho do documento DEVE ser rigorosamente de no máximo 93.636 bytes (`MAX_DOC_BYTES`). Documentos que excedam esse limite DEVEM ser rejeitados na borda com recusa controlada (`FormatUndeliverable`) antes do envio de qualquer dado para síntese.
- **FR-010**: O modelo de isolamento de ingestão DEVE gravar os bytes do documento selecionado em snapshot privado (`filesDir/snapshots/<fingerprint>.<ext>`) antes do repasse ao motor Rust, expurgando snapshots anteriores (keep-current-only) para isolar o playback de alterações no arquivo original.
- **FR-011**: As operações de I/O de arquivo no Android (leitura de stream via `ContentResolver`, cálculo de hash e gravação do snapshot) DEVEM ser executadas fora da thread principal de UI (em thread secundária/coroutine de I/O) para prevenir qualquer bloqueio ou jank na renderização do aplicativo.
- **FR-012**: O endereçamento de dados e a consulta/gravação de marcadores de progresso no banco `redb` DEVEM ser fundamentados exclusivamente no hash de conteúdo Blake3-256, e nunca no caminho (path) ou URI do documento. Documentos com mesmo conteúdo compartilham o mesmo estado e bookmark mesmo que renomeados.
- **FR-013**: Ao alternar para um novo documento a partir de um documento ativo em reprodução ou pausado, a posição audível do documento de saída DEVE ser gravada duravelmente no banco de bookmarks (`redb`) sob sua respectiva impressão digital antes que a sessão anterior seja descarregada (`switch_to_document`) e o novo documento assuma o motor de playback.
- **FR-014**: Todas as recusas de ingestão de arquivos (arquivos não-UTF8, extensões não permitidas ou tamanho superior a 93.636 bytes) DEVEM pertencer ao conjunto fechado de razões de recusa (`FormatUndeliverable` para formato/tamanho inadequado e `InvalidUtf8` para decodificação inválida) e NÃO DEVEM conter variantes genéricas catch-all (`_ =>` / `Other`). O erro DEVE emitir um registro de diagnóstico no subsistema de telemetria sem avançar o relógio de posição (`PositionClock`).
- **FR-015**: Quando um arquivo for recusado ou inválido (incluindo arquivos vazios de 0 bytes), a interface DEVE apresentar feedback discreto e não intrusivo através do banner de status existente (`reasonBanner`) com texto acessível anunciado via região ativa (`liveRegion` com `aria-live="polite"`), mantendo a reprodução e o documento atual inalterados.
- **FR-016**: Caso o sistema operacional Android não possua um provedor de documentos compatível ou a chamada do SAF lance uma exceção não tratável (`ActivityNotFoundException`), o aplicativo DEVE capturar a falha com segurança, registrando o log de erro e preservando a sessão corrente sem encerrar o processo (crash). Caso a aplicação seja colocada em segundo plano ou finalizada pelo sistema operacional enquanto o seletor SAF estiver em primeiro plano, o ciclo de vida nativo do Android (`savedInstanceState` / retomada de Activity) DEVE restaurar o estado da Lente Sinestésica sem perda do progresso de leitura ativo.
- **FR-017**: Se o armazenamento local privado (`filesDir/snapshots`) atingir capacidade esgotada durante a tentativa de escrita do snapshot, o erro DEVE ser classificado como falha de E/S (`IngestOutcome.Unreadable`), o arquivo em execução deve permanecer intocado e um registro diagnóstico emitido, sem travar o aplicativo.

### Key Entities *(include if feature involves data)*

- **Document**: Representa o arquivo de texto carregado; possui nome de exibição (`filename`), impressão digital única baseada em hash (Blake3-256), tamanho em bytes e coleção de seções normalizadas para reprodução.
- **Bookmark**: Posição persistida associada à impressão digital do documento; armazena o deslocamento em segundos, o timestamp da última audição e o tipo de encerramento da sessão anterior (`paused`, `completed`, `interrupted`).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: O usuário consegue abrir um novo arquivo compatível e iniciar a escuta em no máximo 2 toques no dispositivo (1 toque no título + 1 toque para confirmar o arquivo no seletor).
- **SC-002**: A síntese e a reprodução audível iniciam em menos de 1 segundo após o seletor de arquivos devolver o documento selecionado.
- **SC-003**: 100% dos cancelamentos de seleção no seletor de arquivos preservam o estado exato de reprodução e metadados anteriores.
- **SC-004**: 100% dos documentos com histórico de pausa deliberada exibem o diálogo de retomada sem perda de progresso prévio.

## Assumptions

- A infraestrutura nativa do Android e a ponte JNI já fornecem o método seguro de leitura de stream e gravação de snapshot atômico (`DocIngest.kt` e `shell_jni.rs`).
- Os tipos de arquivo compatíveis permanecem limitados a `text/markdown` e `text/plain` com limite máximo de tamanho de 93.636 bytes, conforme os contratos orçados de memória do Ressoa.
- O leitor mantém sua operação estritamente offline, sem transferir dados nem depender de servidores externos.
- O diálogo de retomada (`offerDialog`) já existente na Lente Sinestésica é reutilizado diretamente sem necessidade de refatoração visual profunda.
