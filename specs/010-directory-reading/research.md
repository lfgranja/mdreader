# Phase 0 Research: Leitura Contínua de Diretórios de Markdown

- **Feature**: `010-directory-reading`
- **Date**: 2026-11-03
- **Status**: Complete (All architectural decisions resolved)

## Technical Decisions & Rationale

### Decision 1: SAF `ACTION_OPEN_DOCUMENT_TREE` com Varredura em Coroutine e Permissões Persistentes
- **Decision**: 
  - Utilizar a Intent do Android Storage Access Framework `Intent.ACTION_OPEN_DOCUMENT_TREE` acionada a partir da `MainActivity` / `ShellHost`.
  - Persistir explicitamente os privilégios duráveis de leitura da árvore com `contentResolver.takePersistableUriPermission(treeUri, Intent.FLAG_GRANT_READ_URI_PERMISSION)`.
  - Executar a indexação e varredura dos documentos da pasta em coroutine de segundo plano (`Dispatchers.IO`), limitando a profundidade à pasta raiz e subpastas de 1º nível.
  - Implementar ordenação natural alfanumérica (*human/natural sort order*) que reconhece blocos numéricos (ex.: `01-`, `02-`, `10-`) e confere precedência a listas de links locais encontradas em arquivos de índice (`README.md` ou `index.md`).
- **Rationale**:
  - `ACTION_OPEN_DOCUMENT_TREE` é a API padrão suportada a partir do Android 5.0 (API 21) e plenamente aderente ao `minSdk = 29` (Android 10) do Ressoa, sem exigir permissões amplas de armazenamento (`MANAGE_EXTERNAL_STORAGE` proibida pelo Google Play e pelas diretrizes de privacidade).
  - A varredura de Content Providers / SAF no Android é notoriamente síncrona e lenta quando envolve dezenas de nós. Executar em `Dispatchers.IO` impede ANRs (Application Not Responding) e garante que o primeiro arquivo identificado seja imediatamente despachado para a síntese no Rust antes da conclusão da varredura total dos arquivos subsequentes, cumprindo o teto de TTFA < 3.0s (FR-004 / SC-002).
- **Alternatives Considered**:
  - `ACTION_OPEN_DOCUMENT` com seleção múltipla (`EXTRA_ALLOW_MULTIPLE`): Rejeitado porque exige que o usuário selecione manualmente todos os arquivos individuais e não preserva o contexto da pasta nem suporta adições dinâmicas de novas notas.
  - Varredura recursiva irrestrita: Rejeitada para evitar estouro de memória e travamentos em vaults gigantes com milhares de arquivos em diretórios aninhados (`.git`, `.obsidian`, node_modules). O limite de 1º nível atende o caso de uso primário (livros, cursos e capítulos).
- **Invariants Preserved**:
  - Sem permissões invasivas ou acesso fora do SAF.
  - Sem bloqueio da thread principal (UI responsiva com CLS = 0.00).
  - Pure-Offline: nenhuma comunicação de rede para resolução de metadados.

---

### Decision 2: Orquestração de Fila de Capítulos (*ChapterQueue*) no Rust (`crates/player` / `AppState`)
- **Decision**:
  - Manter a autoridade de reprodução integralmente no Rust criando a estrutura `ChapterQueue` dentro de `crates/player` e gerenciada no `AppState` do bridge Tauri.
  - Quando um arquivo termina (`DocumentEnd`), o Rust não recria o backend `cpal`/`AAudio`; em vez disso, o `ChapterQueue` consome o próximo capítulo válido da fila, carrega seus bytes, extrai títulos/headings e conecta o novo fluxo de áudio diretamente no buffer de síntese.
  - Efetuar pré-aquecimento (*pre-roll*) do próximo capítulo durante os últimos 5 segundos do capítulo atual para garantir transição estritamente contínua (*gapless*, intervalo de silêncio < 1,0s per SC-004).
