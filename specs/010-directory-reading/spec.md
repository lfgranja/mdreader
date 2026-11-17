# Feature Specification: Leitura Contínua de Diretórios de Markdown

**Feature Branch**: `010-directory-reading`

**Created**: 2026-11-03

**Status**: Draft

**Input**: User description: "Leitura contínua de diretórios e vaults inteiros de notas Markdown/texto estruturados ou não-organizados com reprodução em fluxo linear sem atrito, baseada no assessment .specify/assessments/directory-reading e Option B (Streaming Chapter Queue)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Selecionar Diretório e Iniciar Reprodução Contínua (Priority: P1) 🎯 MVP

Como ouvinte de notas, documentações ou livros em Markdown organizados em pastas (como vaults do Obsidian),
Quero selecionar uma pasta completa no meu dispositivo através do seletor nativo e escutar o áudio começar imediatamente a partir do primeiro arquivo,
Para que eu possa ouvir todo o conteúdo em sequência sem precisar parar e abrir manualmente cada nota individualmente.

**Why this priority**: É a proposta de valor central da funcionalidade. Elimina a barreira de abertura repetitiva de arquivos unitários e habilita a audição contínua "mãos-livres".

**Independent Test**:
Pode ser testado selecionando uma pasta local com múltiplos arquivos `.md` (ex.: `01-intro.md`, `02-conceito.md`). O sistema inicia o áudio da primeira nota em menos de 3 segundos e, ao término dela, transiciona automaticamente para a segunda nota sem pausas audíveis ou intervenção manual.

**Acceptance Scenarios**:

1. **Given** que o aplicativo está aberto e visível, **When** o usuário aciona a seleção de pasta e confirma um diretório contendo arquivos de texto compatíveis (`.md`, `.txt`), **Then** o sistema aceita a pasta, identifica o primeiro arquivo elegível pela convenção de ordenação natural e inicia a reprodução audível em menos de 3,0 segundos.
2. **Given** que o arquivo atual da pasta em reprodução atinge o final do seu texto, **When** a última frase é dita, **Then** o sistema transiciona de forma fluida (*gapless*) para o início do próximo arquivo ordenado da pasta, atualizando o título da nota e o indicador de capítulo na interface visual sem interrupção do transporte.
3. **Given** que o último arquivo ordenado da pasta conclui sua leitura, **When** o playback termina, **Then** o transporte avança para o estado concluído (`completed`) e o processo encerra a sessão com sucesso.

---

### User Story 2 - Retomada Precisa por Arquivo e Posição Salva (Priority: P2)

Como ouvinte que precisou pausar ou fechar o aplicativo no meio de uma pasta com dezenas de notas,
Quero reabrir essa mesma pasta e ter a minha posição exata de leitura respeitada (tanto o arquivo específico quanto o segundo exato onde parei),
Para que eu não perca meu progresso nem seja forçado a ouvir notas anteriores novamente.

**Why this priority**: Preserva o tempo do usuário e o progresso em coleções longas (como cursos ou livros de 5 a 10 horas de fala), alinhado com o princípio de durabilidade e consistência de sessão do Ressoa.

**Independent Test**:
Pode ser testado tocando uma pasta com 5 notas, pausando no meio do 3º arquivo aos 45 segundos e fechando o app. Ao selecionar a mesma pasta novamente, o sistema apresenta a oferta de retomada indicando o 3º arquivo e, ao confirmar, retoma exatamente nos 45 segundos daquele arquivo.

**Acceptance Scenarios**:

1. **Given** uma pasta cuja leitura foi pausada deliberadamente no meio de um determinado arquivo, **When** o usuário reabre a mesma pasta, **Then** o sistema exibe a oferta de retomada destacando o nome do arquivo atual e o tempo salvo ("Continuar do arquivo 3 aos 45s ou Recomeçar do início?").
2. **Given** que o usuário confirma "Continuar", **When** o playback é retomado, **Then** o áudio reinicia a partir da posição salva do arquivo correspondente, mantendo a fila dos arquivos restantes da pasta enfileirada.
3. **Given** que o usuário opta por "Recomeçar do início", **When** o playback inicia, **Then** a reprodução recomeça do segundo 0.0 do primeiro arquivo da pasta e o bookmark anterior é descartado.