- **Rationale**:
  - Princípio I (Rust Sole Playback Authority): o Android shell é apenas um espelho (`MediaMirror`). O ciclo de vida do transporte, a contagem de amostras, a detecção de término e o avanço para o próximo arquivo devem residir onde o relógio de áudio (`PositionClock`) opera.
  - Recriar o pipeline ou reinicializar o `cpal::Stream` provocaria estalos audíveis (*audio pops*), latência de reconexão de hardware e perda de foco de áudio.
- **Alternatives Considered**:
  - Notificar o Kotlin no término de cada nota para que o Android chame `openDocument` novamente: Rejeitado categoricamente por violar a autoridade do Rust e o Princípio VII/VIII, introduzindo dependência de IPC round-trip que causaria engasgos audíveis (gaps de 500ms a 2s).
  - Concatenar todos os textos dos arquivos em uma string única gigante na memória: Rejeitado porque quebraria a integridade dos bookmarks por nota, violaria o limite de orçamento de documento (`MAX_DOC_BYTES = 93.636 bytes`), distorceria o cálculo de progresso e causaria latência de síntese excessiva no primeiro chunk.
- **Invariants Preserved**:
  - Handoff de áudio invariante: mono `f32` @ `22050 Hz`.
  - Posição avança apenas com progresso audível (`PositionClock`).
  - Shell Kotlin permanece 100% como espelho passivo (*mirror only*).

---

### Decision 3: Tabela `DIRECTORY_BOOKMARKS` no `redb` com Chave Composta via Hash Blake3 da Coleção
- **Decision**:
  - Criar uma nova tabela no banco transacional `redb` em `crates/session/src/store.rs`:
    ```rust
    pub const DIRECTORY_BOOKMARKS: TableDefinition<&[u8; 32], (u32, f64, u64, u8)> =
        TableDefinition::new("directory_bookmarks");
    ```
  - A chave primária `[u8; 32]` é o hash Blake3-256 resultante da composição ordenada dos hashes Blake3 dos arquivos da pasta:
    `blake3(fp_1 || fp_2 || ... || fp_n)`.
  - O valor armazenado é a tupla:
    1. `u32`: índice ordinal do capítulo ativo (0-based).
    2. `f64`: deslocamento salvo em segundos dentro do capítulo atual.
    3. `u64`: timestamp Unix em milissegundos da gravação.
    4. `u8`: código numérico fechado de `SessionEnd` (`Paused` ou `Interrupted`).
- **Rationale**:
  - O Ressoa indexa bookmarks por conteúdo (`blake3-256` dos bytes do documento), nunca pelo caminho em disco. Para um diretório, a identidade do corpus é o conjunto ordenado dos seus conteúdos.
  - Manter uma tabela separada (`directory_bookmarks`) isola as mutações e evita colisões ou poluição na tabela `BOOKMARKS` de arquivos unitários existente, garantindo migração limpa sem quebra de esquema prévio.
  - Ao reabrir a pasta, o hash composto é recomputado instantaneamente. Havendo correspondência com `SessionEnd::Paused`, o diálogo de oferta (`offerDialog`) é apresentado na interface por 3s com o nome do arquivo e posição exata (FR-009 / SC-003). Se o usuário escolher recomeçar, o índice volta a 0 no segundo 0.0.
- **Alternatives Considered**:
  - Usar a URI da árvore SAF como chave no banco: Rejeitado porque URIs no Android podem variar (ex.: cópia do vault para outro diretório, formatação do cartão SD, reinstalação do app) violando o Princípio de independência de caminho.
  - Salvar apenas na tabela `BOOKMARKS` individual o hash do arquivo onde parou: Rejeitado porque ao reabrir a pasta o sistema não saberia qual era o contexto da pasta nem a ordenação da fila sem inspecionar cada arquivo.
- **Invariants Preserved**:
  - Crash-safe e atomicidade transacional do `redb`.
  - Sem strings de caminho como chaves de durabilidade.
  - `SessionEnd` como conjunto fechado sem `_ =>`.

---