---

### User Story 3 - Navegação Entre Capítulos/Notas e Resiliência contra Arquivos Inválidos (Priority: P3)

Como ouvinte em movimento ou ouvindo pelo fone de ouvido,
Quero poder avançar para o próximo arquivo ou voltar ao arquivo anterior da pasta através de controles dedicados, e ter notas ilegíveis ignoradas silenciosamente,
Para que um arquivo corrompido ou uma nota que desejo pular não trave a minha sessão de escuta.

**Why this priority**: Fornece controle de fluxo de alto nível e garante robustez operacional quando a pasta contém rascunhos acidentais ou arquivos não-UTF8.

**Independent Test**:
Pode ser testado inserindo um arquivo vazio ou binário no meio da pasta numerada. Durante a reprodução, o sistema detecta o arquivo inválido na transição, registra a recusa diagnóstica em silêncio e salta imediatamente para o próximo arquivo válido sem interrupção audível.

**Acceptance Scenarios**:

1. **Given** uma pasta em reprodução, **When** o usuário aciona o comando de "Avançar Capítulo/Arquivo", **Then** a leitura do arquivo atual é interrompida, sua posição é persistida e o sistema inicia imediatamente a reprodução do próximo arquivo da fila ordenada.
2. **Given** uma pasta em reprodução, **When** o usuário aciona o comando de "Capítulo Anterior" nos primeiros 3 segundos do arquivo, **Then** o sistema retrocede para o início do arquivo imediatamente anterior na fila.
3. **Given** uma pasta que contém um arquivo incompatível (não-UTF8, tamanho > 93KB ou arquivo de 0 bytes) entre notas válidas, **When** a transição alcança esse arquivo, **Then** o sistema emite aviso discreto de recusa sem crash, ignora o arquivo ilegível e avança diretamente para o próximo arquivo compatível da pasta.

---

### Edge Cases

- **Pasta vazia ou sem arquivos Markdown/texto suportados**: O seletor recusa a pasta com mensagem acessível no banner ("Nenhum documento de texto compatível encontrado na pasta selecionada"), mantendo a sessão anterior intacta.
- **Arquivo individual na pasta maior que 93.636 bytes (`MAX_DOC_BYTES`)**: O arquivo excedente é recusado de forma isolada com código nomeado (`FormatUndeliverable`), sendo pulado na fila enquanto os demais arquivos válidos continuam sendo tocados normalmente.
- **Pasta contendo notas e subpastas hierárquicas**: Na versão inicial (v1), o leitor indexa os arquivos da pasta raiz selecionada e arquivos de texto contidos em subpastas de 1º nível, aplicando ordenação natural hierárquica (arquivos da raiz ordenados alfanumericamente, seguidos pelas subpastas ordenadas com seus respectivos arquivos).
- **Renomeação de arquivos ou adição de novas notas na pasta após início da leitura**: Como cada nota possui sua própria impressão digital de conteúdo (Blake3-256), a adição de novos arquivos preserva os marcadores das notas já lidas.
- **Revogação de permissão da pasta no Android pelo usuário**: Caso a permissão do diretório seja revogada nas configurações do sistema operacional, qualquer tentativa de acesso captura com segurança a `SecurityException`, notificando o usuário com orientação amigável para selecionar a pasta novamente sem provocar crash.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O sistema DEVE permitir a seleção de diretórios através do Storage Access Framework do sistema operacional (`ACTION_OPEN_DOCUMENT_TREE`) solicitando e persistindo as permissões de leitura duráveis (`takePersistableUriPermission` com `FLAG_GRANT_READ_URI_PERMISSION`). Ao abrir uma nova pasta, o sistema DEVE adotar a política de diretório ativo único, invocando `releasePersistableUriPermission` para concessões de diretórios obsoletos que não possuam sessões ativas, prevenindo vazamentos contra o teto de 512 permissões do Android.
- **FR-002**: O sistema DEVE indexar arquivos com extensões `.md` e `.txt` pertencentes à pasta selecionada em thread secundária de E/S fora da thread principal de interface, aplicando limite de contagem segura para evitar sobrecarga de chamadas de IPC.
- **FR-003**: A ordenação padrão dos arquivos da pasta DEVE seguir o algoritmo de ordenação natural (*natural sort order* / ordenação humana), reconhecendo sequências numéricas (ex.: `01-`, `02-`, `10-`) e ignorando diferenças de caixa alta/baixa. Caso exista um arquivo de índice explícito (`README.md` ou `index.md`) contendo links para os arquivos locais, sua ordem DEVE ter precedência sobre a ordenação alfanumérica pura.
- **FR-004**: Ao selecionar um novo diretório, a síntese e a emissão audível do primeiro arquivo elegível DEVEM iniciar em menos de 3,0 segundos (TTFA), antes do término da varredura completa dos arquivos subsequentes da pasta.
- **FR-005**: A autoridade de transporte do motor de áudio no Rust DEVE orquestrar a fila de capítulos (*ChapterQueue*), gerenciando o carregamento sequencial do próximo arquivo em memória e disparando a transição contínua sem exigir recriação do backend de som. A pré-síntese do próximo capítulo DEVE ser engatada nos últimos 10,0 segundos por padrão (com expansão adaptativa para aparelhos de menor desempenho computacional, desde que não degrade o playback ativo em primeiro plano), com cancelamento cooperativo imediato e expurgo de amostras caso o usuário execute saltos manuais ou seeks.
- **FR-006**: Cada arquivo contido no diretório DEVE ser avaliado individualmente contra o teto orçado de tamanho (`MAX_DOC_BYTES = 93.636 bytes`). Arquivos que excedam o limite DEVEM ser recusados individualmente com `FormatUndeliverable` sem interromper a execução do restante da pasta.
- **FR-007**: Ao concluir a leitura de um arquivo da pasta, o sistema DEVE gravar duravelmente o marcador desse arquivo como concluído (`completed`) e salvar imediatamente o início do novo arquivo no banco local (`redb`) através de uma transação atômica única no banco, prevenindo inconsistência em desligamentos bruscos.
- **FR-008**: O armazenamento de progresso de diretórios DEVE utilizar tabela dedicada no banco `redb` (`DIRECTORY_BOOKMARKS: TableDefinition<&[u8; 32], (u32, f64, u64, u8)>`), preservando 100% da integridade da tabela de documentos únicos. A chave primária DEVE ser calculada pela impressão digital Blake3-256 da coleção de arquivos ordenados (`blake3(fp_1 || fp_2 || ... || fp_n)`), armazenando a tupla: índice ordinal do arquivo ativo (`u32`), deslocamento em segundos no arquivo atual (`f64`), timestamp (`u64`) e discriminante estrito de `SessionEnd` (`0=Paused, 1=Stopped, 2=Completed, 3=Interrupted`). Durante a reprodução contínua em `Playing`, o sistema DEVE gravar um heartbeat durável a cada 20 segundos sob `SessionEnd::Interrupted`, garantindo retomada precisa após quedas de processo (hard kill replay ≤ 30s).
- **FR-009**: Ao reabrir uma pasta com leitura em andamento, o sistema DEVE exibir o diálogo de oferta de retomada com tempo de exibição e comportamento idênticos ao contrato de retomada do Ressoa, indicando o nome do arquivo atual e o ponto de reprodução salvo.
- **FR-010**: A interface visual (Lente Sinestésica) DEVE acomodar os metadados de capítulo preservando rigorosamente o teto de altura de `100dvh` (sem barra de rolagem e com CLS = 0.00). O título do arquivo ativo DEVE ocupar o `<h1>` (com clamp de 2 linhas), enquanto o elemento de subtítulo (`#sectionTitle` / `.subtitle`) DEVE exibir o contador de capítulos estruturado com numerais tabulares e a seção corrente (`"Capítulo X de Y • [Nome da Seção]"`). Leitores de tela (TalkBack) DEVEM anunciar transições de capítulo via `#liveRegion` com `aria-live="polite"` e `aria-atomic="true"`, sem deslocar o foco visual.
- **FR-011**: Os controles de transporte em fones de ouvido Bluetooth, tela de bloqueio e Android Auto (`ACTION_SKIP_TO_NEXT` e `ACTION_SKIP_TO_PREVIOUS`) DEVEM operar em modelo de navegação hierárquica/contextual: o comando avança/retrocede entre as seções/títulos internos do arquivo atual; caso o leitor já se encontre na última seção do arquivo atual, o comando avança de forma fluida para o início do próximo arquivo do diretório, impedindo que o motor permaneça preso no final do arquivo sem intervenção visual.
- **FR-012**: Arquivos vazios (0 bytes), binários ou com codificação não-UTF8 DEVEM ser ignorados durante a transição com aviso discreto registrado na telemetria diagnóstica, sem travar o transporte e sem produzir silêncio perpétuo.
- **FR-013**: O sistema DEVE operar estritamente offline, sem transferir metadados, títulos ou conteúdo de arquivos através de rede ou requisições remotas, em plena conformidade com o Princípio IX da Constituição.