### Decision 4: Controles de Mídia Contextuais e Hierárquicos (Skip Intra-arquivo vs Inter-arquivo)
- **Decision**:
  - Os comandos de transporte remotos (`ACTION_SKIP_TO_NEXT` e `ACTION_SKIP_TO_PREVIOUS`, recebidos de fones Bluetooth, lock-screen e Android Auto) operam com semântica contextual:
    - **Avançar (`SkipToNext`)**: Se o arquivo atual possui múltiplos títulos/seções (`headings`) e o cursor não está na última seção, o comando salta para o início da próxima seção interna (comportamento SONA padrão). Caso o cursor já esteja na última seção (ou o documento não tenha seções intermediárias), o comando avança para o início do próximo capítulo da pasta (`ChapterQueue::next()`).
    - **Retroceder (`SkipToPrevious`)**: Se o cursor estiver após os primeiros 3 segundos da seção atual, ele volta ao início dessa seção. Se estiver nos primeiros 3 segundos da primeira seção do arquivo atual e houver um capítulo anterior na pasta, ele retrocede para o capítulo anterior (`ChapterQueue::previous()`).
  - No frontend da Lente Sinestésica, o botão `#btn-repeat` preserva sua função de repetir a seção atual, e botões de capítulo dedicados ou atalhos de navegação acionam explicitamente os comandos `next_chapter` e `previous_chapter`.
- **Rationale**:
  - Atende plenamente ao Princípio VIII (Driving-First / Não-distrativo): ouvintes usando fones de ouvido no volante ou em caminhada não precisam olhar para a tela para alternar entre notas de uma pasta quando chegam ao fim de um capítulo.
  - Evita que a reprodução fique "presa" indefinidamente ao final de um arquivo curto sem avanço manual.
- **Alternatives Considered**:
  - Atribuir o comando de skip do lock screen exclusivamente para troca de arquivo: Rejeitado porque impediria a navegação dentro de notas longas que contenham seções estruturadas.
  - Botões separados no lock screen para "próxima seção" e "próximo arquivo": Rejeitado porque a `MediaStyle` notification do Android impõe teto estrito de ações compactas (3 ações: anterior, play/pause, próximo).
- **Invariants Preserved**:
  - Vocabulário fechado de verbos IPC (`ShellVerb`).
  - Navegação auditiva sem exigência de atenção visual.

---

### Decision 5: Subtítulo da Lente Sinestésica com Contador de Capítulos, Numerais Tabulares e Região Live Acessível
- **Decision**:
  - Preservar rigorosamente o teto de altura de `100dvh` na interface visual (`status_view.html`), sem qualquer rolagem (scroll) e com CLS = 0.00.
  - O título do arquivo ativo (`ChapterItem.filename`) ocupa o `<h1>` (com clamp de até 2 linhas).
  - O elemento de subtítulo (`#sectionTitle` / `.subtitle`) renderiza o contador de capítulos estruturado com numerais tabulares:
    `"Capítulo {current} de {total} • {section_name}"` (ou `"Capítulo {current} de {total}"` se não houver heading ativo).
  - A fonte utiliza `font-variant-numeric: tabular-nums` para impedir oscilações de layout durante mudanças de dígitos.
  - Leitores de tela (TalkBack) recebem atualizações de transição de capítulo através do `#liveRegion` com `aria-live="polite"` e `aria-atomic="true"`, sem movimentação forçada do foco de acessibilidade.
- **Rationale**:
  - Respeita integralmente o design SONA e as diretrizes de acessibilidade WCAG 2.1 (AA) e 4.1.3 (Status Messages).
  - Garante que ouvintes com baixa visão ou usuários de TalkBack sejam informados com discrição da mudança de capítulo sem interromper a fala sintetizada em andamento.
- **Alternatives Considered**:
  - Criar um novo card visual para exibir a lista completa de capítulos: Rejeitado categoricamente pelo Princípio VIII (Driving-First) e pela regra de pureza da tela única minimalista da Lente Sinestésica.
  - Anunciar por fala (TTS) a mudança de capítulo: Rejeitado porque a inserção de áudio falado artificial quebra a integridade da leitura do texto original. A interface visual e os earcons são os canais corretos para metadados de navegação.
- **Invariants Preserved**:
  - Zero CLS (Cumulative Layout Shift = 0.00).
  - Altura cravada em `100dvh`.
  - Paleta DESIGN-SONA fechada (título `#F2F0E6`, subtítulo `#A9B8CE`).