### Key Entities *(include if feature involves data)*

- **DirectorySession**: Representa o contexto da pasta aberta; contém a URI persistida da árvore, nome da pasta raiz, lista ordenada de capítulos identificados, contagem total de arquivos e índice do capítulo atualmente ativo.
- **ChapterItem**: Representa um arquivo individual dentro do diretório; contém nome do arquivo, URI específica do arquivo, caminho relativo dentro da pasta, tamanho em bytes, impressão digital Blake3-256 e status de leitura (`pending`, `playing`, `completed`, `skipped`).
- **CompositeBookmark**: Extensão do marcador de progresso atrelado à pasta; armazena a URI da árvore, a impressão digital do arquivo ativo, o índice ordinal do capítulo, o offset em segundos e o tipo de encerramento da última audição.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% de transição automática audível do arquivo `N` para o arquivo `N+1` sem necessidade de qualquer toque na tela durante a reprodução contínua da pasta.
- **SC-002**: Tempo até o primeiro som (TTFA) inferior a 3,0 segundos a partir da seleção de um diretório contendo até 100 arquivos Markdown.
- **SC-003**: 100% de precisão na restauração de progresso ao reabrir uma pasta pausada, retornando ao arquivo exato e ao mesmo segundo salvo.
- **SC-004**: Menos de 1 segundo de intervalo de silêncio na transição entre o término de um arquivo e o início da fala do próximo arquivo da fila.

## Assumptions

- O dispositivo do usuário suporta o seletor padrão Android Storage Access Framework (`ACTION_OPEN_DOCUMENT_TREE`) disponível a partir da API 29 (Android 10), que é o `minSdk` do projeto.
- A maioria dos usuários organiza sequências deliberadas de notas utilizando prefixos numéricos (`01-`, `02-`, etc.) ou títulos em listas em arquivos `README.md`.
- A capacidade de processamento do dispositivo permite sintetizar os primeiros blocos de áudio da próxima nota enquanto os últimos segundos da nota atual ainda estão sendo ouvidos.
- Diretórios contendo mais de 500 arquivos de texto são casos atípicos em vaults pessoais de anotações; a indexação de até 100 arquivos atende a grande maioria dos ouvintes com folga de desempenho.
